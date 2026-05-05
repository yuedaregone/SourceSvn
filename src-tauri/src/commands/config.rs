use crate::app_state::AppState;
use crate::common::AppConfig;
use crate::svn;
use tauri::State;

#[tauri::command]
pub fn get_config(state: State<AppState>) -> Result<AppConfig, String> {
    let config = state.config.read().map_err(|e| e.to_string())?;
    Ok(config.clone())
}

#[tauri::command]
pub fn set_config(state: State<AppState>, conf: AppConfig) -> Result<(), String> {
    if let Some(ref exe) = conf.svn.executable {
        if !exe.is_empty() {
            svn::set_svn_path(exe.clone());
        }
    }
    state.update(conf)
}
