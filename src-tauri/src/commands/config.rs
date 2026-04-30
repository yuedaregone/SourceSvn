use crate::common::AppConfig;
use crate::config;

#[tauri::command]
pub fn get_config() -> Result<AppConfig, String> {
    Ok(config::load_config())
}

#[tauri::command]
pub fn set_config(conf: AppConfig) -> Result<(), String> {
    config::save_config(&conf)
}
