use crate::common::AppError;
use crate::svn::models::DiffTarget;

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
    crate::svn::run_svn_async_in_dir(&args_refs, timeout_secs, Some(path)).await
}
