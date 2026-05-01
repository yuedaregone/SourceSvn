use crate::common::AppConfig;
use crate::config::{load_config, save_config};
use std::sync::Mutex;

pub struct AppState {
    pub config: Mutex<AppConfig>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            config: Mutex::new(load_config()),
        }
    }

    pub fn update(&self, new_config: AppConfig) -> Result<(), String> {
        save_config(&new_config)?;
        let mut config = self
            .config
            .lock()
            .map_err(|e| format!("Lock poisoned: {}", e))?;
        *config = new_config;
        Ok(())
    }
}
