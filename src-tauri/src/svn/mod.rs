pub mod cat;
pub mod checkout;
pub mod commit;
pub mod diff;
pub mod info;
pub mod list;
pub mod log;
pub mod models;
pub mod ops;
pub mod status;
pub mod update;

use crate::common::AppError;
use std::sync::Mutex;
use std::time::Duration;
use tokio::process::Command;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// Decode bytes as UTF-8, falling back to GBK on Windows.
pub fn decode_bytes(bytes: &[u8]) -> String {
    if let Ok(s) = std::str::from_utf8(bytes) {
        if !s.contains('\u{FFFD}') {
            return s.to_owned();
        }
    }
    // UTF-8 解码失败或包含替换字符，尝试 GBK（Windows 中文环境常见）
    let (decoded, _, had_errors) = encoding_rs::GBK.decode(bytes);
    if !had_errors {
        return decoded.into_owned();
    }
    String::from_utf8_lossy(bytes).into_owned()
}

pub async fn run_svn_async(args: &[&str], timeout_secs: u64) -> Result<String, AppError> {
    run_svn_async_in_dir(args, timeout_secs, None).await
}

/// Execute SVN command with UTF-8 output via environment variables.
pub async fn run_svn_async_in_dir(
    args: &[&str],
    timeout_secs: u64,
    work_dir: Option<&str>,
) -> Result<String, AppError> {
    let svn_path =
        tokio::task::spawn_blocking(|| find_svn_executable())
            .await
            .map_err(|e| AppError::Svn(format!("Task join error: {}", e)))??;

    let mut cmd = Command::new(&svn_path);
    cmd.args(args);
    if let Some(dir) = work_dir {
        cmd.current_dir(dir);
    }
    #[cfg(target_os = "windows")]
    cmd.as_std_mut().creation_flags(0x08000000); // CREATE_NO_WINDOW
    cmd.env("OUTPUT_CHARSET", "UTF-8");
    cmd.env("LANG", "en_US.UTF-8");
    cmd.env("LC_ALL", "en_US.UTF-8");
    cmd.env("LC_CTYPE", "en_US.UTF-8");
    run_cmd_output(cmd, timeout_secs).await
}

async fn run_cmd_output(
    mut cmd: Command,
    timeout_secs: u64,
) -> Result<String, AppError> {
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

static SVN_PATH: Mutex<Option<String>> = Mutex::new(None);

pub fn set_svn_path(path: String) {
    if let Ok(mut cached) = SVN_PATH.lock() {
        *cached = Some(path);
    }
}

pub fn find_svn_executable() -> Result<String, AppError> {
    if let Ok(cached) = SVN_PATH.lock() {
        if let Some(ref path) = *cached {
            return Ok(path.clone());
        }
    }

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

    let output = {
        let mut c = std::process::Command::new(find_cmd);
        c.arg(find_arg);
        #[cfg(target_os = "windows")]
        c.creation_flags(0x08000000); // CREATE_NO_WINDOW
        c.output()
    }
        .map_err(|e| AppError::Svn(format!("Failed to find svn: {}", e)))?;

    if output.status.success() {
        let stdout = decode_bytes(&output.stdout);
        // `where` on Windows may return multiple paths (one per line); use the first
        let path = stdout.lines().next().unwrap_or("").trim().to_string();
        if !path.is_empty() {
            if let Ok(mut cached) = SVN_PATH.lock() {
                *cached = Some(path.clone());
            }
            return Ok(path);
        }
    }

    Err(AppError::Svn(
        "SVN command line tool not found. Please install SVN client or configure path.".to_string(),
    ))
}
