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

/// Decode bytes from SVN output, handling GBK encoding on Windows.
///
/// SVN on Chinese Windows may output GBK-encoded bytes even with --xml.
/// This function tries UTF-8 first, then falls back to GBK on Windows.
fn decode_bytes(bytes: &[u8]) -> String {
    // Try UTF-8 first (works for well-configured SVN or --xml on modern SVN)
    if let Ok(s) = std::str::from_utf8(bytes) {
        // Check for replacement characters — indicates broken UTF-8
        // that from_utf8_lossy silently repaired
        if !s.contains('\u{FFFD}') {
            return s.to_owned();
        }
    }
    // UTF-8 failed or contained replacement chars — try GBK on Windows
    #[cfg(target_os = "windows")]
    {
        let (decoded, _, had_errors) = encoding_rs::GBK.decode(bytes);
        if !had_errors {
            return decoded.into_owned();
        }
        // GBK also had errors — return best effort
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

    #[cfg(target_os = "windows")]
    let mut cmd = {
        // Force UTF-8 output via cmd.exe + chcp 65001
        let mut shell = Command::new("cmd.exe");
        shell.arg("/C").arg("chcp").arg("65001").arg(">nul").arg("&&");
        shell.arg(&svn_path);
        shell.args(args);
        shell
    };
    #[cfg(not(target_os = "windows"))]
    let mut cmd = {
        let mut c = Command::new(&svn_path);
        c.args(args);
        c
    };

    if let Some(dir) = work_dir {
        cmd.current_dir(dir);
    }
    // Ensure subprocess inherits UTF-8 locale hints
    cmd.env("OUTPUT_CHARSET", "UTF-8");
    cmd.env("LANG", "en_US.UTF-8");

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
