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
            .map_err(|e| AppError::svn_io(format!("Task join error: {}", e)))??;

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
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| AppError::svn_spawn(e.to_string()))?;

    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| AppError::svn_io("Failed to capture svn stdout"))?;
    let mut stderr = child
        .stderr
        .take()
        .ok_or_else(|| AppError::svn_io("Failed to capture svn stderr"))?;

    // 并发读取 stdout 和 stderr，避免管道缓冲区满时死锁
    let read_output = async {
        let stdout_handle = tokio::spawn(async move {
            let mut buf = Vec::new();
            tokio::io::AsyncReadExt::read_to_end(&mut stdout, &mut buf).await?;
            Ok::<Vec<u8>, std::io::Error>(buf)
        });
        let stderr_handle = tokio::spawn(async move {
            let mut buf = Vec::new();
            tokio::io::AsyncReadExt::read_to_end(&mut stderr, &mut buf).await?;
            Ok::<Vec<u8>, std::io::Error>(buf)
        });

        let stdout_buf = stdout_handle.await??;
        let stderr_buf = stderr_handle.await??;
        let status = child.wait().await?;
        Ok::<_, std::io::Error>((status, stdout_buf, stderr_buf))
    };

    match tokio::time::timeout(Duration::from_secs(timeout_secs), read_output).await {
        Ok(Ok((status, stdout, stderr))) => {
            if !status.success() {
                let code = status.code().unwrap_or(-1);
                return Err(AppError::svn_exit_code(code, decode_bytes(&stderr)));
            }
            Ok(decode_bytes(&stdout))
        }
        Ok(Err(e)) => Err(AppError::svn_io(e.to_string())),
        Err(_) => {
            let _ = child.kill().await;
            let _ = child.wait().await;
            Err(AppError::svn_timeout(timeout_secs))
        }
    }
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
        .map_err(|e| AppError::svn_io(format!("Failed to find svn: {}", e)))?;

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

    Err(AppError::svn_not_found())
}
