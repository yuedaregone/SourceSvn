use crate::app_state::AppState;
use crate::svn;
use crate::svn::models::{
    CommitResult, DiffTarget, DirEntry, FileStatus, LogEntry, RepoInfo, UpdateResult,
};
use tauri::State;

#[tauri::command]
pub async fn svn_status(state: State<'_, AppState>, path: String) -> Result<Vec<FileStatus>, String> {
    let timeout = {
        let config = state.config.lock().map_err(|e| e.to_string())?;
        config.advanced.svn_timeout_secs
    };
    svn::status::svn_status(&path, timeout).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn svn_info(state: State<'_, AppState>, path: String) -> Result<RepoInfo, String> {
    let timeout = {
        let config = state.config.lock().map_err(|e| e.to_string())?;
        config.advanced.svn_timeout_secs
    };
    svn::info::svn_info(&path, timeout).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn svn_log(
    state: State<'_, AppState>,
    path: String,
    limit: Option<u32>,
    from_rev: Option<String>,
) -> Result<Vec<LogEntry>, String> {
    let timeout = {
        let config = state.config.lock().map_err(|e| e.to_string())?;
        config.advanced.svn_timeout_secs
    };
    svn::log::svn_log(&path, limit, from_rev.as_deref(), timeout)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn svn_diff(
    state: State<'_, AppState>,
    path: String,
    target: DiffTarget,
) -> Result<String, String> {
    let timeout = {
        let config = state.config.lock().map_err(|e| e.to_string())?;
        config.advanced.svn_timeout_secs
    };
    svn::diff::svn_diff(&path, &target, timeout)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn svn_commit(
    state: State<'_, AppState>,
    path: String,
    message: String,
    files: Vec<String>,
) -> Result<CommitResult, String> {
    let timeout = {
        let config = state.config.lock().map_err(|e| e.to_string())?;
        config.advanced.svn_timeout_secs
    };
    svn::commit::svn_commit(&path, &message, &files, timeout)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn svn_list(
    state: State<'_, AppState>,
    path: String,
    revision: Option<String>,
    recursive: bool,
) -> Result<Vec<DirEntry>, String> {
    let timeout = {
        let config = state.config.lock().map_err(|e| e.to_string())?;
        config.advanced.svn_timeout_secs
    };
    svn::list::svn_list(&path, revision.as_deref(), recursive, timeout)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn svn_cat(
    state: State<'_, AppState>,
    path: String,
    revision: Option<String>,
) -> Result<String, String> {
    let timeout = {
        let config = state.config.lock().map_err(|e| e.to_string())?;
        config.advanced.svn_timeout_secs
    };
    svn::cat::svn_cat(&path, revision.as_deref(), timeout)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn svn_checkout(
    state: State<'_, AppState>,
    url: String,
    dest: String,
) -> Result<(), String> {
    let timeout = {
        let config = state.config.lock().map_err(|e| e.to_string())?;
        config.advanced.svn_timeout_secs
    };
    svn::checkout::svn_checkout(&url, &dest, timeout)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn svn_update(state: State<'_, AppState>, path: String) -> Result<UpdateResult, String> {
    let timeout = {
        let config = state.config.lock().map_err(|e| e.to_string())?;
        config.advanced.svn_timeout_secs
    };
    svn::update::svn_update(&path, timeout)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn svn_detect_executable() -> Result<String, String> {
    svn::find_svn_executable().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn diff_unversioned_file(repo_path: String, file_path: String) -> Result<String, String> {
    svn::diff::diff_unversioned_file(&repo_path, &file_path)
        .await
        .map_err(|e| e.to_string())
}
