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
use std::sync::OnceLock;
use std::time::Duration;
use tokio::process::Command;

/// Decode bytes from SVN output, handling GBK encoding on Windows.
///
/// SVN on Chinese Windows may output GBK-encoded bytes even with --xml.
/// This function tries UTF-8 first, then falls back to GBK on Windows.
pub fn decode_bytes(bytes: &[u8]) -> String {
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

    #[cfg(target_os = "windows")]
    let old_cp = unsafe { windows_sys::Win32::System::Console::GetConsoleCP() };

    #[cfg(target_os = "windows")]
    if old_cp != 936 {
        unsafe { windows_sys::Win32::System::Console::SetConsoleCP(936) };
        unsafe { windows_sys::Win32::System::Console::SetConsoleOutputCP(936) };
    }

    let mut cmd = Command::new(&svn_path);
    cmd.args(args);
    if let Some(dir) = work_dir {
        cmd.current_dir(dir);
    }
    cmd.env("OUTPUT_CHARSET", "UTF-8");
    cmd.env("LANG", "en_US.UTF-8");
    cmd.env("LC_ALL", "en_US.UTF-8");
    cmd.env("LC_CTYPE", "en_US.UTF-8");
    let result = run_cmd_output(cmd, timeout_secs).await;

    #[cfg(target_os = "windows")]
    {
        unsafe { windows_sys::Win32::System::Console::SetConsoleCP(old_cp) };
        unsafe { windows_sys::Win32::System::Console::SetConsoleOutputCP(old_cp) };
    }

    result
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

static SVN_PATH: OnceLock<String> = OnceLock::new();

pub fn find_svn_executable() -> Result<String, AppError> {
    if let Some(path) = SVN_PATH.get() {
        return Ok(path.clone());
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

    let output = std::process::Command::new(find_cmd)
        .arg(find_arg)
        .output()
        .map_err(|e| AppError::Svn(format!("Failed to find svn: {}", e)))?;

    if output.status.success() {
        let stdout = decode_bytes(&output.stdout);
        // `where` on Windows may return multiple paths (one per line); use the first
        let path = stdout.lines().next().unwrap_or("").trim().to_string();
        if !path.is_empty() {
            let _ = SVN_PATH.set(path.clone());
            return Ok(path);
        }
    }

    Err(AppError::Svn(
        "SVN command line tool not found. Please install SVN client or configure path.".to_string(),
    ))
}
