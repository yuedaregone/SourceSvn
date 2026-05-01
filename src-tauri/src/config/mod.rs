use crate::common::AppConfig;
use std::path::PathBuf;

const APP_NAME: &str = "sourcesvn";

pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(APP_NAME)
}

pub fn load_config() -> AppConfig {
    let cfg: AppConfig = confy::load(APP_NAME, Some("config")).unwrap_or_else(|e| {
        log::warn!("Failed to load config, using defaults: {}", e);
        AppConfig::default()
    });
    migrate_if_needed(cfg)
}

pub fn save_config(config: &AppConfig) -> Result<(), String> {
    confy::store(APP_NAME, Some("config"), config)
        .map_err(|e| format!("[CFG] Failed to save config: {}", e))
}

fn migrate_if_needed(mut cfg: AppConfig) -> AppConfig {
    if cfg.config_version < 1 {
        cfg.config_version = 1;
        let _ = save_config(&cfg);
    }
    cfg
}
