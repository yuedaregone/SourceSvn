use crate::common::AppConfig;
use crate::config::{load_config, save_config};
use std::sync::RwLock;

pub struct AppState {
    pub config: RwLock<AppConfig>,
    pub http_client: reqwest::Client,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            config: RwLock::new(load_config()),
            http_client: reqwest::Client::new(),
        }
    }

    pub fn update(&self, new_config: AppConfig) -> Result<(), String> {
        let mut config = self
            .config
            .write()
            .map_err(|e| format!("Lock poisoned: {}", e))?;
        save_config(&new_config)?;
        *config = new_config;
        Ok(())
    }
}
