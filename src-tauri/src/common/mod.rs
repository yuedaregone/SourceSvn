mod error;
pub use error::AppError;

use serde::{Deserialize, Serialize};

// ---- Config Types ----

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub config_version: u32,
    pub window: WindowConfig,
    pub appearance: AppearanceConfig,
    pub session: SessionConfig,
    pub svn: SvnConfig,
    pub ai: AiConfig,
    pub diff: DiffConfig,
    pub log: LogConfig,
    pub commit: CommitConfig,
    pub file_browser: FileBrowserConfig,
    pub behavior: BehaviorConfig,
    pub advanced: AdvancedConfig,
    #[serde(default)]
    pub cleanup: CleanupConfig,
    #[serde(default)]
    pub external_editor: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WindowConfig {
    pub width: u32,
    pub height: u32,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub maximized: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppearanceConfig {
    pub theme: String,
    pub ui_font_family: String,
    pub ui_font_size: u32,
    pub code_font_family: String,
    pub code_font_size: u32,
    pub icon_size: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SessionConfig {
    pub open_tabs: Vec<TabInfo>,
    pub active_tab_index: u64,
    pub recent_repos: Vec<RepoEntry>,
    pub max_recent_repos: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TabInfo {
    pub repo_path: String,
    pub active_view: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RepoEntry {
    pub path: String,
    pub last_opened: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SvnConfig {
    pub executable: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AiConfig {
    pub provider: String,
    pub endpoint: String,
    pub api_key: String,
    pub model: String,
    pub timeout_secs: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DiffConfig {
    pub context_lines: u32,
    pub ignore_whitespace: bool,
    pub view_mode: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LogConfig {
    pub fetch_limit: u32,
    pub show_changed_paths: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CommitConfig {
    pub template: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileBrowserConfig {
    pub show_hidden: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BehaviorConfig {
    pub confirm_before_commit: bool,
    pub confirm_before_revert: bool,
    pub auto_refresh_secs: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AdvancedConfig {
    pub svn_timeout_secs: u64,
    pub log_level: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct CleanupConfig {
    pub vacuum_pristines: bool,
    pub vacuum_prunables: bool,
    pub include_externals: bool,
    pub remove_unversioned_trees: bool,
    pub remove_ignored_trees: bool,
    pub drop_dav_cache: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            config_version: 1,
            window: WindowConfig {
                width: 1200,
                height: 800,
                x: None,
                y: None,
                maximized: false,
            },
            appearance: AppearanceConfig {
                theme: "light".to_string(),
                ui_font_family: "sans-serif".to_string(),
                ui_font_size: 14,
                code_font_family: "monospace".to_string(),
                code_font_size: 13,
                icon_size: 20,
            },
            session: SessionConfig {
                open_tabs: vec![],
                active_tab_index: 0,
                recent_repos: vec![],
                max_recent_repos: 10,
            },
            svn: SvnConfig { executable: None },
            ai: AiConfig {
                provider: "openai".to_string(),
                endpoint: "https://api.openai.com/v1".to_string(),
                api_key: String::new(),
                model: "gpt-4o-mini".to_string(),
                timeout_secs: 30,
            },
            diff: DiffConfig {
                context_lines: 3,
                ignore_whitespace: false,
                view_mode: "unified".to_string(),
            },
            log: LogConfig {
                fetch_limit: 100,
                show_changed_paths: true,
            },
            commit: CommitConfig { template: None },
            file_browser: FileBrowserConfig { show_hidden: false },
            behavior: BehaviorConfig {
                confirm_before_commit: true,
                confirm_before_revert: true,
                auto_refresh_secs: None,
            },
            advanced: AdvancedConfig {
                svn_timeout_secs: 60,
                log_level: "warn".to_string(),
            },
            cleanup: CleanupConfig {
                vacuum_pristines: false,
                vacuum_prunables: false,
                include_externals: false,
                remove_unversioned_trees: false,
                remove_ignored_trees: false,
                drop_dav_cache: false,
            },
            external_editor: None,
        }
    }
}
