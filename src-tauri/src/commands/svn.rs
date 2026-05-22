use crate::app_state::AppState;
use crate::hook::{HookContext, HookEvent, HookType};
use crate::svn;
use crate::svn::models::{
    BlameEntry, ChangedPath, CommitResult, DiffTarget, DirEntry, FileStatus, LogEntry, RepoInfo,
    WcLogResult,
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
pub async fn svn_log_changed_paths(
    state: State<'_, AppState>,
    path: String,
    revision: u64,
) -> Result<Vec<ChangedPath>, String> {
    let timeout = get_timeout(&state)?;
    svn::log::svn_log_changed_paths(&path, revision, timeout)
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
    let mut context = HookContext::new(HookType::PreCommit, path.clone());
    context = context.with_data("message".to_string(), serde_json::Value::String(message.clone()));
    context = context.with_data("files".to_string(), serde_json::to_value(&files).map_err(|e| e.to_string())?);
    let event = HookEvent::new(HookType::PreCommit, context);
    let results = state.hook_event_bus.emit(event).await;
    for result in results {
        if let Err(e) = result {
            log::warn!("Hook execution failed: {}", e.user_message());
        }
    }

    let timeout = get_timeout(&state)?;
    let result = svn::commit::svn_commit(&path, &message, &files, timeout)
        .await
        .map_err(|e| e.to_string())?;

    let mut context = HookContext::new(HookType::PostCommit, path.clone());
    context = context.with_data("revision".to_string(), serde_json::Value::Number(result.revision.into()));
    let event = HookEvent::new(HookType::PostCommit, context);
    let results = state.hook_event_bus.emit(event).await;
    for result in results {
        if let Err(e) = result {
            log::warn!("Hook execution failed: {}", e.user_message());
        }
    }

    Ok(result)
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
    let context = HookContext::new(HookType::PreUpdate, path.clone());
    let event = HookEvent::new(HookType::PreUpdate, context);
    let results = state.hook_event_bus.emit(event).await;
    for result in results {
        if let Err(e) = result {
            log::warn!("Hook execution failed: {}", e.user_message());
        }
    }

    let timeout = get_timeout(&state)?;
    let revision = svn::update::svn_update_streaming(&path, timeout, &app_handle)
        .await
        .map_err(|e| e.to_string())?;

    let mut context = HookContext::new(HookType::PostUpdate, path.clone());
    context = context.with_data("revision".to_string(), serde_json::Value::Number(revision.into()));
    let event = HookEvent::new(HookType::PostUpdate, context);
    let results = state.hook_event_bus.emit(event).await;
    for result in results {
        if let Err(e) = result {
            log::warn!("Hook execution failed: {}", e.user_message());
        }
    }

    Ok(())
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
pub async fn read_local_file(repo_path: String, file_path: String) -> Result<String, String> {
    svn::diff::read_local_file(&repo_path, &file_path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn svn_cat_at_revision(
    state: State<'_, AppState>,
    repo_path: String,
    file_path: String,
    revision: String,
) -> Result<String, String> {
    let timeout = get_timeout(&state)?;
    svn::cat::svn_cat_at_revision(&repo_path, &file_path, &revision, timeout)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn svn_cat_in_dir(
    state: State<'_, AppState>,
    repo_path: String,
    file_path: String,
    revision: Option<String>,
) -> Result<String, String> {
    let timeout = get_timeout(&state)?;
    svn::cat::svn_cat_in_dir(&repo_path, &file_path, revision.as_deref(), timeout)
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
pub async fn svn_resolve(
    state: State<'_, AppState>,
    path: String,
    paths: Vec<String>,
    accept: String,
) -> Result<Vec<String>, String> {
    let timeout = get_timeout(&state)?;
    svn::ops::svn_resolve(&path, &paths, &accept, timeout)
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
        let result = if canonical.is_dir() {
            std::fs::remove_dir_all(&canonical)
        } else {
            std::fs::remove_file(&canonical)
        };
        match result {
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
    if p.is_dir() {
        #[cfg(target_os = "windows")]
        std::process::Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open: {}", e))?;
        #[cfg(target_os = "macos")]
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open: {}", e))?;
        #[cfg(target_os = "linux")]
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open: {}", e))?;
    } else {
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            let mut cmd = std::process::Command::new("explorer");
            cmd.creation_flags(0x08000000);
            cmd.raw_arg(format!("/select,\"{}\"", path));
            cmd.spawn().map_err(|e| format!("Failed to open: {}", e))?;
        }
        #[cfg(target_os = "macos")]
        std::process::Command::new("open")
            .arg("-R")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open: {}", e))?;
        #[cfg(target_os = "linux")]
        {
            let parent = p.parent().unwrap_or(p);
            std::process::Command::new("xdg-open")
                .arg(parent)
                .spawn()
                .map_err(|e| format!("Failed to open: {}", e))?;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn open_file_with_default_app(path: String) -> Result<(), String> {
    open::that(&path).map_err(|e| format!("Failed to open: {}", e))
}

#[tauri::command]
pub async fn file_size_diff(state: State<'_, AppState>, repo_path: String, file_path: String) -> Result<(u64, u64), String> {
    let timeout = get_timeout(&state)?;
    svn::ops::file_size_diff(&repo_path, &file_path, timeout)
        .await
        .map_err(|e| e.to_string())
}
