use crate::app_state::AppState;
use crate::svn;
use crate::svn::models::{
    BlameEntry, CommitResult, DiffTarget, DirEntry, FileStatus, LogEntry, RepoInfo, WcLogResult,
};
use tauri::{AppHandle, State};

fn get_timeout(state: &State<'_, AppState>) -> Result<u64, String> {
    let config = state.config.read().map_err(|e| e.to_string())?;
    Ok(config.advanced.svn_timeout_secs)
}

#[tauri::command]
pub async fn svn_status(state: State<'_, AppState>, path: String) -> Result<Vec<FileStatus>, String> {
    let timeout = get_timeout(&state)?;
    svn::status::svn_status(&path, timeout).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn svn_info(state: State<'_, AppState>, path: String) -> Result<RepoInfo, String> {
    let timeout = get_timeout(&state)?;
    svn::info::svn_info(&path, timeout).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn svn_log(
    state: State<'_, AppState>,
    path: String,
    limit: Option<u32>,
    from_rev: Option<String>,
) -> Result<Vec<LogEntry>, String> {
    let timeout = get_timeout(&state)?;
    svn::log::svn_log(&path, limit, from_rev.as_deref(), timeout)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn svn_log_server(
    state: State<'_, AppState>,
    path: String,
    limit: Option<u32>,
) -> Result<WcLogResult, String> {
    let timeout = get_timeout(&state)?;
    svn::log::svn_log_server(&path, limit, timeout)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn svn_diff(
    state: State<'_, AppState>,
    path: String,
    target: DiffTarget,
) -> Result<String, String> {
    let timeout = get_timeout(&state)?;
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
    let timeout = get_timeout(&state)?;
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
    let timeout = get_timeout(&state)?;
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
    let timeout = get_timeout(&state)?;
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
    let timeout = get_timeout(&state)?;
    svn::checkout::svn_checkout(&url, &dest, timeout)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn svn_update(state: State<'_, AppState>, path: String, app_handle: AppHandle) -> Result<(), String> {
    let timeout = get_timeout(&state)?;
    svn::update::svn_update_streaming(&path, timeout, &app_handle)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn svn_cleanup(
    state: State<'_, AppState>,
    path: String,
    options: Option<Vec<String>>,
) -> Result<String, String> {
    let timeout = get_timeout(&state)?;
    const ALLOWED: &[&str] = &[
        "--vacuum-pristines",
        "--vacuum-prunables",
        "--include-externals",
        "--remove-unversioned-trees",
        "--remove-ignored-trees",
        "--drop-dav-cache",
    ];
    let mut args = vec!["cleanup"];
    if let Some(opts) = &options {
        for opt in opts {
            if !ALLOWED.contains(&opt.as_str()) {
                return Err(format!("Invalid cleanup option: {}", opt));
            }
            args.push(opt.as_str());
        }
    }
    svn::run_svn_async_in_dir(&args, timeout, Some(&path))
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

#[tauri::command]
pub async fn svn_revert(
    state: State<'_, AppState>,
    path: String,
    paths: Vec<String>,
) -> Result<Vec<String>, String> {
    let timeout = get_timeout(&state)?;
    svn::ops::svn_revert(&path, &paths, timeout)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn svn_add(
    state: State<'_, AppState>,
    path: String,
    paths: Vec<String>,
) -> Result<Vec<String>, String> {
    let timeout = get_timeout(&state)?;
    svn::ops::svn_add(&path, &paths, timeout)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn svn_delete(
    state: State<'_, AppState>,
    path: String,
    paths: Vec<String>,
    keep_local: bool,
) -> Result<Vec<String>, String> {
    let timeout = get_timeout(&state)?;
    svn::ops::svn_delete(&path, &paths, keep_local, timeout)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn svn_blame(
    state: State<'_, AppState>,
    path: String,
    revision: Option<i32>,
) -> Result<Vec<BlameEntry>, String> {
    let timeout = get_timeout(&state)?;
    svn::ops::svn_blame(&path, revision, timeout)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn svn_update_to_revision(
    state: State<'_, AppState>,
    path: String,
    revision: i32,
) -> Result<String, String> {
    let timeout = get_timeout(&state)?;
    svn::ops::svn_update_to_revision(&path, revision, timeout)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_files_from_disk(path: String, paths: Vec<String>) -> Result<Vec<String>, String> {
    let base = std::fs::canonicalize(&path).map_err(|e| format!("Invalid repo path: {}", e))?;
    let mut deleted = Vec::new();
    for p in &paths {
        let full_path = std::path::Path::new(&path).join(p);
        let canonical = match std::fs::canonicalize(&full_path) {
            Ok(c) => c,
            Err(e) => { log::warn!("Failed to canonicalize {}: {}", full_path.display(), e); continue; }
        };
        if !canonical.starts_with(&base) {
            return Err(format!("Path traversal rejected: {}", p));
        }
        match std::fs::remove_file(&canonical) {
            Ok(()) => deleted.push(p.clone()),
            Err(e) => log::warn!("Failed to delete {}: {}", canonical.display(), e),
        }
    }
    Ok(deleted)
}

#[tauri::command]
pub fn find_svn_root(path: String) -> Result<String, String> {
    svn::ops::find_svn_root(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_in_system(path: String) -> Result<(), String> {
    let p = std::path::Path::new(&path);
    let cmd = if p.is_dir() {
        #[cfg(target_os = "windows")]
        { ("explorer", vec![path]) }
        #[cfg(target_os = "macos")]
        { ("open", vec![path]) }
        #[cfg(target_os = "linux")]
        { ("xdg-open", vec![path]) }
    } else {
        #[cfg(target_os = "windows")]
        { ("explorer", vec![format!("/select,{}", path)]) }
        #[cfg(target_os = "macos")]
        { ("open", vec!["-R".to_string(), path]) }
        #[cfg(target_os = "linux")]
        {
            let parent = p.parent().unwrap_or(p);
            ("xdg-open", vec![parent.to_string_lossy().into_owned()])
        }
    };
    std::process::Command::new(cmd.0)
        .args(&cmd.1)
        .spawn()
        .map_err(|e| format!("Failed to open: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn open_file_with_default_app(path: String) -> Result<(), String> {
    open::that(&path).map_err(|e| format!("Failed to open: {}", e))
}
