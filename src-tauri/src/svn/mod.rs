pub mod status;
pub mod log;
pub mod diff;
pub mod commit;

use crate::common::AppError;
use std::process::Command;

pub fn run_svn(args: &[&str], timeout_secs: u64) -> Result<String, AppError> {
    let svn_path = find_svn_executable()?;
    let output = Command::new(&svn_path)
        .args(args)
        .output()
        .map_err(|e| AppError::Svn(format!("Failed to execute svn: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Svn(format!("SVN command failed: {}", stderr.trim())));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn find_svn_executable() -> Result<String, AppError> {
    let output = Command::new("where")
        .arg("svn")
        .output()
        .map_err(|e| AppError::Svn(format!("Failed to find svn: {}", e)))?;

    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() {
            return Ok(path);
        }
    }

    Err(AppError::Svn(
        "SVN command line tool not found. Please install SVN client or configure path.".to_string(),
    ))
}
