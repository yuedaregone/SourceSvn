use crate::common::AppError;
use crate::svn::models::{CommitResult, FileStatusType};
use std::collections::HashSet;

pub async fn svn_commit(
    path: &str,
    message: &str,
    files: &[String],
    timeout_secs: u64,
) -> Result<CommitResult, AppError> {
    auto_add_unversioned(path, files, timeout_secs).await?;

    let mut args = vec!["commit", "-m", message];
    for f in files {
        args.push(f);
    }
    let output = crate::svn::run_svn_async_in_dir(&args, timeout_secs, Some(path)).await?;

    let revision = extract_revision_from_output(&output);
    Ok(CommitResult {
        revision,
        success: true,
        errors: None,
    })
}

async fn auto_add_unversioned(
    path: &str,
    files: &[String],
    timeout_secs: u64,
) -> Result<(), AppError> {
    let status_xml =
        crate::svn::run_svn_async_in_dir(&["status", "--xml"], timeout_secs, Some(path)).await?;
    let statuses = crate::svn::status::parse_status_xml(&status_xml)?;

    let unversioned: HashSet<&str> = statuses
        .iter()
        .filter(|s| s.status == FileStatusType::Unversioned)
        .map(|s| s.path.as_str())
        .collect();

    let to_add: Vec<&String> = files
        .iter()
        .filter(|f| unversioned.contains(f.as_str()))
        .collect();

    if to_add.is_empty() {
        return Ok(());
    }

    let mut add_args = vec!["add"];
    for f in &to_add {
        add_args.push(f);
    }
    crate::svn::run_svn_async_in_dir(&add_args, timeout_secs, Some(path)).await?;

    Ok(())
}

fn extract_revision_from_output(output: &str) -> u64 {
    for line in output.lines() {
        if line.contains("Committed revision") {
            if let Some(rev_str) = line.split_whitespace().last() {
                if let Ok(rev) = rev_str.trim_end_matches('.').parse::<u64>() {
                    return rev;
                }
            }
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_revision() {
        let output = "Sending        src/main.rs\nCommitted revision 42.\n";
        assert_eq!(extract_revision_from_output(output), 42);
    }

    #[test]
    fn test_extract_revision_no_match() {
        let output = "No changes.\n";
        assert_eq!(extract_revision_from_output(output), 0);
    }
}
