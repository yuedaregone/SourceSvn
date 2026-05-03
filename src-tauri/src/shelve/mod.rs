use crate::common::AppError;
use crate::svn::models::ShelveInfo;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

const STALE_LOCK_AGE: Duration = Duration::from_secs(3600);

struct ShelveLock {
    path: PathBuf,
}

impl ShelveLock {
    fn acquire(dir: &PathBuf) -> Result<Self, AppError> {
        // Clean stale locks (older than 1 hour, e.g. from a previous crash)
        let lock_path = dir.join(".lock");
        if lock_path.exists() {
            if let Ok(meta) = fs::metadata(&lock_path) {
                if let Ok(modified) = meta.modified() {
                    if modified.elapsed().unwrap_or_default() > STALE_LOCK_AGE {
                        let _ = fs::remove_file(&lock_path);
                    }
                }
            }
        }

        // Atomic lock creation — fails if already exists
        std::fs::File::create_new(&lock_path)
            .map_err(|_| AppError::Fs(
                "Shelve directory is locked by another operation. Please try again.".to_string()
            ))?;

        Ok(Self { path: lock_path })
    }
}

impl Drop for ShelveLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

pub fn validate_shelve_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Shelve name cannot be empty".to_string());
    }
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err("Invalid shelve name: contains illegal characters".to_string());
    }
    if name.len() > 128 {
        return Err("Shelve name is too long (max 128 characters)".to_string());
    }
    Ok(())
}

fn shelve_base_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("sourcesvn")
        .join("shelves")
}

fn repo_shelve_dir(repo_path: &str) -> PathBuf {
    let mut hasher = Sha256::new();
    hasher.update(repo_path.as_bytes());
    let hash = format!("{:x}", hasher.finalize());
    let short_hash = &hash[..16];
    shelve_base_dir().join(short_hash)
}

pub async fn shelve_save(
    repo_path: &str,
    name: &str,
    timeout_secs: u64,
) -> Result<(), AppError> {
    let shelve_dir = repo_shelve_dir(repo_path);
    fs::create_dir_all(&shelve_dir)
        .map_err(|e| AppError::Fs(format!("Failed to create shelve directory: {}", e)))?;
    let _lock = ShelveLock::acquire(&shelve_dir)?;

    let patch_file = shelve_dir.join(format!("{}.patch", name));
    if patch_file.exists() {
        return Err(AppError::Fs(format!(
            "Shelve '{}' already exists",
            name
        )));
    }

    let diff = crate::svn::run_svn_utf8_async(&["diff", repo_path], timeout_secs)
        .await
        .map_err(|e| AppError::Svn(format!("Failed to get diff: {}", e)))?;

    if diff.trim().is_empty() {
        return Err(AppError::Fs("No changes to shelve".to_string()));
    }

    fs::write(&patch_file, &diff)
        .map_err(|e| AppError::Fs(format!("Failed to write patch file: {}", e)))?;

    Ok(())
}

pub fn shelve_list(repo_path: &str) -> Result<Vec<ShelveInfo>, AppError> {
    let shelve_dir = repo_shelve_dir(repo_path);
    if !shelve_dir.exists() {
        return Ok(vec![]);
    }

    let entries = fs::read_dir(&shelve_dir)
        .map_err(|e| AppError::Fs(format!("Failed to read shelve directory: {}", e)))?;

    let mut shelves = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| AppError::Fs(format!("Failed to read entry: {}", e)))?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("patch") {
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();
            let date = fs::metadata(&path)
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| {
                    let datetime: chrono::DateTime<chrono::Utc> = t.into();
                    Some(datetime.to_rfc3339())
                })
                .unwrap_or_default();
            shelves.push(ShelveInfo { name, date });
        }
    }

    shelves.sort_by(|a, b| b.date.cmp(&a.date));
    Ok(shelves)
}

pub async fn shelve_apply(
    repo_path: &str,
    name: &str,
    timeout_secs: u64,
) -> Result<(), AppError> {
    let shelve_dir = repo_shelve_dir(repo_path);
    let _lock = ShelveLock::acquire(&shelve_dir)?;
    let patch_file = shelve_dir.join(format!("{}.patch", name));
    if !patch_file.exists() {
        return Err(AppError::Fs(format!("Shelve '{}' not found", name)));
    }

    let patch_path = patch_file.to_str().unwrap_or("");
    crate::svn::run_svn_async(&["patch", patch_path, repo_path], timeout_secs).await?;

    Ok(())
}

pub fn shelve_delete(repo_path: &str, name: &str) -> Result<(), AppError> {
    let shelve_dir = repo_shelve_dir(repo_path);
    let _lock = ShelveLock::acquire(&shelve_dir)?;
    let patch_file = shelve_dir.join(format!("{}.patch", name));
    if !patch_file.exists() {
        return Err(AppError::Fs(format!("Shelve '{}' not found", name)));
    }

    fs::remove_file(&patch_file)
        .map_err(|e| AppError::Fs(format!("Failed to delete shelve: {}", e)))?;

    Ok(())
}
