use crate::common::AppError;
use crate::svn::models::UpdateResult;

fn parse_update_output(output: &str) -> Result<UpdateResult, AppError> {
    let mut revision: u64 = 0;
    let mut updated_files = Vec::new();
    let mut merged_files = Vec::new();
    let mut conflicts = Vec::new();

    for line in output.lines() {
        let trimmed = line.trim();

        // "Updated revision 105."
        if let Some(rest) = trimmed.strip_prefix("Updated revision ") {
            if let Some(rev_str) = rest.strip_suffix('.') {
                if let Ok(rev) = rev_str.parse::<u64>() {
                    revision = rev;
                }
            }
        }

        // Status lines: "A    new_file.rs" or "A   +  new_file.rs"
        if trimmed.len() >= 2 {
            let status_char = trimmed.as_bytes()[0];
            let path_part = if trimmed.as_bytes().get(1) == Some(&b' ') {
                // "A + file" format - skip status + flag
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
            if path.is_empty() || path.starts_with("Updating") || path.starts_with("Summary") {
                continue;
            }

            match status_char {
                b'A' => updated_files.push(path.to_string()),
                b'U' => updated_files.push(path.to_string()),
                b'M' => merged_files.push(path.to_string()),
                b'C' => conflicts.push(path.to_string()),
                _ => {}
            }
        }
    }

    Ok(UpdateResult {
        revision,
        updated_files,
        merged_files,
        conflicts,
    })
}

pub async fn svn_update(path: &str, timeout_secs: u64) -> Result<UpdateResult, AppError> {
    let output = crate::svn::run_svn_async_in_dir(&["update"], timeout_secs, Some(path)).await?;
    parse_update_output(&output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_update_output_with_changes() {
        let output = "Updating '.':\nA    new_file.rs\nU    existing.rs\nUpdated revision 105.\n";
        let result = parse_update_output(output).unwrap();
        assert_eq!(result.revision, 105);
        assert_eq!(result.updated_files.len(), 2);
        assert!(result.merged_files.is_empty());
    }

    #[test]
    fn test_parse_update_output_no_changes() {
        let output = "Updating '.':\nAt revision 100.\n";
        let result = parse_update_output(output).unwrap();
        assert_eq!(result.revision, 0);
        assert!(result.updated_files.is_empty());
    }

    #[test]
    fn test_parse_update_output_conflicts() {
        let output = "Updating '.':\nC    conflict.rs\nA    ok.rs\nUpdated revision 200.\n";
        let result = parse_update_output(output).unwrap();
        assert_eq!(result.revision, 200);
        assert_eq!(result.conflicts.len(), 1);
        assert_eq!(result.conflicts[0], "conflict.rs");
    }
}
