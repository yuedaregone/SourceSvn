use crate::common::AppError;
use crate::svn::models::{UpdateFileItem, UpdateResult};

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
