use crate::common::AppError;

pub async fn svn_cat(
    path: &str,
    revision: Option<&str>,
    timeout_secs: u64,
) -> Result<String, AppError> {
    let mut args = vec!["cat", path];
    if let Some(rev) = revision {
        args.push("-r");
        args.push(rev);
    }
    crate::svn::run_svn_async(&args, timeout_secs).await
}
