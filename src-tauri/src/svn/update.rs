use crate::common::AppError;
use crate::svn::models::{SvnUpdateEvent, UpdateFileItem, UpdateResult};
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tauri::{AppHandle, Emitter};

/// Parse svn update output into (revision, files_without_authors).
/// Each file entry is (path, status_char).
fn parse_update_output(output: &str) -> Result<(u64, Vec<(String, String)>), AppError> {
    let mut revision: u64 = 0;
    let mut files = Vec::new();

    for line in output.lines() {
        let trimmed = line.trim();

        // "Updated revision 105."
        if let Some(rest) = trimmed.strip_prefix("Updated to revision ") {
            if let Some(rev_str) = rest.strip_suffix('.') {
                if let Ok(rev) = rev_str.parse::<u64>() {
                    revision = rev;
                }
            }
        }

        // Skip non-status lines before extracting the status character.
        // Without this, "Updating '.':" (U), "At revision 100." (A),
        // and "Updated revision 105." (U) would be mis-parsed.
        if trimmed.starts_with("Updating")
            || trimmed.starts_with("Updated")
            || trimmed.starts_with("At ")
            || trimmed.starts_with("Summary")
        {
            continue;
        }

        // Status lines: "A    new_file.rs" or "U   +  existing.rs"
        if trimmed.len() >= 2 {
            let status_char = trimmed.as_bytes()[0];
            let path_part = if trimmed.as_bytes().get(1) == Some(&b' ') {
                let after_status = &trimmed[1..];
                if let Some(pos) = after_status.find(|c: char| c.is_alphabetic()) {
                    after_status[pos..].trim()
                } else {
                    after_status.trim()
                }
            } else {
                &trimmed[1..]
            };

            let path = path_part.trim();
            if path.is_empty() {
                continue;
            }

            match status_char {
                b'A' | b'U' | b'M' | b'C' => {
                    let status = (status_char as char).to_string();
                    files.push((path.to_string(), status));
                }
                _ => {}
            }
        }
    }

    Ok((revision, files))
}

/// Query svn log for a single revision to get the author per changed file.
async fn fetch_authors_for_revision(
    path: &str,
    revision: u64,
    timeout_secs: u64,
) -> Result<std::collections::HashMap<String, String>, AppError> {
    let rev_str = revision.to_string();
    let xml = crate::svn::run_svn_utf8_async(
        &["log", "--xml", "-v", "-r", &rev_str, path],
        timeout_secs,
    )
    .await?;
    let entries = crate::svn::log::parse_log_xml(&xml)?;

    let mut author_map = std::collections::HashMap::new();
    if let Some(entry) = entries.into_iter().next() {
        if let Some(changed_paths) = entry.changed_paths {
            for cp in changed_paths {
                // SVN log paths have leading "/" and are repo-relative
                // (e.g., "/trunk/file.rs"). Strip leading "/" to match
                // the local relative paths from svn update output.
                let normalized = cp.path.trim_start_matches('/');
                author_map.insert(normalized.to_string(), entry.author.clone());
            }
        }
    }
    Ok(author_map)
}

pub async fn svn_update(path: &str, timeout_secs: u64) -> Result<UpdateResult, AppError> {
    let output = crate::svn::run_svn_async_in_dir(&["update"], timeout_secs, Some(path)).await?;
    let (revision, raw_files) = parse_update_output(&output)?;

    if raw_files.is_empty() {
        return Ok(UpdateResult {
            revision,
            files: Vec::new(),
        });
    }

    let author_map = fetch_authors_for_revision(path, revision, timeout_secs)
        .await
        .unwrap_or_default();

    let files = raw_files
        .into_iter()
        .map(|(file_path, status)| {
            let author = author_map.get(&file_path).cloned().unwrap_or_default();
            UpdateFileItem {
                path: file_path,
                status,
                author,
            }
        })
        .collect();

    Ok(UpdateResult { revision, files })
}

/// Parse a single line from `svn update` output. Returns an event if the line
/// is a file status or revision summary line, or `None` for irrelevant lines.
fn parse_single_update_line(line: &str) -> Option<SvnUpdateEvent> {
    let trimmed = line.trim();

    if let Some(rest) = trimmed.strip_prefix("Updated to revision ") {
        if let Some(rev_str) = rest.strip_suffix('.') {
            if let Ok(rev) = rev_str.parse::<u64>() {
                return Some(SvnUpdateEvent::Done { revision: rev });
            }
        }
    }

    if trimmed.starts_with("Updating")
        || trimmed.starts_with("Updated")
        || trimmed.starts_with("At ")
        || trimmed.starts_with("Summary")
    {
        return None;
    }

    if trimmed.len() >= 2 {
        let status_char = trimmed.as_bytes()[0];
        let path_part = if trimmed.as_bytes().get(1) == Some(&b' ') {
            let after_status = &trimmed[1..];
            if let Some(pos) = after_status.find(|c: char| c.is_alphabetic()) {
                after_status[pos..].trim()
            } else {
                after_status.trim()
            }
        } else {
            &trimmed[1..]
        };

        let path = path_part.trim();
        if path.is_empty() {
            return None;
        }

        match status_char {
            b'A' | b'U' | b'M' | b'C' => {
                return Some(SvnUpdateEvent::File {
                    status: (status_char as char).to_string(),
                    path: path.to_string(),
                });
            }
            _ => {}
        }
    }

    None
}

