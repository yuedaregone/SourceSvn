use crate::common::AppConfig;
use crate::config::{load_config, save_config};
use crate::hook::{DefaultEventBus, EventBus, FileHookConfigManager, FileLogger, ScriptExecutorManager};
use std::sync::{Arc, OnceLock, RwLock};

pub struct AppState {
    pub config: RwLock<AppConfig>,
    http_client: OnceLock<reqwest::Client>,
    pub hook_event_bus: Arc<dyn EventBus>,
    pub hook_config_manager: Arc<FileHookConfigManager>,
    pub hook_script_executor: Arc<ScriptExecutorManager>,
}

impl AppState {
    pub fn new(app_handle: Option<tauri::AppHandle>) -> Self {
        let logger = Arc::new(FileLogger::new(FileLogger::default_path()));
        let event_bus = Arc::new(DefaultEventBus::new(logger));
        let config_manager = Arc::new(FileHookConfigManager::new(FileHookConfigManager::default_path()));
        let script_executor = Arc::new(ScriptExecutorManager::new(app_handle));

        Self {
            config: RwLock::new(load_config()),
            http_client: OnceLock::new(),
            hook_event_bus: event_bus,
            hook_config_manager: config_manager,
            hook_script_executor: script_executor,
        }
    }

    pub fn http_client(&self) -> &reqwest::Client {
        self.http_client.get_or_init(reqwest::Client::new)
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
