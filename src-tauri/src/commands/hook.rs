use tauri::State;

use crate::app_state::AppState;
use crate::hook::{HookConfigManager, HookContext, HookHandlerConfig, HookResult, HookType, HooksConfig};

#[tauri::command]
pub async fn hook_subscribe(
    _state: State<'_, AppState>,
    _hook_type: HookType,
    _handler_name: String,
    _script_path: String,
) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub async fn hook_unsubscribe(
    _state: State<'_, AppState>,
    _hook_type: HookType,
    _handler_name: String,
) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub async fn hook_emit(
    state: State<'_, AppState>,
    script_path: String,
    hook_type: HookType,
    repo_path: String,
) -> Result<HookResult, String> {
    let context = HookContext::new(hook_type, repo_path);
    state
        .hook_script_executor
        .execute(&script_path, &context)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn hook_load_config(
    state: State<'_, AppState>,
) -> Result<HooksConfig, String> {
    state
        .hook_config_manager
        .load_config()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn hook_save_config(
    state: State<'_, AppState>,
    config: HooksConfig,
) -> Result<(), String> {
    state
        .hook_config_manager
        .save_config(&config)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn hook_add_handler(
    state: State<'_, AppState>,
    handler: HookHandlerConfig,
) -> Result<(), String> {
    state
        .hook_config_manager
        .add_handler(handler)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn hook_remove_handler(
    state: State<'_, AppState>,
    name: String,
) -> Result<(), String> {
    state
        .hook_config_manager
        .remove_handler(&name)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn hook_update_handler(
    state: State<'_, AppState>,
    name: String,
    handler: HookHandlerConfig,
) -> Result<(), String> {
    state
        .hook_config_manager
        .update_handler(&name, handler)
        .map_err(|e| e.to_string())
}
