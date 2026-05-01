pub mod cat;
pub mod checkout;
pub mod commit;
pub mod diff;
pub mod info;
pub mod list;
pub mod log;
pub mod models;
pub mod status;
pub mod update;

use crate::common::AppError;
use std::time::Duration;
use tokio::process::Command;

fn decode_bytes(bytes: &[u8]) -> String {
    if std::str::from_utf8(bytes).is_ok() {
        return String::from_utf8_lossy(bytes).into_owned();
    }
    #[cfg(target_os = "windows")]
    {
        let (decoded, _, _) = encoding_rs::GBK.decode(bytes);
        return decoded.into_owned();
    }
    #[cfg(not(target_os = "windows"))]
    {
        String::from_utf8_lossy(bytes).into_owned()
    }
}

pub async fn run_svn_async(args: &[&str], timeout_secs: u64) -> Result<String, AppError> {
    run_svn_async_in_dir(args, timeout_secs, None).await
}

pub async fn run_svn_async_in_dir(
    args: &[&str],
    timeout_secs: u64,
    work_dir: Option<&str>,
) -> Result<String, AppError> {
    let svn_path = find_svn_executable()?;

    let mut cmd = Command::new(&svn_path);
    cmd.args(args);
    if let Some(dir) = work_dir {
        cmd.current_dir(dir);
    }

    let output = tokio::time::timeout(Duration::from_secs(timeout_secs), cmd.output())
        .await
        .map_err(|_| {
            AppError::Svn(format!(
                "SVN command timed out after {} seconds",
                timeout_secs
            ))
        })?
        .map_err(|e| AppError::Svn(format!("Failed to execute svn: {}", e)))?;

    if !output.status.success() {
        let stderr = decode_bytes(&output.stderr);
        return Err(AppError::Svn(format!(
            "SVN command failed: {}",
            stderr.trim()
        )));
    }

    Ok(decode_bytes(&output.stdout))
}

pub fn find_svn_executable() -> Result<String, AppError> {
    let find_cmd;
    let find_arg;

    #[cfg(target_os = "windows")]
    {
        find_cmd = "where";
        find_arg = "svn";
    }
    #[cfg(not(target_os = "windows"))]
    {
        find_cmd = "which";
        find_arg = "svn";
    }

    let output = std::process::Command::new(find_cmd)
        .arg(find_arg)
        .output()
        .map_err(|e| AppError::Svn(format!("Failed to find svn: {}", e)))?;

    if output.status.success() {
        let path = decode_bytes(&output.stdout).trim().to_string();
        if !path.is_empty() {
            return Ok(path);
        }
    }

    Err(AppError::Svn(
        "SVN command line tool not found. Please install SVN client or configure path.".to_string(),
    ))
}
