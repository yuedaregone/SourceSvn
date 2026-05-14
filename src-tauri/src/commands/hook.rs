use std::collections::HashMap;

use tauri::State;

use crate::app_state::AppState;
use crate::hook::{HookHandlerConfig, HookType, HooksConfig};

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
    _state: State<'_, AppState>,
    _hook_type: HookType,
    _repo_path: String,
    _data: HashMap<String, serde_json::Value>,
) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub async fn hook_load_config(
    _state: State<'_, AppState>,
) -> Result<HooksConfig, String> {
    Ok(HooksConfig::default())
}

#[tauri::command]
pub async fn hook_save_config(
    _state: State<'_, AppState>,
    _config: HooksConfig,
) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub async fn hook_add_handler(
    _state: State<'_, AppState>,
    _handler: HookHandlerConfig,
) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub async fn hook_remove_handler(
    _state: State<'_, AppState>,
    _name: String,
) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub async fn hook_update_handler(
    _state: State<'_, AppState>,
    _name: String,
    _handler: HookHandlerConfig,
) -> Result<(), String> {
    Ok(())
}