fn emit_update_event(app: &AppHandle, event: &SvnUpdateEvent, found_files: &mut bool) {
    match event {
        SvnUpdateEvent::File { .. } => {
            *found_files = true;
            let _ = app.emit("svn_update_progress", event);
        }
        SvnUpdateEvent::Done { .. } => {
            let _ = app.emit("svn_update_progress", event);
        }
        _ => {}
    }
}

/// Stream svn update, emitting file events in real time.
pub async fn svn_update_streaming(path: &str, timeout_secs: u64, app: &AppHandle) -> Result<(), AppError> {
    let svn_path = crate::svn::find_svn_executable()?;

    let mut cmd = Command::new(&svn_path);
    cmd.arg("update");
    cmd.current_dir(path);
    cmd.env("OUTPUT_CHARSET", "UTF-8");
    cmd.env("LANG", "en_US.UTF-8");
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| AppError::Svn(format!("Failed to execute svn: {}", e)))?;

    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| AppError::Svn("Failed to capture svn stdout".to_string()))?;

    let mut buf = Vec::new();
    let mut found_files = false;

    let result = tokio::time::timeout(Duration::from_secs(timeout_secs), async {
        let mut tmp = [0u8; 4096];
        loop {
            let n = stdout.read(&mut tmp).await.map_err(|e| AppError::Svn(format!("Failed to read svn output: {}", e)))?;
            if n == 0 {
                // Flush remaining bytes
                if !buf.is_empty() {
                    let line = crate::svn::decode_bytes(&buf);
                    if let Some(event) = parse_single_update_line(&line) {
                        emit_update_event(app, &event, &mut found_files);
                    }
                    buf.clear();
                }
                break;
            }
            buf.extend_from_slice(&tmp[..n]);
            while let Some(pos) = buf.iter().position(|&b| b == b'\n') {
                let line_bytes = buf[..pos].to_vec();
                buf.drain(..=pos);
                let line = crate::svn::decode_bytes(&line_bytes);
                if let Some(event) = parse_single_update_line(&line) {
                    emit_update_event(app, &event, &mut found_files);
                }
            }
        }
        Ok::<(), AppError>(())
    })
    .await;

    match result {
        Ok(Ok(())) => {
            let status = child
                .wait()
                .await
                .map_err(|e| AppError::Svn(format!("Failed to wait for svn: {}", e)))?;
            if !status.success() {
                let msg = format!("SVN command failed with exit code: {}", status);
                let _ = app.emit("svn_update_progress", SvnUpdateEvent::Error { message: msg.clone() });
                return Err(AppError::Svn(msg));
            }
            if !found_files {
                let _ = app.emit("svn_update_progress", SvnUpdateEvent::UpToDate { revision: 0 });
            }
            Ok(())
        }
        Ok(Err(e)) => {
            let _ = app.emit("svn_update_progress", SvnUpdateEvent::Error { message: e.to_string() });
            Err(e)
        }
        Err(_) => {
            let msg = format!("SVN command timed out after {} seconds", timeout_secs);
            let _ = app.emit("svn_update_progress", SvnUpdateEvent::Error { message: msg.clone() });
            let _ = child.kill().await;
            Err(AppError::Svn(msg))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_update_output_with_changes() {
        let output = "Updating '.':\nA    new_file.rs\nU    existing.rs\nUpdated to revision 105.\n";
        let (revision, files) = parse_update_output(output).unwrap();
        assert_eq!(revision, 105);
        assert_eq!(files.len(), 2);
        assert_eq!(files[0], ("new_file.rs".to_string(), "A".to_string()));
        assert_eq!(files[1], ("existing.rs".to_string(), "U".to_string()));
    }

    #[test]
    fn test_parse_update_output_no_changes() {
        let output = "Updating '.':\nAt revision 100.\n";
        let (revision, files) = parse_update_output(output).unwrap();
        assert_eq!(revision, 0);
        assert!(files.is_empty());
    }

    #[test]
    fn test_parse_update_output_conflicts() {
        let output = "Updating '.':\nC    conflict.rs\nA    ok.rs\nUpdated to revision 200.\n";
        let (revision, files) = parse_update_output(output).unwrap();
        assert_eq!(revision, 200);
        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|(p, s)| p == "conflict.rs" && s == "C"));
    }
}
