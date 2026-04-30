use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileStatus {
    pub path: String,
    pub status: FileStatusType,
    pub is_directory: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copied: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FileStatusType {
    Modified,
    Added,
    Deleted,
    Unversioned,
    Missing,
    Conflicted,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogEntry {
    pub revision: u64,
    pub author: String,
    pub date: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changed_paths: Option<Vec<ChangedPath>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChangedPath {
    pub path: String,
    pub action: PathAction,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copy_from_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copy_from_rev: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum PathAction {
    A,
    M,
    D,
    R,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RepoInfo {
    pub url: String,
    pub root: String,
    pub revision: u64,
    pub last_changed_rev: u64,
    pub last_changed_date: String,
    pub last_changed_author: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub kind: EntryKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    pub revision: u64,
    pub author: String,
    pub date: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EntryKind {
    File,
    Dir,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShelveInfo {
    pub name: String,
    pub date: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "data")]
pub enum DiffTarget {
    File {
        path: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        revision: Option<String>,
    },
    Revisions {
        old_rev: String,
        new_rev: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommitResult {
    pub revision: u64,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateResult {
    pub revision: u64,
    pub updated_files: Vec<String>,
    pub merged_files: Vec<String>,
    pub conflicts: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReviewChunkEvent {
    pub content: String,
    pub done: bool,
}

// ---- Config Types ----

#[derive(Serialize, Deserialize, Debug, Clone)]
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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WindowConfig {
    pub width: u32,
    pub height: u32,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub maximized: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppearanceConfig {
    pub theme: String,
    pub ui_font_family: String,
    pub ui_font_size: u32,
    pub code_font_family: String,
    pub code_font_size: u32,
    pub icon_size: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SessionConfig {
    pub open_tabs: Vec<TabInfo>,
    pub active_tab_index: usize,
    pub recent_repos: Vec<RepoEntry>,
    pub max_recent_repos: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TabInfo {
    pub repo_path: String,
    pub active_view: ActiveView,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ActiveView {
    Log,
    LocalChanges,
    FileBrowser,
    Shelve,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RepoEntry {
    pub path: String,
    pub last_opened: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SvnConfig {
    pub executable: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AiConfig {
    pub provider: String,
    pub endpoint: String,
    pub api_key: String,
    pub model: String,
    pub timeout_secs: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DiffConfig {
    pub context_lines: u32,
    pub ignore_whitespace: bool,
    pub view_mode: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogConfig {
    pub fetch_limit: u32,
    pub show_changed_paths: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommitConfig {
    pub template: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileBrowserConfig {
    pub show_hidden: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BehaviorConfig {
    pub confirm_before_commit: bool,
    pub confirm_before_revert: bool,
    pub auto_refresh_secs: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AdvancedConfig {
    pub svn_timeout_secs: u64,
    pub log_level: String,
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
        }
    }
}

// ---- Error Type ----

#[derive(Debug)]
pub enum AppError {
    Svn(String),
    Ai(String),
    Fs(String),
    Config(String),
    Lock(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Svn(msg) => write!(f, "[SVN] {}", msg),
            AppError::Ai(msg) => write!(f, "[AI] {}", msg),
            AppError::Fs(msg) => write!(f, "[FS] {}", msg),
            AppError::Config(msg) => write!(f, "[CFG] {}", msg),
            AppError::Lock(msg) => write!(f, "[LOCK] {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl From<AppError> for String {
    fn from(e: AppError) -> String {
        e.to_string()
    }
}
