use crate::common::AppError;
use crate::svn::models::DiffTarget;
use std::path::Path;

pub async fn svn_diff(
    path: &str,
    target: &DiffTarget,
    timeout_secs: u64,
) -> Result<String, AppError> {
    match target {
        DiffTarget::File {
            path: file_path,
            revision,
        } => {
            // Use working directory mode to avoid encoding issues with Chinese paths on Windows.
            // Run svn diff from the repo root and pass relative path.
            let mut args = vec!["diff".to_string(), file_path.to_string()];
            log::debug!("args: {:?}", args);
            
            if let Some(rev) = revision {
                args.push("-r".to_string());
                args.push(rev.clone());
                log::debug!("args with revision: {:?}", args);
            }
            
            let args_refs: Vec<&str> = args.iter().map(String::as_str).collect();
            log::debug!("args_refs: {:?}", args_refs);
            
            crate::svn::run_svn_async_in_dir(&args_refs, timeout_secs, Some(path)).await
        }
        DiffTarget::FileAtRevision {
            path: file_path,
            base_revision,
            revision,
        } => {
            let info_xml =
                crate::svn::run_svn_async(&["info", "--xml", path], timeout_secs).await?;
            let info = crate::svn::info::parse_info_for_log(&info_xml)?;
            let base_url = if info.root.is_empty() { &info.url } else { &info.root };
            let file_url = if base_url.ends_with('/') {
                format!("{}{}", base_url, file_path)
            } else {
                format!("{}/{}", base_url, file_path)
            };
            let rev_range = format!("{}:{}", base_revision, revision);
            let args = vec!["diff", "-r", &rev_range, &file_url];
            crate::svn::run_svn_async(&args, timeout_secs).await
        }
        DiffTarget::Revisions { old_rev, new_rev } => {
            let rev_range = format!("{}:{}", old_rev, new_rev);
            let args = vec!["diff", "-r", &rev_range];
            crate::svn::run_svn_async_in_dir(&args, timeout_secs, Some(path)).await
        }
    }
}

/// 读取本地文件内容（复用 diff_unversioned_file 的读文件+解码逻辑）
pub async fn read_local_file_content(repo_path: &str, file_path: &str) -> Result<String, AppError> {
    let full_path = Path::new(repo_path).join(file_path);
    let bytes = tokio::fs::read(&full_path).await.map_err(|e| {
        AppError::Fs(format!(
            "Failed to read file {}: {}",
            full_path.display(),
            e
        ))
    })?;
    Ok(crate::svn::decode_bytes(&bytes))
}

pub async fn read_local_file(repo_path: &str, file_path: &str) -> Result<String, AppError> {
    read_local_file_content(repo_path, file_path).await
}

pub async fn diff_unversioned_file(repo_path: &str, file_path: &str) -> Result<String, AppError> {
    let content = read_local_file_content(repo_path, file_path).await?;

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
