use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::types::HookType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HooksConfig {
    pub enabled: bool,
    pub handlers: Vec<HookHandlerConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookHandlerConfig {
    pub name: String,
    pub hook_type: HookType,
    pub script_path: String,
    pub enabled: bool,
}

impl Default for HooksConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            handlers: Vec::new(),
        }
    }
}

pub trait HookConfigManager: Send + Sync {
    fn load_config(&self) -> Result<HooksConfig, Box<dyn std::error::Error>>;
    fn save_config(&self, config: &HooksConfig) -> Result<(), Box<dyn std::error::Error>>;
    fn add_handler(&self, config: HookHandlerConfig) -> Result<(), Box<dyn std::error::Error>>;
    fn remove_handler(&self, name: &str) -> Result<(), Box<dyn std::error::Error>>;
    fn update_handler(
        &self,
        name: &str,
        config: HookHandlerConfig,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct FileHookConfigManager {
    config_path: PathBuf,
}

impl FileHookConfigManager {
    pub fn new(config_path: PathBuf) -> Self {
        Self { config_path }
    }

    pub fn default_path() -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(".sourcesvn").join("hooks.toml")
    }
}

impl HookConfigManager for FileHookConfigManager {
    fn load_config(&self) -> Result<HooksConfig, Box<dyn std::error::Error>> {
        if !self.config_path.exists() {
            return Ok(HooksConfig::default());
        }
        let content = std::fs::read_to_string(&self.config_path)?;
        let config: HooksConfig = toml::from_str(&content)?;
        Ok(config)
    }

    fn save_config(&self, config: &HooksConfig) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(config)?;
        std::fs::write(&self.config_path, content)?;
        Ok(())
    }

    fn add_handler(&self, config: HookHandlerConfig) -> Result<(), Box<dyn std::error::Error>> {
        let mut hooks_config = self.load_config()?;
        hooks_config.handlers.push(config);
        self.save_config(&hooks_config)
    }

    fn remove_handler(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut hooks_config = self.load_config()?;
        hooks_config.handlers.retain(|h| h.name != name);
        self.save_config(&hooks_config)
    }

