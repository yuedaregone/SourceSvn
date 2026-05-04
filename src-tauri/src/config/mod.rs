use crate::common::AppConfig;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

const APP_NAME: &str = "sourcesvn";

fn config_file_path() -> PathBuf {
    let base = dirs::config_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| {
            log::warn!("No config or home directory found, using current directory");
            PathBuf::from(".")
        });
    base.join(APP_NAME).join("config.toml")
}

pub fn config_dir() -> PathBuf {
    config_file_path()
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn load_config() -> AppConfig {
    let path = config_file_path();

    if let Ok(content) = fs::read_to_string(&path) {
        match toml::from_str(&content) {
            Ok(cfg) => return migrate_if_needed(cfg),
            Err(e) => {
                log::warn!("Failed to parse config file: {}, using defaults", e);
            }
        }
    } else {
        log::debug!("Config file not found, using defaults");
    }

    AppConfig::default()
}

pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let toml_str = toml::to_string_pretty(config)
        .map_err(|e| format!("TOML serialization failed: {}", e))?;

    let path = config_file_path();
    let tmp_path = path.with_extension("toml.tmp");

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    // Write to temp file first, then rename for crash safety
    File::create(&tmp_path)
        .and_then(|mut file| {
            file.write_all(toml_str.as_bytes())?;
            file.sync_all()
        })
        .map_err(|e| format!("Failed to write config temp file: {}", e))?;

    // Windows 10 1903+ supports atomic rename-over-existing
    fs::rename(&tmp_path, &path)
        .map_err(|e| format!("Failed to rename config file: {}", e))
}

fn migrate_if_needed(mut cfg: AppConfig) -> AppConfig {
    if cfg.config_version < 1 {
        cfg.config_version = 1;
        if let Err(e) = save_config(&cfg) {
            log::warn!("Failed to save migrated config: {}", e);
        }
    }
    cfg
}