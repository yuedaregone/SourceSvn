use crate::common::AppError;
use crate::svn::models::DiffTarget;
use std::path::Path;

pub async fn svn_diff(
    path: &str,
    target: &DiffTarget,
    timeout_secs: u64,
) -> Result<String, AppError> {
    let mut args = vec!["diff".to_string()];

    match target {
        DiffTarget::File {
            path: file_path,
            revision,
        } => {
            args.push(file_path.clone());
            if let Some(rev) = revision {
                args.push("-r".to_string());
                args.push(rev.clone());
            }
        }
        DiffTarget::Revisions { old_rev, new_rev } => {
            args.push("-r".to_string());
            args.push(format!("{}:{}", old_rev, new_rev));
        }
    }

    let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    crate::svn::run_svn_utf8_async_in_dir(&args_refs, timeout_secs, Some(path)).await
}

pub async fn diff_unversioned_file(repo_path: &str, file_path: &str) -> Result<String, AppError> {
    let full_path = Path::new(repo_path).join(file_path);
    let content = std::fs::read_to_string(&full_path).map_err(|e| {
        AppError::Fs(format!(
            "Failed to read file {}: {}",
            full_path.display(),
            e
        ))
    })?;

    let filename = Path::new(file_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| file_path.to_string());

    let mut diff = format!("--- /dev/null\n+++ {}\n", filename);
    if !content.is_empty() {
        let line_count = content.lines().count();
        diff.push_str(&format!("@@ -0,0 +1,{} @@\n", line_count));
        for line in content.lines() {
            diff.push('+');
            diff.push_str(line);
            diff.push('\n');
        }
    }

    Ok(diff)
}
