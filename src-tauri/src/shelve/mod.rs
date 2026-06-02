use crate::common::AppError;
use crate::svn::models::ShelveInfo;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

const STALE_LOCK_AGE: Duration = Duration::from_secs(3600);
const RETRY_DELAY: Duration = Duration::from_millis(100);
const MAX_RETRIES: u32 = 3;

struct ShelveLock {
    path: PathBuf,
}

fn clean_stale_lock(lock_path: &std::path::Path) {
    if lock_path.exists() {
        if let Ok(meta) = fs::metadata(lock_path) {
            if let Ok(modified) = meta.modified() {
                if modified.elapsed().unwrap_or_default() > STALE_LOCK_AGE {
                    let _ = fs::remove_file(lock_path);
                }
            }
        }
    }
}

fn try_acquire_lock(lock_path: &std::path::Path) -> Result<(), AppError> {
    match std::fs::File::create_new(lock_path) {
        Ok(_) => Ok(()),
        Err(_) => Err(AppError::Fs(
            "Shelve directory is locked by another operation. Please try again.".to_string(),
        )),
    }
}

impl ShelveLock {
    async fn acquire(dir: &PathBuf) -> Result<Self, AppError> {
        let lock_path = dir.join(".lock");
        clean_stale_lock(&lock_path);
        for attempt in 0..MAX_RETRIES {
            match try_acquire_lock(&lock_path) {
                Ok(_) => return Ok(Self { path: lock_path }),
                Err(e) if attempt + 1 >= MAX_RETRIES => return Err(e),
                Err(_) => tokio::time::sleep(RETRY_DELAY).await,
            }
        }
        unreachable!()
    }

    fn acquire_blocking(dir: &PathBuf) -> Result<Self, AppError> {
        let lock_path = dir.join(".lock");
        clean_stale_lock(&lock_path);
        for attempt in 0..MAX_RETRIES {
            match try_acquire_lock(&lock_path) {
                Ok(_) => return Ok(Self { path: lock_path }),
                Err(e) if attempt + 1 >= MAX_RETRIES => return Err(e),
                Err(_) => std::thread::sleep(RETRY_DELAY),
            }
        }
        unreachable!()
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

/// 保存储藏。
/// - `files`：要储藏的文件路径列表（绝对路径）。若为空则储藏整个工作区。
pub async fn shelve_save(
    repo_path: &str,
    name: &str,
    files: &[String],
    timeout_secs: u64,
) -> Result<(), AppError> {
    let shelve_dir = repo_shelve_dir(repo_path);
    fs::create_dir_all(&shelve_dir)
        .map_err(|e| AppError::Fs(format!("Failed to create shelve directory: {}", e)))?;
    let _lock = ShelveLock::acquire(&shelve_dir).await?;

    let patch_file = shelve_dir.join(format!("{}.patch", name));
    if patch_file.exists() {
        return Err(AppError::Fs(format!(
            "Shelve '{}' already exists",
            name
        )));
    }

    // 构建 svn diff 参数：在仓库目录下运行，文件路径转为相对路径
    let mut args = vec!["diff".to_string()];
    if files.is_empty() {
        // 没有指定文件，diff 整个工作区
        // 不加额外参数，run_svn_async_in_dir 会在 repo_path 下运行
    } else {
        for f in files {
            // svn diff 在工作区目录下运行时接受相对路径或绝对路径
            args.push(f.clone());
        }
    }

    let args_refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let diff = crate::svn::run_svn_async_in_dir(&args_refs, timeout_secs, Some(repo_path))
        .await
        .map_err(|e| AppError::svn_other(format!("Failed to get diff: {}", e)))?;

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

/// 应用储藏（将 patch 打回工作区）。
/// `delete_after_apply`：应用后是否删除该储藏（pop 语义）。
pub async fn shelve_apply(
    repo_path: &str,
    name: &str,
    delete_after_apply: bool,
    timeout_secs: u64,
) -> Result<(), AppError> {
    let shelve_dir = repo_shelve_dir(repo_path);
    let _lock = ShelveLock::acquire(&shelve_dir).await?;
    let patch_file = shelve_dir.join(format!("{}.patch", name));
    if !patch_file.exists() {
        return Err(AppError::Fs(format!("Shelve '{}' not found", name)));
    }

    let patch_path = patch_file
        .to_str()
        .ok_or_else(|| AppError::Fs("Invalid patch file path".to_string()))?
        .to_string();

    crate::svn::run_svn_async_in_dir(&["patch", &patch_path], timeout_secs, Some(repo_path)).await?;

    if delete_after_apply {
        fs::remove_file(&patch_file)
            .map_err(|e| AppError::Fs(format!("Failed to delete shelve after apply: {}", e)))?;
    }

    Ok(())
}

pub fn shelve_delete(repo_path: &str, name: &str) -> Result<(), AppError> {
    let shelve_dir = repo_shelve_dir(repo_path);
    let _lock = ShelveLock::acquire_blocking(&shelve_dir)?;
    let patch_file = shelve_dir.join(format!("{}.patch", name));
    if !patch_file.exists() {
        return Err(AppError::Fs(format!("Shelve '{}' not found", name)));
    }

    fs::remove_file(&patch_file)
        .map_err(|e| AppError::Fs(format!("Failed to delete shelve: {}", e)))?;

    Ok(())
}

/// 重命名储藏（直接重命名 patch 文件，不重新生成 diff）。
pub fn shelve_rename(repo_path: &str, old_name: &str, new_name: &str) -> Result<(), AppError> {
    let shelve_dir = repo_shelve_dir(repo_path);
    let _lock = ShelveLock::acquire_blocking(&shelve_dir)?;

    let old_path = shelve_dir.join(format!("{}.patch", old_name));
    let new_path = shelve_dir.join(format!("{}.patch", new_name));

    if !old_path.exists() {
        return Err(AppError::Fs(format!("Shelve '{}' not found", old_name)));
    }
    if new_path.exists() {
        return Err(AppError::Fs(format!("Shelve '{}' already exists", new_name)));
    }

    fs::rename(&old_path, &new_path)
        .map_err(|e| AppError::Fs(format!("Failed to rename shelve: {}", e)))?;

    Ok(())
}
