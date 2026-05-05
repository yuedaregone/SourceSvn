use crate::common::AppError;
use crate::svn::models::SvnUpdateEvent;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tauri::{AppHandle, Emitter};
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

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
    #[cfg(target_os = "windows")]
    cmd.as_std_mut().creation_flags(0x08000000); // CREATE_NO_WINDOW
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
