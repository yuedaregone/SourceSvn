use crate::common::AppError;
use crate::svn::models::{CommitResult, FileStatusType};
use std::collections::HashSet;

/// Commit with -m and platform-specific encoding parameter.
/// On Windows, Rust's Command converts UTF-8 strings to GBK (system code page),
/// so we specify --encoding GBK. On other platforms, UTF-8 is preserved.
async fn commit_with_message(
    path: &str,
    message: &str,
    files: &[String],
    timeout_secs: u64,
) -> Result<String, AppError> {
    #[cfg(target_os = "windows")]
    let encoding = "GBK";
    #[cfg(not(target_os = "windows"))]
    let encoding = "UTF-8";

    let mut args = vec!["commit", "-m", message, "--encoding", encoding];
    for f in files {
        args.push(f);
    }
    crate::svn::run_svn_async_in_dir(&args, timeout_secs, Some(path)).await
}

pub async fn svn_commit(
    path: &str,
    message: &str,
    files: &[String],
    timeout_secs: u64,
) -> Result<CommitResult, AppError> {
    auto_add_unversioned(path, files, timeout_secs).await?;
    auto_delete_missing(path, files, timeout_secs).await?;

    let output = commit_with_message(path, message, files, timeout_secs).await?;

    let revision = extract_revision_from_output(&output);

    Ok(CommitResult {
        revision,
        success: true,
        errors: None,
    })
}

async fn auto_delete_missing(
    path: &str,
    files: &[String],
    timeout_secs: u64,
) -> Result<(), AppError> {
    let mut status_args = vec!["status", "--xml"];
    for f in files {
        status_args.push(f);
    }
    let status_xml =
        crate::svn::run_svn_async_in_dir(&status_args, timeout_secs, Some(path)).await?;
    let statuses = crate::svn::status::parse_status_xml(&status_xml)?;

    let missing: HashSet<&str> = statuses
        .iter()
        .filter(|s| s.status == FileStatusType::Missing)
        .map(|s| s.path.as_str())
        .collect();

    let to_delete: Vec<&String> = files
        .iter()
        .filter(|f| {
            if missing.contains(f.as_str()) {
                return true;
            }
            let file_path = std::path::Path::new(f);
            let repo_path = std::path::Path::new(path);
            let rel = file_path.strip_prefix(repo_path).unwrap_or(file_path);
            let rel_normalized: String = rel
                .components()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            let rel_normalized = rel_normalized.trim_start_matches('/');
            missing.contains(rel_normalized)
        })
        .collect();

    if to_delete.is_empty() {
        return Ok(());
    }

    let mut delete_args = vec!["delete"];
    for f in &to_delete {
        delete_args.push(f);
    }
    crate::svn::run_svn_async_in_dir(&delete_args, timeout_secs, Some(path)).await?;

    Ok(())
}

async fn auto_add_unversioned(
    path: &str,
    files: &[String],
    timeout_secs: u64,
) -> Result<(), AppError> {
    // Query status only for the specific files, not the entire repo
    let mut status_args = vec!["status", "--xml"];
    for f in files {
        status_args.push(f);
    }
    let status_xml =
        crate::svn::run_svn_async_in_dir(&status_args, timeout_secs, Some(path)).await?;
    let statuses = crate::svn::status::parse_status_xml(&status_xml)?;

    let unversioned: HashSet<&str> = statuses
        .iter()
        .filter(|s| s.status == FileStatusType::Unversioned)
        .map(|s| s.path.as_str())
        .collect();

    let to_add: Vec<&String> = files
        .iter()
        .filter(|f| {
            if unversioned.contains(f.as_str()) {
                return true;
            }
            // Normalize to repo-relative path using Path::components for robust comparison
            let file_path = std::path::Path::new(f);
            let repo_path = std::path::Path::new(path);
            let rel = file_path.strip_prefix(repo_path).unwrap_or(file_path);
            let rel_normalized: String = rel
                .components()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            let rel_normalized = rel_normalized.trim_start_matches('/');
            unversioned.contains(rel_normalized)
        })
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
