use crate::common::AppError;

pub async fn svn_checkout(
    url: &str,
    dest: &str,
    timeout_secs: u64,
) -> Result<(), AppError> {
    crate::svn::run_svn_async(&["checkout", url, dest], timeout_secs).await?;
    Ok(())
}