    fn update_handler(
        &self,
        name: &str,
        config: HookHandlerConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut hooks_config = self.load_config()?;
        if let Some(handler) = hooks_config.handlers.iter_mut().find(|h| h.name == name) {
            *handler = config;
        }
        self.save_config(&hooks_config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hooks_config_default() {
        let config = HooksConfig::default();
        assert!(config.enabled);
        assert!(config.handlers.is_empty());
    }

    #[test]
    fn test_file_config_manager_load_nonexistent() {
        let manager = FileHookConfigManager::new(PathBuf::from("/tmp/nonexistent_hooks.toml"));
        let config = manager.load_config().unwrap();
        assert!(config.enabled);
        assert!(config.handlers.is_empty());
    }

    #[test]
    fn test_file_config_manager_save_and_load() {
        let dir = std::env::temp_dir().join("sourcesvn_hook_test");
        let _ = std::fs::remove_dir_all(&dir);

        let manager = FileHookConfigManager::new(dir.join("hooks.toml"));

        let config = HooksConfig {
            enabled: false,
            handlers: vec![HookHandlerConfig {
                name: "test".to_string(),
                hook_type: HookType::PreCommit,
                script_path: "/path/to/script.sh".to_string(),
                enabled: true,
            }],
        };

        manager.save_config(&config).unwrap();
        let loaded = manager.load_config().unwrap();

        assert!(!loaded.enabled);
        assert_eq!(loaded.handlers.len(), 1);
        assert_eq!(loaded.handlers[0].name, "test");
        assert_eq!(loaded.handlers[0].hook_type, HookType::PreCommit);
        assert_eq!(loaded.handlers[0].script_path, "/path/to/script.sh");
        assert!(loaded.handlers[0].enabled);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_file_config_manager_add_handler() {
        let dir = std::env::temp_dir().join("sourcesvn_hook_add_test");
        let _ = std::fs::remove_dir_all(&dir);

        let manager = FileHookConfigManager::new(dir.join("hooks.toml"));

        manager
            .add_handler(HookHandlerConfig {
                name: "handler1".to_string(),
                hook_type: HookType::PreCommit,
                script_path: "/a.sh".to_string(),
                enabled: true,
            })
            .unwrap();

        manager
            .add_handler(HookHandlerConfig {
                name: "handler2".to_string(),
                hook_type: HookType::PostCommit,
                script_path: "/b.sh".to_string(),
                enabled: false,
            })
            .unwrap();

        let config = manager.load_config().unwrap();
        assert_eq!(config.handlers.len(), 2);
        assert_eq!(config.handlers[0].name, "handler1");
        assert_eq!(config.handlers[1].name, "handler2");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_file_config_manager_remove_handler() {
        let dir = std::env::temp_dir().join("sourcesvn_hook_remove_test");
        let _ = std::fs::remove_dir_all(&dir);

        let manager = FileHookConfigManager::new(dir.join("hooks.toml"));

        manager
            .add_handler(HookHandlerConfig {
                name: "keep".to_string(),
                hook_type: HookType::PreCommit,
                script_path: "/a.sh".to_string(),
                enabled: true,
            })
            .unwrap();

        manager
            .add_handler(HookHandlerConfig {
                name: "remove_me".to_string(),
                hook_type: HookType::PostCommit,
                script_path: "/b.sh".to_string(),
                enabled: true,
            })
            .unwrap();

        manager.remove_handler("remove_me").unwrap();

        let config = manager.load_config().unwrap();
        assert_eq!(config.handlers.len(), 1);
        assert_eq!(config.handlers[0].name, "keep");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_file_config_manager_update_handler() {
        let dir = std::env::temp_dir().join("sourcesvn_hook_update_test");
        let _ = std::fs::remove_dir_all(&dir);

        let manager = FileHookConfigManager::new(dir.join("hooks.toml"));

        manager
            .add_handler(HookHandlerConfig {
                name: "handler1".to_string(),
                hook_type: HookType::PreCommit,
                script_path: "/old.sh".to_string(),
                enabled: true,
            })
            .unwrap();

        manager
            .update_handler(
                "handler1",
                HookHandlerConfig {
                    name: "handler1".to_string(),
                    hook_type: HookType::PostCommit,
                    script_path: "/new.sh".to_string(),
                    enabled: false,
                },
            )
            .unwrap();

        let config = manager.load_config().unwrap();
        assert_eq!(config.handlers.len(), 1);
        assert_eq!(config.handlers[0].hook_type, HookType::PostCommit);
        assert_eq!(config.handlers[0].script_path, "/new.sh");
        assert!(!config.handlers[0].enabled);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_file_config_manager_remove_nonexistent() {
        let dir = std::env::temp_dir().join("sourcesvn_hook_remove_nonexist_test");
        let _ = std::fs::remove_dir_all(&dir);

        let manager = FileHookConfigManager::new(dir.join("hooks.toml"));
        manager.remove_handler("does_not_exist").unwrap();

        let config = manager.load_config().unwrap();
        assert!(config.handlers.is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_file_config_manager_update_nonexistent() {
        let dir = std::env::temp_dir().join("sourcesvn_hook_update_nonexist_test");
        let _ = std::fs::remove_dir_all(&dir);

        let manager = FileHookConfigManager::new(dir.join("hooks.toml"));
        manager
            .update_handler(
                "does_not_exist",
                HookHandlerConfig {
                    name: "does_not_exist".to_string(),
                    hook_type: HookType::PreCommit,
                    script_path: "/x.sh".to_string(),
                    enabled: true,
                },
            )
            .unwrap();

        let config = manager.load_config().unwrap();
        assert!(config.handlers.is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_default_path() {
        let path = FileHookConfigManager::default_path();
        assert!(path.ends_with(".sourcesvn/hooks.toml"));
    }
}
