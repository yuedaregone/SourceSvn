use crate::app_state::AppState;
use crate::common::AppConfig;
use tauri::State;

#[tauri::command]
pub fn get_config(state: State<AppState>) -> Result<AppConfig, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    Ok(config.clone())
}

#[tauri::command]
pub fn set_config(state: State<AppState>, conf: AppConfig) -> Result<(), String> {
    state.update(conf)
}
