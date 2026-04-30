use crate::common::{AppError, ShelveInfo};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

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
    let short_hash = &hash[..12];
    shelve_base_dir().join(short_hash)
}

pub fn shelve_save(repo_path: &str, name: &str) -> Result<(), AppError> {
    let shelve_dir = repo_shelve_dir(repo_path);
    fs::create_dir_all(&shelve_dir)
        .map_err(|e| AppError::Fs(format!("Failed to create shelve directory: {}", e)))?;

    let patch_file = shelve_dir.join(format!("{}.patch", name));
    if patch_file.exists() {
        return Err(AppError::Fs(format!("Shelve '{}' already exists", name)));
    }

    let diff = crate::svn::run_svn(&["diff", repo_path], 60)
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

pub fn shelve_apply(repo_path: &str, name: &str) -> Result<(), AppError> {
    let shelve_dir = repo_shelve_dir(repo_path);
    let patch_file = shelve_dir.join(format!("{}.patch", name));
    if !patch_file.exists() {
        return Err(AppError::Fs(format!("Shelve '{}' not found", name)));
    }

    crate::svn::run_svn(
        &["patch", patch_file.to_str().unwrap_or(""), repo_path],
        60,
    )?;

    Ok(())
}

pub fn shelve_delete(repo_path: &str, name: &str) -> Result<(), AppError> {
    let shelve_dir = repo_shelve_dir(repo_path);
    let patch_file = shelve_dir.join(format!("{}.patch", name));
    if !patch_file.exists() {
        return Err(AppError::Fs(format!("Shelve '{}' not found", name)));
    }

    fs::remove_file(&patch_file)
        .map_err(|e| AppError::Fs(format!("Failed to delete shelve: {}", e)))?;

    Ok(())
}
