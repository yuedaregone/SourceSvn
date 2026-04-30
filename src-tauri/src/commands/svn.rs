use crate::common::*;
use crate::config::load_config;
use crate::svn;

#[tauri::command]
pub fn svn_status(path: String) -> Result<Vec<FileStatus>, String> {
    let config = load_config();
    svn::status::svn_status(&path, config.advanced.svn_timeout_secs).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn svn_info(path: String) -> Result<RepoInfo, String> {
    let config = load_config();
    svn::commit::svn_info(&path, config.advanced.svn_timeout_secs).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn svn_log(
    path: String,
    limit: Option<u32>,
    from_rev: Option<String>,
) -> Result<Vec<LogEntry>, String> {
    let config = load_config();
    svn::log::svn_log(&path, limit, from_rev.as_deref(), config.advanced.svn_timeout_secs)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn svn_diff(path: String, target: DiffTarget) -> Result<String, String> {
    let config = load_config();
    svn::diff::svn_diff(&path, &target, config.advanced.svn_timeout_secs)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn svn_commit(
    path: String,
    message: String,
    files: Vec<String>,
) -> Result<CommitResult, String> {
    let config = load_config();
    svn::commit::svn_commit(&path, &message, &files, config.advanced.svn_timeout_secs)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn svn_list(
    path: String,
    revision: Option<String>,
    recursive: bool,
) -> Result<Vec<DirEntry>, String> {
    let config = load_config();
    svn::commit::svn_list(
        &path,
        revision.as_deref(),
        recursive,
        config.advanced.svn_timeout_secs,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn svn_cat(path: String, revision: Option<String>) -> Result<String, String> {
    let config = load_config();
    svn::commit::svn_cat(&path, revision.as_deref(), config.advanced.svn_timeout_secs)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn svn_checkout(url: String, dest: String) -> Result<(), String> {
    let config = load_config();
    svn::commit::svn_checkout(&url, &dest, config.advanced.svn_timeout_secs)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn svn_update(path: String) -> Result<UpdateResult, String> {
    let config = load_config();
    svn::commit::svn_update(&path, config.advanced.svn_timeout_secs).map_err(|e| e.to_string())
}
