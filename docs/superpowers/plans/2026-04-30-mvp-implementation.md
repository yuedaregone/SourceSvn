# SourceSvn MVP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a lightweight, modern SVN GUI client using Tauri + Vue 3 + TypeScript + Rust with tab management, log/status/diff/commit views, file browser, shelve, AI code review, and settings.

**Architecture:** Backend-first approach. Rust handles all SVN commands, AI calls, shelve management, and config persistence via Tauri commands. Vue 3 frontend provides a single-instance dynamic view with tab management for multiple repos. Communication through Tauri's invoke/listen bridge.

**Tech Stack:** Tauri 2.x, Vue 3.5+, TypeScript 5.7+, Vite 6.x, Pinia, Rust 1.86+, reqwest, confy, quick-xml, serde, tokio

---

## File Structure

### Backend (Rust) — `src-tauri/`

| File | Responsibility |
|------|----------------|
| `src-tauri/Cargo.toml` | Rust dependencies (tauri, serde, quick-xml, reqwest, confy, sha2, tokio) |
| `src-tauri/tauri.conf.json` | Tauri app config (window, permissions, bundle) |
| `src-tauri/src/main.rs` | App entry, register commands, setup window + config |
| `src-tauri/src/common/mod.rs` | Shared types: FileStatus, LogEntry, RepoInfo, DirEntry, ShelveInfo, DiffTarget, AppConfig, errors |
| `src-tauri/src/svn/mod.rs` | SVN service: run_svn(), XML parsers, re-exports |
| `src-tauri/src/svn/status.rs` | svn status --xml parsing |
| `src-tauri/src/svn/log.rs` | svn log --xml parsing |
| `src-tauri/src/svn/diff.rs` | svn diff / svn diff -r |
| `src-tauri/src/svn/commit.rs` | svn commit, update, checkout, list, cat |
| `src-tauri/src/ai/mod.rs` | AiProvider trait, re-exports |
| `src-tauri/src/ai/openai.rs` | OpenAI-compatible API provider (streaming) |
| `src-tauri/src/shelve/mod.rs` | Shelve manager (save/list/apply/delete via patch files) |
| `src-tauri/src/config/mod.rs` | AppConfig loading/saving with confy, version migration |
| `src-tauri/src/commands/mod.rs` | Re-exports all command modules |
| `src-tauri/src/commands/svn.rs` | Tauri commands for SVN operations (9 commands) |
| `src-tauri/src/commands/ai.rs` | Tauri commands for AI operations (2 commands) |
| `src-tauri/src/commands/shelve.rs` | Tauri commands for Shelve operations (4 commands) |
| `src-tauri/src/commands/config.rs` | Tauri commands for config operations (2 commands) |

### Frontend (Vue 3) — `src/`

| File | Responsibility |
|------|----------------|
| `src/main.ts` | Vue app entry, register Pinia |
| `src/App.vue` | Root layout: GlobalTabBar + Toolbar + IconNavBar + dynamic view |
| `src/types/svn.ts` | TypeScript types mirroring Rust common types |
| `src/types/config.ts` | TypeScript types for AppConfig |
| `src/stores/configStore.ts` | Pinia store for global AppConfig |
| `src/stores/tabStore.ts` | Dynamic Pinia stores per tab (repo index) |
| `src/components/GlobalTabBar.vue` | Tab bar with settings button, add/close/switch tabs |
| `src/components/IconNavBar.vue` | Narrow icon nav (48px): Log, LocalChanges, FileBrowser, Shelve |
| `src/components/Toolbar.vue` | Repository-level buttons: Pull, Commit, Refresh |
| `src/components/DiffViewer.vue` | Modal diff viewer (unified/side-by-side) |
| `src/components/AiReviewPanel.vue` | Streaming AI review output panel |
| `src/views/LogView.vue` | Commit history table with filter, pagination, detail expand |
| `src/views/LocalChangesView.vue` | File status list + commit form + diff preview |
| `src/views/FileBrowserView.vue` | Tree directory browser + file content preview |
| `src/views/ShelveView.vue` | Shelve list with save/apply/delete |
| `src/views/SettingsPage.vue` | Settings modal dialog with tabbed config sections |

### Config Files — Root

| File | Responsibility |
|------|----------------|
| `.gitignore` | Ignore node_modules, target, dist, .env |
| `rustfmt.toml` | Rust formatting rules |
| `.prettierrc` | Prettier config |
| `eslint.config.js` | ESLint flat config |

---

## Task 1: Project Scaffolding

**Files:**
- Create: `src-tauri/` (via create-tauri-app)
- Create: `package.json`, `vite.config.ts`, `tsconfig.json`
- Create: `.gitignore`, `rustfmt.toml`, `.prettierrc`, `eslint.config.js`

- [ ] **Step 1: Initialize Tauri + Vue 3 project**

Run in project root:
```bash
cd D:/study/github/SourceSvn
pnpm create tauri-app sourcesvn --template vue-ts
cd sourcesvn
```

Note: If `create-tauri-app` asks interactive questions, select:
- Package manager: pnpm
- Language: TypeScript
- UI framework: Vue

- [ ] **Step 2: Move generated files to project root**

Move all files from `sourcesvn/` to `D:/study/github/SourceSvn/`:
```bash
mv sourcesvn/* sourcesvn/.* . 2>/dev/null
rm -rf sourcesvn
```

- [ ] **Step 3: Create .gitignore**

```gitignore
# Node
node_modules/
dist/

# Rust
src-tauri/target/

# IDE
.vscode/
.idea/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# Environment
.env
.env.local
```

- [ ] **Step 4: Create rustfmt.toml**

```toml
edition = "2021"
max_width = 100
hard_tabs = false
tab_spaces = 2
use_small_heuristics = "Max"
reorder_imports = true
reorder_modules = true
```

- [ ] **Step 5: Create .prettierrc**

```json
{
  "semi": false,
  "singleQuote": true,
  "tabWidth": 2,
  "trailingComma": "es5",
  "printWidth": 100,
  "endOfLine": "auto"
}
```

- [ ] **Step 6: Install frontend dependencies**

```bash
cd D:/study/github/SourceSvn
pnpm install
pnpm add pinia
```

- [ ] **Step 7: Verify dev server starts**

```bash
pnpm tauri dev
```

Expected: Tauri window opens with Vue 3 default page. Kill after verification.

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "chore: initialize Tauri + Vue 3 project scaffold"
```

---

## Task 2: Backend Common Types

**Files:**
- Create: `src-tauri/Cargo.toml` (modify existing)
- Create: `src-tauri/src/common/mod.rs`

- [ ] **Step 1: Update Cargo.toml dependencies**

Read existing `src-tauri/Cargo.toml`, then edit to add required dependencies:

```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-shell = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
quick-xml = { version = "0.36", features = ["serialize"] }
reqwest = { version = "0.12", features = ["json", "stream"] }
tokio = { version = "1", features = ["full"] }
confy = "0.5"
sha2 = "0.10"
chrono = { version = "0.4", features = ["serde"] }
async-trait = "0.1"
log = "0.4"
env_logger = "0.11"

[build-dependencies]
tauri-build = { version = "2", features = [] }
```

- [ ] **Step 2: Create common/mod.rs with shared types**

```rust
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
```

- [ ] **Step 3: Verify Rust compiles**

```bash
cd D:/study/github/SourceSvn
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: Compilation succeeds (or warnings only).

- [ ] **Step 4: Commit**

```bash
git add src-tauri/
git commit -m "feat(common): add shared types and AppConfig"
```

---

## Task 3: Config Module

**Files:**
- Create: `src-tauri/src/config/mod.rs`

- [ ] **Step 1: Create config/mod.rs**

```rust
use crate::common::AppConfig;
use std::path::PathBuf;

const APP_NAME: &str = "sourcesvn";
const CONFIG_FILE: &str = "config.toml";

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
```

- [ ] **Step 2: Add dirs dependency to Cargo.toml**

Add to `[dependencies]` in `src-tauri/Cargo.toml`:
```toml
dirs = "5"
```

- [ ] **Step 3: Verify compilation**

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

- [ ] **Step 4: Commit**

```bash
git add src-tauri/
git commit -m "feat(config): add config loading and saving with confy"
```

---

## Task 4: SVN Service — Core Runner + Status Parser

**Files:**
- Create: `src-tauri/src/svn/mod.rs`
- Create: `src-tauri/src/svn/status.rs`

- [ ] **Step 1: Create svn/mod.rs with run_svn**

```rust
pub mod status;
pub mod log;
pub mod diff;
pub mod commit;

use crate::common::AppError;
use std::process::Command;
use std::time::Duration;

pub fn run_svn(args: &[&str], timeout_secs: u64) -> Result<String, AppError> {
    let svn_path = find_svn_executable()?;
    let output = Command::new(&svn_path)
        .args(args)
        .output()
        .map_err(|e| AppError::Svn(format!("Failed to execute svn: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Svn(format!("SVN command failed: {}", stderr.trim())));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn find_svn_executable() -> Result<String, AppError> {
    let output = Command::new("where")
        .arg("svn")
        .output()
        .map_err(|e| AppError::Svn(format!("Failed to find svn: {}", e)))?;

    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() {
            return Ok(path);
        }
    }

    Err(AppError::Svn(
        "SVN command line tool not found. Please install SVN client or configure path.".to_string(),
    ))
}
```

- [ ] **Step 2: Create svn/status.rs with XML parser**

```rust
use crate::common::{AppError, FileStatus, FileStatusType};
use quick_xml::de::from_str;
use serde::Deserialize;

#[derive(Deserialize)]
struct StatusXml {
    #[serde(rename = "target")]
    target: Option<StatusTarget>,
}

#[derive(Deserialize)]
struct StatusTarget {
    #[serde(rename = "entry")]
    entries: Option<Vec<StatusEntry>>,
}

#[derive(Deserialize)]
struct StatusEntry {
    #[serde(rename = "@path")]
    path: String,
    #[serde(rename = "wc-status")]
    wc_status: WcStatus,
}

#[derive(Deserialize)]
struct WcStatus {
    #[serde(rename = "@item")]
    item: String,
    #[serde(rename = "@props")]
    props: Option<String>,
    #[serde(rename = "@copy-from-url")]
    copy_from_url: Option<String>,
}

pub fn parse_status_xml(xml: &str) -> Result<Vec<FileStatus>, AppError> {
    let status: StatusXml = from_str(xml)
        .map_err(|e| AppError::Svn(format!("Failed to parse status XML: {}", e)))?;

    let target = status.target.ok_or_else(|| {
        AppError::Svn("No target found in status XML".to_string())
    })?;

    let entries = target.entries.unwrap_or_default();

    Ok(entries
        .into_iter()
        .map(|entry| {
            let status_type = match entry.wc_status.item.as_str() {
                "modified" => FileStatusType::Modified,
                "added" => FileStatusType::Added,
                "deleted" => FileStatusType::Deleted,
                "unversioned" => FileStatusType::Unversioned,
                "missing" => FileStatusType::Missing,
                "conflicted" => FileStatusType::Conflicted,
                _ => FileStatusType::Unversioned,
            };

            FileStatus {
                path: entry.path,
                status: status_type,
                is_directory: false,
                copied: entry.wc_status.copy_from_url.map(|_| true),
            }
        })
        .collect())
}

pub fn svn_status(path: &str, timeout_secs: u64) -> Result<Vec<FileStatus>, AppError> {
    let xml = crate::svn::run_svn(&["status", "--xml", path], timeout_secs)?;
    parse_status_xml(&xml)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_status_xml_modified() {
        let xml = r#"<?xml version="1.0"?>
<status>
  <target>
    <entry path="src/main.rs">
      <wc-status item="modified" revision="100"/>
    </entry>
  </target>
</status>"#;
        let result = parse_status_xml(xml).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].path, "src/main.rs");
        assert_eq!(result[0].status, FileStatusType::Modified);
    }

    #[test]
    fn test_parse_status_xml_multiple() {
        let xml = r#"<?xml version="1.0"?>
<status>
  <target>
    <entry path="a.txt">
      <wc-status item="modified"/>
    </entry>
    <entry path="b.txt">
      <wc-status item="added"/>
    </entry>
    <entry path="c.txt">
      <wc-status item="deleted"/>
    </entry>
  </target>
</status>"#;
        let result = parse_status_xml(xml).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].status, FileStatusType::Modified);
        assert_eq!(result[1].status, FileStatusType::Added);
        assert_eq!(result[2].status, FileStatusType::Deleted);
    }

    #[test]
    fn test_parse_status_xml_empty() {
        let xml = r#"<?xml version="1.0"?>
<status>
  <target/>
</status>"#;
        let result = parse_status_xml(xml).unwrap();
        assert_eq!(result.len(), 0);
    }
}
```

- [ ] **Step 3: Run tests**

```bash
cargo test --manifest-path src-tauri/Cargo.toml -- svn::status
```

Expected: 3 tests pass.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/svn/
git commit -m "feat(svn): add run_svn core and status XML parser with tests"
```

---

## Task 5: SVN Service — Log Parser

**Files:**
- Create: `src-tauri/src/svn/log.rs`

- [ ] **Step 1: Create svn/log.rs**

```rust
use crate::common::{AppError, ChangedPath, LogEntry, PathAction};
use quick_xml::de::from_str;
use serde::Deserialize;

#[derive(Deserialize)]
struct LogXml {
    #[serde(rename = "logentry")]
    entries: Option<Vec<LogEntryXml>>,
}

#[derive(Deserialize)]
struct LogEntryXml {
    #[serde(rename = "@revision")]
    revision: u64,
    author: String,
    date: String,
    msg: String,
    #[serde(rename = "paths")]
    paths: Option<PathsXml>,
}

#[derive(Deserialize)]
struct PathsXml {
    #[serde(rename = "path")]
    entries: Option<Vec<PathEntryXml>>,
}

#[derive(Deserialize)]
struct PathEntryXml {
    #[serde(rename = "@action")]
    action: String,
    #[serde(rename = "@copyfrom-path")]
    copy_from_path: Option<String>,
    #[serde(rename = "@copyfrom-rev")]
    copy_from_rev: Option<u64>,
    #[serde(rename = "$text")]
    text: String,
}

pub fn parse_log_xml(xml: &str) -> Result<Vec<LogEntry>, AppError> {
    let log: LogXml = from_str(xml)
        .map_err(|e| AppError::Svn(format!("Failed to parse log XML: {}", e)))?;

    let entries = log.entries.unwrap_or_default();

    Ok(entries
        .into_iter()
        .map(|entry| {
            let changed_paths = entry.paths.and_then(|p| p.entries).map(|paths| {
                paths
                    .into_iter()
                    .map(|p| {
                        let action = match p.action.as_str() {
                            "A" => PathAction::A,
                            "M" => PathAction::M,
                            "D" => PathAction::D,
                            "R" => PathAction::R,
                            _ => PathAction::M,
                        };
                        ChangedPath {
                            path: p.text,
                            action,
                            copy_from_path: p.copy_from_path,
                            copy_from_rev: p.copy_from_rev,
                        }
                    })
                    .collect()
            });

            LogEntry {
                revision: entry.revision,
                author: entry.author,
                date: entry.date,
                message: entry.msg,
                changed_paths,
            }
        })
        .collect())
}

pub fn svn_log(
    path: &str,
    limit: Option<u32>,
    from_rev: Option<&str>,
    timeout_secs: u64,
) -> Result<Vec<LogEntry>, AppError> {
    let mut args = vec!["log", "--xml", "-v", path];
    let limit_str;
    if let Some(l) = limit {
        limit_str = format!("-l:{}", l);
        args.push(&limit_str);
    }
    if let Some(rev) = from_rev {
        args.push("-r");
        args.push(rev);
    }
    let xml = crate::svn::run_svn(&args, timeout_secs)?;
    parse_log_xml(&xml)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_log_xml_single() {
        let xml = r#"<?xml version="1.0"?>
<log>
  <logentry revision="100">
    <author>alice</author>
    <date>2026-04-30T10:30:00Z</date>
    <msg>Fix login bug</msg>
    <paths>
      <path action="M">src/main.rs</path>
      <path action="A">tests/test.rs</path>
    </paths>
  </logentry>
</log>"#;
        let result = parse_log_xml(xml).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].revision, 100);
        assert_eq!(result[0].author, "alice");
        assert_eq!(result[0].message, "Fix login bug");
        let paths = result[0].changed_paths.as_ref().unwrap();
        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0].action, PathAction::M);
        assert_eq!(paths[1].action, PathAction::A);
    }

    #[test]
    fn test_parse_log_xml_empty() {
        let xml = r#"<?xml version="1.0"?>
<log/>"#;
        let result = parse_log_xml(xml).unwrap();
        assert_eq!(result.len(), 0);
    }
}
```

- [ ] **Step 2: Run tests**

```bash
cargo test --manifest-path src-tauri/Cargo.toml -- svn::log
```

Expected: 2 tests pass.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/svn/log.rs
git commit -m "feat(svn): add log XML parser with tests"
```

---

## Task 6: SVN Service — Diff, Commit (update/checkout/list/cat)

**Files:**
- Create: `src-tauri/src/svn/diff.rs`
- Create: `src-tauri/src/svn/commit.rs`

- [ ] **Step 1: Create svn/diff.rs**

```rust
use crate::common::{AppError, DiffTarget};

pub fn svn_diff(
    path: &str,
    target: &DiffTarget,
    timeout_secs: u64,
) -> Result<String, AppError> {
    let mut args = vec!["diff", "--xml", path];

    match target {
        DiffTarget::File { path: file_path, revision } => {
            args.push(file_path);
            if let Some(rev) = revision {
                args.push("-r");
                args.push(rev);
            }
        }
        DiffTarget::Revisions { old_rev, new_rev } => {
            let rev_range = format!("{}:{}", old_rev, new_rev);
            args.push("-r");
            args.push(&rev_range);
        }
    }

    crate::svn::run_svn(&args, timeout_secs)
}
```

- [ ] **Step 2: Create svn/commit.rs**

```rust
use crate::common::{
    AppError, CommitResult, DirEntry, EntryKind, RepoInfo, UpdateResult,
};
use quick_xml::de::from_str;
use serde::Deserialize;

pub fn svn_commit(
    path: &str,
    message: &str,
    files: &[String],
    timeout_secs: u64,
) -> Result<CommitResult, AppError> {
    let mut args = vec!["commit", "-m", message, path];
    for f in files {
        args.push(f);
    }
    let output = crate::svn::run_svn(&args, timeout_secs)?;

    let revision = extract_revision_from_output(&output);
    Ok(CommitResult {
        revision,
        success: true,
        errors: None,
    })
}

fn extract_revision_from_output(output: &str) -> u64 {
    for line in output.lines() {
        if line.contains("Committed revision") {
            if let Some(rev_str) = line.split_whitespace().last() {
                if let Ok(rev) = rev_str.trim_end_matches('.').parse::<u64>() {
                    return rev;
                }
            }
        }
    }
    0
}

pub fn svn_update(path: &str, timeout_secs: u64) -> Result<UpdateResult, AppError> {
    let output = crate::svn::run_svn(&["update", "--xml", path], timeout_secs)?;
    parse_update_xml(&output)
}

#[derive(Deserialize)]
struct UpdateXml {
    #[serde(rename = "update-report")]
    report: Option<UpdateReport>,
}

#[derive(Deserialize)]
struct UpdateReport {
    #[serde(rename = "target")]
    target: Option<String>,
    #[serde(rename = "revision")]
    revision: Option<String>,
}

fn parse_update_xml(xml: &str) -> Result<UpdateResult, AppError> {
    let _report: UpdateXml = from_str(xml)
        .map_err(|e| AppError::Svn(format!("Failed to parse update XML: {}", e)))?;

    Ok(UpdateResult {
        revision: 0,
        updated_files: vec![],
        merged_files: vec![],
        conflicts: vec![],
    })
}

pub fn svn_checkout(
    url: &str,
    dest: &str,
    timeout_secs: u64,
) -> Result<(), AppError> {
    crate::svn::run_svn(&["checkout", url, dest], timeout_secs)?;
    Ok(())
}

pub fn svn_list(
    path: &str,
    revision: Option<&str>,
    recursive: bool,
    timeout_secs: u64,
) -> Result<Vec<DirEntry>, AppError> {
    let mut args = vec!["list", "--xml", path];
    if let Some(rev) = revision {
        args.push("-r");
        args.push(rev);
    }
    if recursive {
        args.push("-R");
    }
    let xml = crate::svn::run_svn(&args, timeout_secs)?;
    parse_list_xml(&xml)
}

#[derive(Deserialize)]
struct ListXml {
    #[serde(rename = "list")]
    list: Option<ListEntry>,
}

#[derive(Deserialize)]
struct ListEntry {
    #[serde(rename = "entry")]
    entries: Option<Vec<ListItem>>,
}

#[derive(Deserialize)]
struct ListItem {
    #[serde(rename = "@kind")]
    kind: String,
    name: String,
    #[serde(rename = "size")]
    size: Option<u64>,
    #[serde(rename = "commit")]
    commit: Option<ListCommit>,
}

#[derive(Deserialize)]
struct ListCommit {
    #[serde(rename = "@revision")]
    revision: u64,
    author: Option<String>,
    date: Option<String>,
}

fn parse_list_xml(xml: &str) -> Result<Vec<DirEntry>, AppError> {
    let list_xml: ListXml = from_str(xml)
        .map_err(|e| AppError::Svn(format!("Failed to parse list XML: {}", e)))?;

    let list = list_xml.list.ok_or_else(|| {
        AppError::Svn("No list found in XML".to_string())
    })?;

    let entries = list.entries.unwrap_or_default();

    Ok(entries
        .into_iter()
        .map(|entry| {
            let kind = if entry.kind == "dir" {
                EntryKind::Dir
            } else {
                EntryKind::File
            };
            let commit = entry.commit.unwrap_or(ListCommit {
                revision: 0,
                author: None,
                date: None,
            });
            DirEntry {
                name: entry.name,
                kind,
                size: entry.size,
                revision: commit.revision,
                author: commit.author.unwrap_or_default(),
                date: commit.date.unwrap_or_default(),
            }
        })
        .collect())
}

pub fn svn_cat(
    path: &str,
    revision: Option<&str>,
    timeout_secs: u64,
) -> Result<String, AppError> {
    let mut args = vec!["cat", path];
    if let Some(rev) = revision {
        args.push("-r");
        args.push(rev);
    }
    crate::svn::run_svn(&args, timeout_secs)
}

pub fn svn_info(path: &str, timeout_secs: u64) -> Result<RepoInfo, AppError> {
    let xml = crate::svn::run_svn(&["info", "--xml", path], timeout_secs)?;
    parse_info_xml(&xml)
}

#[derive(Deserialize)]
struct InfoXml {
    info: Option<InfoEntry>,
}

#[derive(Deserialize)]
struct InfoEntry {
    entry: Option<InfoDetail>,
}

#[derive(Deserialize)]
struct InfoDetail {
    #[serde(rename = "@url")]
    url: String,
    #[serde(rename = "@revision")]
    revision: u64,
    repository: Option<InfoRepository>,
    wc_info: Option<InfoWcInfo>,
}

#[derive(Deserialize)]
struct InfoRepository {
    root: Option<String>,
}

#[derive(Deserialize)]
struct InfoWcInfo {
    #[serde(rename = "@revision")]
    revision: Option<u64>,
}

fn parse_info_xml(xml: &str) -> Result<RepoInfo, AppError> {
    let info_xml: InfoXml = from_str(xml)
        .map_err(|e| AppError::Svn(format!("Failed to parse info XML: {}", e)))?;

    let info = info_xml.info.ok_or_else(|| {
        AppError::Svn("No info found in XML".to_string())
    })?;

    let entry = info.entry.ok_or_else(|| {
        AppError::Svn("No entry found in info XML".to_string())
    })?;

    let root = entry
        .repository
        .and_then(|r| r.root)
        .unwrap_or_default();

    Ok(RepoInfo {
        url: entry.url,
        root,
        revision: entry.revision,
        last_changed_rev: 0,
        last_changed_date: String::new(),
        last_changed_author: String::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_revision() {
        let output = "Sending        src/main.rs\nCommitted revision 42.\n";
        assert_eq!(extract_revision_from_output(output), 42);
    }

    #[test]
    fn test_extract_revision_no_match() {
        let output = "No changes.\n";
        assert_eq!(extract_revision_from_output(output), 0);
    }

    #[test]
    fn test_parse_list_xml() {
        let xml = r#"<?xml version="1.0"?>
<list path="/">
  <entry kind="dir">
    <name>src</name>
    <commit revision="10">
      <author>alice</author>
      <date>2026-04-30T10:00:00Z</date>
    </commit>
  </entry>
  <entry kind="file">
    <name>README.md</name>
    <size>1234</size>
    <commit revision="5">
      <author>bob</author>
      <date>2026-04-28T09:00:00Z</date>
    </commit>
  </entry>
</list>"#;
        let result = parse_list_xml(xml).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].kind, EntryKind::Dir);
        assert_eq!(result[0].name, "src");
        assert_eq!(result[1].kind, EntryKind::File);
        assert_eq!(result[1].size, Some(1234));
    }
}
```

- [ ] **Step 3: Run tests**

```bash
cargo test --manifest-path src-tauri/Cargo.toml -- svn::commit
```

Expected: 3 tests pass.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/svn/
git commit -m "feat(svn): add diff, commit, update, checkout, list, cat, info commands"
```

---

## Task 7: AI Service Module

**Files:**
- Create: `src-tauri/src/ai/mod.rs`
- Create: `src-tauri/src/ai/openai.rs`

- [ ] **Step 1: Create ai/mod.rs with AiProvider trait**

```rust
pub mod openai;

use crate::common::AppError;
use async_trait::async_trait;
use tauri::AppHandle;

#[async_trait]
pub trait AiProvider: Send + Sync {
    async fn generate_message(&self, diff: &str) -> Result<String, AppError>;
    async fn review_changes(&self, diff: &str, app_handle: &AppHandle) -> Result<(), AppError>;
}

pub fn create_provider(endpoint: &str, api_key: &str, model: &str, timeout_secs: u64) -> Box<dyn AiProvider> {
    Box::new(openai::OpenAiProvider::new(
        endpoint,
        api_key,
        model,
        timeout_secs,
    ))
}
```

- [ ] **Step 2: Create ai/openai.rs**

```rust
use super::AiProvider;
use crate::common::{AppError, ReviewChunkEvent};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

pub struct OpenAiProvider {
    client: Client,
    endpoint: String,
    api_key: String,
    model: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

#[derive(Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Deserialize)]
struct StreamChunk {
    choices: Vec<StreamChoice>,
}

#[derive(Deserialize)]
struct StreamChoice {
    delta: Option<StreamDelta>,
}

#[derive(Deserialize)]
struct StreamDelta {
    content: Option<String>,
}

impl OpenAiProvider {
    pub fn new(endpoint: &str, api_key: &str, model: &str, timeout_secs: u64) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            endpoint: endpoint.to_string(),
            api_key: api_key.to_string(),
            model: model.to_string(),
        }
    }

    async fn chat_completion(&self, messages: Vec<ChatMessage>) -> Result<String, AppError> {
        let request = ChatRequest {
            model: self.model.clone(),
            messages,
            stream: false,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.endpoint))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Ai(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Ai(format!("API error {}: {}", status, body)));
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .map_err(|e| AppError::Ai(format!("Failed to parse response: {}", e)))?;

        chat_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| AppError::Ai("No response from AI".to_string()))
    }
}

#[async_trait]
impl AiProvider for OpenAiProvider {
    async fn generate_message(&self, diff: &str) -> Result<String, AppError> {
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are a helpful assistant that generates concise commit messages for code changes. Output ONLY the commit message, no explanation.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: format!("Generate a concise commit message for these changes:\n\n{}", diff),
            },
        ];
        self.chat_completion(messages).await
    }

    async fn review_changes(&self, diff: &str, app_handle: &AppHandle) -> Result<(), AppError> {
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are a senior code reviewer. Review the following code changes and provide constructive feedback on potential issues, bugs, and improvements. Be concise.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: format!("Review these code changes:\n\n{}", diff),
            },
        ];

        let request = ChatRequest {
            model: self.model.clone(),
            messages,
            stream: true,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.endpoint))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Ai(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Ai(format!("API error {}: {}", status, body)));
        }

        let mut buffer = String::new();
        let mut stream = response.bytes_stream();

        use futures_util::StreamExt;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| AppError::Ai(format!("Stream error: {}", e)))?;
            let text = String::from_utf8_lossy(&chunk);
            buffer.push_str(&text);

            while let Some(line_end) = buffer.find('\n') {
                let line = buffer[..line_end].trim().to_string();
                buffer = buffer[line_end + 1..].to_string();

                if line.starts_with("data: ") {
                    let data = &line[6..];
                    if data == "[DONE]" {
                        let _ = app_handle.emit(
                            "review_chunk",
                            ReviewChunkEvent {
                                content: String::new(),
                                done: true,
                            },
                        );
                        return Ok(());
                    }
                    if let Ok(chunk) = serde_json::from_str::<StreamChunk>(data) {
                        if let Some(choice) = chunk.choices.first() {
                            if let Some(delta) = &choice.delta {
                                if let Some(content) = &delta.content {
                                    let _ = app_handle.emit(
                                        "review_chunk",
                                        ReviewChunkEvent {
                                            content: content.clone(),
                                            done: false,
                                        },
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        let _ = app_handle.emit(
            "review_chunk",
            ReviewChunkEvent {
                content: String::new(),
                done: true,
            },
        );
        Ok(())
    }
}
```

- [ ] **Step 3: Add futures-util dependency to Cargo.toml**

Add to `[dependencies]`:
```toml
futures-util = "0.3"
```

- [ ] **Step 4: Verify compilation**

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/ai/
git commit -m "feat(ai): add AiProvider trait and OpenAI streaming provider"
```

---

## Task 8: Shelve Module

**Files:**
- Create: `src-tauri/src/shelve/mod.rs`

- [ ] **Step 1: Create shelve/mod.rs**

```rust
use crate::common::{AppError, ShelveInfo};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

fn shelve_base_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("sourcesvn")
        .join("shelves")
}

fn repo_shelve_dir(repo_path: &str) -> PathBuf {
    let mut hasher = Sha256::new();
    hasher.update(repo_path.as_bytes());
    let hash = format!("{:x}", hasher.finalize());
    let short_hash = &hash[..12];
    shelve_base_dir().join(short_hash)
}

pub fn shelve_save(repo_path: &str, name: &str) -> Result<(), AppError> {
    let shelve_dir = repo_shelve_dir(repo_path);
    fs::create_dir_all(&shelve_dir)
        .map_err(|e| AppError::Fs(format!("Failed to create shelve directory: {}", e)))?;

    let patch_file = shelve_dir.join(format!("{}.patch", name));
    if patch_file.exists() {
        return Err(AppError::Fs(format!("Shelve '{}' already exists", name)));
    }

    let diff = crate::svn::run_svn(&["diff", repo_path], 60)
        .map_err(|e| AppError::Svn(format!("Failed to get diff: {}", e)))?;

    if diff.trim().is_empty() {
        return Err(AppError::Fs("No changes to shelve".to_string()));
    }

    fs::write(&patch_file, &diff)
        .map_err(|e| AppError::Fs(format!("Failed to write patch file: {}", e)))?;

    Ok(())
}

pub fn shelve_list(repo_path: &str) -> Result<Vec<ShelveInfo>, AppError> {
    let shelve_dir = repo_shelve_dir(repo_path);
    if !shelve_dir.exists() {
        return Ok(vec![]);
    }

    let entries = fs::read_dir(&shelve_dir)
        .map_err(|e| AppError::Fs(format!("Failed to read shelve directory: {}", e)))?;

    let mut shelves = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| AppError::Fs(format!("Failed to read entry: {}", e)))?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("patch") {
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();
            let date = fs::metadata(&path)
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| {
                    let datetime: chrono::DateTime<chrono::Utc> = t.into();
                    Some(datetime.to_rfc3339())
                })
                .unwrap_or_default();
            shelves.push(ShelveInfo { name, date });
        }
    }

    shelves.sort_by(|a, b| b.date.cmp(&a.date));
    Ok(shelves)
}

pub fn shelve_apply(repo_path: &str, name: &str) -> Result<(), AppError> {
    let shelve_dir = repo_shelve_dir(repo_path);
    let patch_file = shelve_dir.join(format!("{}.patch", name));
    if !patch_file.exists() {
        return Err(AppError::Fs(format!("Shelve '{}' not found", name)));
    }

    crate::svn::run_svn(
        &["patch", patch_file.to_str().unwrap_or(""), repo_path],
        60,
    )?;

    Ok(())
}

pub fn shelve_delete(repo_path: &str, name: &str) -> Result<(), AppError> {
    let shelve_dir = repo_shelve_dir(repo_path);
    let patch_file = shelve_dir.join(format!("{}.patch", name));
    if !patch_file.exists() {
        return Err(AppError::Fs(format!("Shelve '{}' not found", name)));
    }

    fs::remove_file(&patch_file)
        .map_err(|e| AppError::Fs(format!("Failed to delete shelve: {}", e)))?;

    Ok(())
}
```

- [ ] **Step 2: Add chrono dependency**

Already added in Task 2. Verify `chrono` with `serde` feature is in Cargo.toml.

- [ ] **Step 3: Verify compilation**

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/shelve/
git commit -m "feat(shelve): add shelve save/list/apply/delete with patch files"
```

---

## Task 9: Tauri Command Layer

**Files:**
- Create: `src-tauri/src/commands/mod.rs`
- Create: `src-tauri/src/commands/svn.rs`
- Create: `src-tauri/src/commands/ai.rs`
- Create: `src-tauri/src/commands/shelve.rs`
- Create: `src-tauri/src/commands/config.rs`

- [ ] **Step 1: Create commands/mod.rs**

```rust
pub mod svn;
pub mod ai;
pub mod shelve;
pub mod config;
```

- [ ] **Step 2: Create commands/svn.rs**

```rust
use crate::common::*;
use crate::config::load_config;
use crate::svn;

#[tauri::command]
pub fn svn_status(path: String) -> Result<Vec<FileStatus>, String> {
    let config = load_config();
    svn::status::svn_status(&path, config.advanced.svn_timeout_secs).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn svn_info(path: String) -> Result<RepoInfo, String> {
    let config = load_config();
    svn::commit::svn_info(&path, config.advanced.svn_timeout_secs).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn svn_log(
    path: String,
    limit: Option<u32>,
    from_rev: Option<String>,
) -> Result<Vec<LogEntry>, String> {
    let config = load_config();
    svn::log::svn_log(&path, limit, from_rev.as_deref(), config.advanced.svn_timeout_secs)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn svn_diff(path: String, target: DiffTarget) -> Result<String, String> {
    let config = load_config();
    svn::diff::svn_diff(&path, &target, config.advanced.svn_timeout_secs)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn svn_commit(
    path: String,
    message: String,
    files: Vec<String>,
) -> Result<CommitResult, String> {
    let config = load_config();
    svn::commit::svn_commit(&path, &message, &files, config.advanced.svn_timeout_secs)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn svn_list(
    path: String,
    revision: Option<String>,
    recursive: bool,
) -> Result<Vec<DirEntry>, String> {
    let config = load_config();
    svn::commit::svn_list(
        &path,
        revision.as_deref(),
        recursive,
        config.advanced.svn_timeout_secs,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn svn_cat(path: String, revision: Option<String>) -> Result<String, String> {
    let config = load_config();
    svn::commit::svn_cat(&path, revision.as_deref(), config.advanced.svn_timeout_secs)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn svn_checkout(url: String, dest: String) -> Result<(), String> {
    let config = load_config();
    svn::commit::svn_checkout(&url, &dest, config.advanced.svn_timeout_secs)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn svn_update(path: String) -> Result<UpdateResult, String> {
    let config = load_config();
    svn::commit::svn_update(&path, config.advanced.svn_timeout_secs).map_err(|e| e.to_string())
}
```

- [ ] **Step 3: Create commands/ai.rs**

```rust
use crate::ai;
use crate::common::{AppConfig, ReviewChunkEvent};
use crate::config::load_config;
use tauri::AppHandle;

#[tauri::command]
pub async fn generate_commit_message(diff: String) -> Result<String, String> {
    let config = load_config();
    if config.ai.api_key.is_empty() {
        return Err("[AI] API key not configured".to_string());
    }

    let provider = ai::create_provider(
        &config.ai.endpoint,
        &config.ai.api_key,
        &config.ai.model,
        config.ai.timeout_secs,
    );

    provider.generate_message(&diff).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn review_changes(diff: String, app_handle: AppHandle) -> Result<(), String> {
    let config = load_config();
    if config.ai.api_key.is_empty() {
        return Err("[AI] API key not configured".to_string());
    }

    let provider = ai::create_provider(
        &config.ai.endpoint,
        &config.ai.api_key,
        &config.ai.model,
        config.ai.timeout_secs,
    );

    provider.review_changes(&diff, &app_handle).await.map_err(|e| e.to_string())
}
```

- [ ] **Step 4: Create commands/shelve.rs**

```rust
use crate::common::ShelveInfo;
use crate::shelve;

#[tauri::command]
pub fn shelve_save(path: String, name: String) -> Result<(), String> {
    shelve::shelve_save(&path, &name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn shelve_list(path: String) -> Result<Vec<ShelveInfo>, String> {
    shelve::shelve_list(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn shelve_apply(path: String, name: String) -> Result<(), String> {
    shelve::shelve_apply(&path, &name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn shelve_delete(path: String, name: String) -> Result<(), String> {
    shelve::shelve_delete(&path, &name).map_err(|e| e.to_string())
}
```

- [ ] **Step 5: Create commands/config.rs**

```rust
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
```

- [ ] **Step 6: Update main.rs to register commands**

Replace `src-tauri/src/main.rs`:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod common;
mod commands;
mod config;
mod svn;
mod ai;
mod shelve;

fn main() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let conf = config::load_config();
            let window = app.get_webview_window("main").expect("failed to get main window");
            if conf.window.maximized {
                let _ = window.maximize();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::svn::svn_status,
            commands::svn::svn_info,
            commands::svn::svn_log,
            commands::svn::svn_diff,
            commands::svn::svn_commit,
            commands::svn::svn_list,
            commands::svn::svn_cat,
            commands::svn::svn_checkout,
            commands::svn::svn_update,
            commands::ai::generate_commit_message,
            commands::ai::review_changes,
            commands::shelve::shelve_save,
            commands::shelve::shelve_list,
            commands::shelve::shelve_apply,
            commands::shelve::shelve_delete,
            commands::config::get_config,
            commands::config::set_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 7: Verify compilation**

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

- [ ] **Step 8: Commit**

```bash
git add src-tauri/
git commit -m "feat(commands): register all 17 Tauri commands with error handling"
```

---

## Task 10: Frontend Types and Stores

**Files:**
- Create: `src/types/svn.ts`
- Create: `src/types/config.ts`
- Create: `src/stores/configStore.ts`
- Create: `src/stores/tabStore.ts`

- [ ] **Step 1: Create src/types/svn.ts**

```typescript
export type FileStatusType = 'modified' | 'added' | 'deleted' | 'unversioned' | 'missing' | 'conflicted'

export interface FileStatus {
  path: string
  status: FileStatusType
  isDirectory: boolean
  copied?: boolean
}

export interface LogEntry {
  revision: number
  author: string
  date: string
  message: string
  changedPaths?: ChangedPath[]
}

export interface ChangedPath {
  path: string
  action: 'A' | 'M' | 'D' | 'R'
  copyFromPath?: string
  copyFromRev?: number
}

export interface RepoInfo {
  url: string
  root: string
  revision: number
  lastChangedRev: number
  lastChangedDate: string
  lastChangedAuthor: string
}

export interface DirEntry {
  name: string
  kind: 'file' | 'dir'
  size?: number
  revision: number
  author: string
  date: string
}

export interface ShelveInfo {
  name: string
  date: string
}

export type DiffTarget =
  | { type: 'File'; data: { path: string; revision?: string } }
  | { type: 'Revisions'; data: { oldRev: string; newRev: string } }

export interface CommitResult {
  revision: number
  success: boolean
  errors?: string[]
}

export interface UpdateResult {
  revision: number
  updatedFiles: string[]
  mergedFiles: string[]
  conflicts: string[]
}

export type ActiveView = 'log' | 'localChanges' | 'fileBrowser' | 'shelve'
```

- [ ] **Step 2: Create src/types/config.ts**

```typescript
export interface AppConfig {
  configVersion: number
  window: WindowConfig
  appearance: AppearanceConfig
  session: SessionConfig
  svn: SvnConfig
  ai: AiConfig
  diff: DiffConfig
  log: LogConfig
  commit: CommitConfig
  fileBrowser: FileBrowserConfig
  behavior: BehaviorConfig
  advanced: AdvancedConfig
}

export interface WindowConfig {
  width: number
  height: number
  x?: number
  y?: number
  maximized: boolean
}

export interface AppearanceConfig {
  theme: string
  uiFontFamily: string
  uiFontSize: number
  codeFontFamily: string
  codeFontSize: number
  iconSize: number
}

export interface SessionConfig {
  openTabs: TabInfo[]
  activeTabIndex: number
  recentRepos: RepoEntry[]
  maxRecentRepos: number
}

export interface TabInfo {
  repoPath: string
  activeView: 'log' | 'localChanges' | 'fileBrowser' | 'shelve'
}

export interface RepoEntry {
  path: string
  lastOpened: string
}

export interface SvnConfig {
  executable?: string
}

export interface AiConfig {
  provider: string
  endpoint: string
  apiKey: string
  model: string
  timeoutSecs: number
}

export interface DiffConfig {
  contextLines: number
  ignoreWhitespace: boolean
  viewMode: 'unified' | 'side_by_side'
}

export interface LogConfig {
  fetchLimit: number
  showChangedPaths: boolean
}

export interface CommitConfig {
  template?: string
}

export interface FileBrowserConfig {
  showHidden: boolean
}

export interface BehaviorConfig {
  confirmBeforeCommit: boolean
  confirmBeforeRevert: boolean
  autoRefreshSecs?: number
}

export interface AdvancedConfig {
  svnTimeoutSecs: number
  logLevel: 'error' | 'warn' | 'info' | 'debug'
}
```

- [ ] **Step 3: Create src/stores/configStore.ts**

```typescript
import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import type { AppConfig } from '../types/config'

export const useConfigStore = defineStore('config', {
  state: () => ({
    config: null as AppConfig | null,
  }),
  getters: {
    theme: (state) => state.config?.appearance.theme ?? 'light',
    fontSize: (state) => state.config?.appearance.uiFontSize ?? 14,
    codeFont: (state) => state.config?.appearance.codeFontFamily ?? 'monospace',
  },
  actions: {
    async loadConfig() {
      try {
        this.config = await invoke<AppConfig>('get_config')
      } catch (e) {
        console.error('Failed to load config:', e)
      }
    },
    async saveConfig() {
      if (!this.config) return
      try {
        await invoke('set_config', { conf: this.config })
      } catch (e) {
        console.error('Failed to save config:', e)
      }
    },
    updateTheme(theme: string) {
      if (!this.config) return
      this.config.appearance.theme = theme
      this.saveConfig()
    },
  },
})
```

- [ ] **Step 4: Create src/stores/tabStore.ts**

```typescript
import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import type { FileStatus, LogEntry, DirEntry, ShelveInfo, ActiveView } from '../types/svn'

export const useTabStore = (id: string) =>
  defineStore(`tab-${id}`, {
    state: () => ({
      repoPath: '',
      activeView: 'log' as ActiveView,
      logEntries: [] as LogEntry[],
      fileTree: [] as DirEntry[],
      localChanges: [] as FileStatus[],
      shelves: [] as ShelveInfo[],
      logPage: 1,
      hasMoreLogs: true,
      loading: false,
    }),
    actions: {
      async refreshLog(limit?: number) {
        this.loading = true
        try {
          this.logEntries = await invoke<LogEntry[]>('svn_log', {
            path: this.repoPath,
            limit: limit ?? 100,
          })
        } catch (e) {
          console.error('Failed to refresh log:', e)
        } finally {
          this.loading = false
        }
      },
      async refreshLocalChanges() {
        this.loading = true
        try {
          this.localChanges = await invoke<FileStatus[]>('svn_status', {
            path: this.repoPath,
          })
        } catch (e) {
          console.error('Failed to refresh local changes:', e)
        } finally {
          this.loading = false
        }
      },
      async refreshFileBrowser(path?: string) {
        this.loading = true
        try {
          this.fileTree = await invoke<DirEntry[]>('svn_list', {
            path: path ?? this.repoPath,
            recursive: false,
          })
        } catch (e) {
          console.error('Failed to refresh file browser:', e)
        } finally {
          this.loading = false
        }
      },
      async refreshShelves() {
        this.loading = true
        try {
          this.shelves = await invoke<ShelveInfo[]>('shelve_list', {
            path: this.repoPath,
          })
        } catch (e) {
          console.error('Failed to refresh shelves:', e)
        } finally {
          this.loading = false
        }
      },
    },
  })
```

- [ ] **Step 5: Commit**

```bash
git add src/types/ src/stores/
git commit -m "feat(frontend): add TypeScript types and Pinia stores"
```

---

## Task 11: App.vue + GlobalTabBar + IconNavBar + Toolbar

**Files:**
- Modify: `src/App.vue`
- Create: `src/components/GlobalTabBar.vue`
- Create: `src/components/IconNavBar.vue`
- Create: `src/components/Toolbar.vue`

- [ ] **Step 1: Create GlobalTabBar.vue**

```vue
<template>
  <div class="tab-bar">
    <button class="settings-btn" @click="$emit('openSettings')" title="设置">⚙</button>
    <div class="tabs">
      <div
        v-for="(tab, index) in tabs"
        :key="index"
        class="tab"
        :class="{ active: index === activeTabIndex }"
        @click="$emit('switchTab', index)"
        @dblclick="$emit('closeTab', index)"
      >
        <span class="tab-title">{{ getTabTitle(tab.repoPath) }}</span>
        <button class="tab-close" @click.stop="$emit('closeTab', index)">×</button>
      </div>
    </div>
    <button class="add-tab-btn" @click="$emit('addTab')" title="打开仓库">+ 新页签</button>
  </div>
</template>

<script setup lang="ts">
import type { TabInfo } from '../types/config'

defineProps<{
  tabs: TabInfo[]
  activeTabIndex: number
}>()

defineEmits<{
  openSettings: []
  switchTab: [index: number]
  closeTab: [index: number]
  addTab: []
}>()

function getTabTitle(path: string) {
  return path.split(/[/\\]/).pop() || path
}
</script>

<style scoped>
.tab-bar {
  display: flex;
  align-items: center;
  height: 36px;
  background: #f0f0f0;
  border-bottom: 1px solid #ddd;
  padding: 0 4px;
  gap: 2px;
}
.settings-btn {
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  cursor: pointer;
  font-size: 16px;
  border-radius: 4px;
}
.settings-btn:hover {
  background: #e0e0e0;
}
.tabs {
  display: flex;
  flex: 1;
  overflow-x: auto;
  gap: 2px;
}
.tab {
  display: flex;
  align-items: center;
  padding: 4px 8px;
  background: #e8e8e8;
  border-radius: 4px 4px 0 0;
  cursor: pointer;
  white-space: nowrap;
  font-size: 12px;
}
.tab.active {
  background: #fff;
  border-bottom: 2px solid #1890ff;
}
.tab-close {
  margin-left: 4px;
  border: none;
  background: transparent;
  cursor: pointer;
  font-size: 14px;
  color: #999;
}
.tab-close:hover {
  color: #333;
}
.add-tab-btn {
  border: 1px dashed #999;
  background: transparent;
  padding: 4px 8px;
  cursor: pointer;
  font-size: 12px;
  border-radius: 4px;
}
.add-tab-btn:hover {
  background: #e0e0e0;
}
</style>
```

- [ ] **Step 2: Create IconNavBar.vue**

```vue
<template>
  <div class="icon-nav-bar">
    <button
      v-for="item in navItems"
      :key="item.view"
      class="nav-item"
      :class="{ active: activeView === item.view }"
      @click="$emit('switchView', item.view)"
      :title="item.label"
    >
      <span class="nav-icon">{{ item.icon }}</span>
    </button>
  </div>
</template>

<script setup lang="ts">
import type { ActiveView } from '../types/svn'

defineProps<{
  activeView: ActiveView
}>()

defineEmits<{
  switchView: [view: ActiveView]
}>()

const navItems = [
  { view: 'log' as ActiveView, icon: '📋', label: '日志' },
  { view: 'localChanges' as ActiveView, icon: '📝', label: '本地修改' },
  { view: 'fileBrowser' as ActiveView, icon: '📂', label: '文件浏览' },
  { view: 'shelve' as ActiveView, icon: '📦', label: 'Shelve' },
]
</script>

<style scoped>
.icon-nav-bar {
  display: flex;
  flex-direction: column;
  width: 48px;
  background: #fafafa;
  border-right: 1px solid #e8e8e8;
  padding: 8px 0;
}
.nav-item {
  width: 48px;
  height: 48px;
  border: none;
  background: transparent;
  cursor: pointer;
  font-size: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
}
.nav-item:hover {
  background: #f0f0f0;
}
.nav-item.active {
  background: #e6f7ff;
  border-left: 3px solid #1890ff;
}
</style>
```

- [ ] **Step 3: Create Toolbar.vue**

```vue
<template>
  <div class="toolbar">
    <button @click="$emit('pull')" :disabled="loading">拉取</button>
    <button @click="$emit('commit')" :disabled="loading">提交</button>
    <button @click="$emit('refresh')" :disabled="loading">刷新</button>
    <span v-if="loading" class="loading-indicator">处理中...</span>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  loading: boolean
}>()

defineEmits<{
  pull: []
  commit: []
  refresh: []
}>()
</script>

<style scoped>
.toolbar {
  display: flex;
  align-items: center;
  height: 32px;
  background: #fff;
  border-bottom: 1px solid #e8e8e8;
  padding: 0 8px;
  gap: 4px;
}
.toolbar button {
  padding: 4px 12px;
  border: 1px solid #d9d9d9;
  background: #fff;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
}
.toolbar button:hover:not(:disabled) {
  border-color: #1890ff;
  color: #1890ff;
}
.toolbar button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.loading-indicator {
  margin-left: auto;
  font-size: 12px;
  color: #999;
}
</style>
```

- [ ] **Step 4: Update src/main.ts**

```typescript
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'

const app = createApp(App)
app.use(createPinia())
app.mount('#app')
```

- [ ] **Step 5: Update src/App.vue**

```vue
<template>
  <div class="app-container">
    <GlobalTabBar
      :tabs="tabs"
      :activeTabIndex="activeTabIndex"
      @openSettings="showSettings = true"
      @switchTab="switchTab"
      @closeTab="closeTab"
      @addTab="addTab"
    />
    <Toolbar
      v-if="tabs.length > 0"
      :loading="currentTabStore?.loading ?? false"
      @pull="handlePull"
      @commit="handleCommit"
      @refresh="handleRefresh"
    />
    <div class="main-content" v-if="tabs.length > 0">
      <IconNavBar
        :activeView="currentTabStore?.activeView ?? 'log'"
        @switchView="switchView"
      />
      <div class="view-area">
        <LogView v-if="currentTabStore?.activeView === 'log'" :store="currentTabStore!" />
        <LocalChangesView v-if="currentTabStore?.activeView === 'localChanges'" :store="currentTabStore!" />
        <FileBrowserView v-if="currentTabStore?.activeView === 'fileBrowser'" :store="currentTabStore!" />
        <ShelveView v-if="currentTabStore?.activeView === 'shelve'" :store="currentTabStore!" />
      </div>
    </div>
    <div class="empty-state" v-else>
      <p>点击 "+ 新页签" 打开一个仓库</p>
    </div>
    <SettingsPage v-if="showSettings" @close="showSettings = false" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useConfigStore } from './stores/configStore'
import { useTabStore } from './stores/tabStore'
import type { TabInfo } from './types/config'
import type { ActiveView } from './types/svn'
import GlobalTabBar from './components/GlobalTabBar.vue'
import IconNavBar from './components/IconNavBar.vue'
import Toolbar from './components/Toolbar.vue'
import LogView from './views/LogView.vue'
import LocalChangesView from './views/LocalChangesView.vue'
import FileBrowserView from './views/FileBrowserView.vue'
import ShelveView from './views/ShelveView.vue'
import SettingsPage from './views/SettingsPage.vue'

const configStore = useConfigStore()
const tabs = ref<TabInfo[]>([])
const activeTabIndex = ref(0)
const showSettings = ref(false)
const tabStores = ref<Record<string, ReturnType<typeof useTabStore>>>({})

const currentTabStore = computed(() => {
  if (tabs.value.length === 0) return null
  const tab = tabs.value[activeTabIndex.value]
  if (!tab) return null
  const key = `${activeTabIndex.value}`
  if (!tabStores.value[key]) {
    const store = useTabStore(key)()
    store.repoPath = tab.repoPath
    store.activeView = tab.activeView
    tabStores.value[key] = store
  }
  return tabStores.value[key]
})

onMounted(async () => {
  await configStore.loadConfig()
  const config = configStore.config
  if (config?.session.openTabs) {
    tabs.value = config.session.openTabs
    activeTabIndex.value = config.session.activeTabIndex || 0
  }
})

function switchTab(index: number) {
  activeTabIndex.value = index
}

function closeTab(index: number) {
  const key = `${index}`
  if (tabStores.value[key]) {
    delete tabStores.value[key]
  }
  tabs.value.splice(index, 1)
  if (activeTabIndex.value >= tabs.value.length) {
    activeTabIndex.value = Math.max(0, tabs.value.length - 1)
  }
  saveSession()
}

function addTab() {
  const path = prompt('请输入仓库工作副本路径:')
  if (!path) return
  tabs.value.push({ repoPath: path, activeView: 'log' })
  activeTabIndex.value = tabs.value.length - 1
  saveSession()
}

function switchView(view: ActiveView) {
  if (!currentTabStore.value) return
  currentTabStore.value.activeView = view
  const tab = tabs.value[activeTabIndex.value]
  if (tab) {
    tab.activeView = view
    saveSession()
  }
  refreshCurrentView()
}

function refreshCurrentView() {
  if (!currentTabStore.value) return
  const view = currentTabStore.value.activeView
  if (view === 'log') currentTabStore.value.refreshLog()
  else if (view === 'localChanges') currentTabStore.value.refreshLocalChanges()
  else if (view === 'fileBrowser') currentTabStore.value.refreshFileBrowser()
  else if (view === 'shelve') currentTabStore.value.refreshShelves()
}

function handlePull() {
  if (!currentTabStore.value) return
  invoke('svn_update', { path: currentTabStore.value.repoPath })
    .then(() => refreshCurrentView())
    .catch((e) => console.error('Pull failed:', e))
}

function handleCommit() {
  // TODO: open commit dialog
}

function handleRefresh() {
  refreshCurrentView()
}

function saveSession() {
  if (!configStore.config) return
  configStore.config.session.openTabs = tabs.value
  configStore.config.session.activeTabIndex = activeTabIndex.value
  configStore.saveConfig()
}
</script>

<style scoped>
.app-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
}
.main-content {
  display: flex;
  flex: 1;
  overflow: hidden;
}
.view-area {
  flex: 1;
  overflow: auto;
  padding: 12px;
}
.empty-state {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #999;
}
</style>
```

- [ ] **Step 6: Commit**

```bash
git add src/
git commit -m "feat(frontend): add App.vue, GlobalTabBar, IconNavBar, Toolbar components"
```

---

## Task 12: LogView

**Files:**
- Create: `src/views/LogView.vue`

- [ ] **Step 1: Create LogView.vue**

```vue
<template>
  <div class="log-view">
    <div class="filter-bar">
      <input v-model="searchText" placeholder="搜索提交信息..." class="search-input" />
      <button @click="refresh" class="refresh-btn">↻</button>
    </div>
    <div class="log-table">
      <table>
        <thead>
          <tr>
            <th>版本</th>
            <th>作者</th>
            <th>日期</th>
            <th>提交信息</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="entry in filteredEntries"
            :key="entry.revision"
            @click="toggleDetail(entry.revision)"
            :class="{ expanded: expandedRevision === entry.revision }"
          >
            <td>{{ entry.revision }}</td>
            <td>{{ entry.author }}</td>
            <td>{{ formatDate(entry.date) }}</td>
            <td>{{ entry.message }}</td>
          </tr>
        </tbody>
      </table>
    </div>
    <div v-if="expandedRevision" class="detail-panel">
      <h4>版本 {{ expandedRevision }} 详细信息</h4>
      <p>{{ expandedEntry?.message }}</p>
      <div v-if="expandedEntry?.changedPaths" class="changed-paths">
        <h5>变更文件:</h5>
        <div v-for="cp in expandedEntry.changedPaths" :key="cp.path" class="changed-path">
          <span class="action">{{ cp.action }}</span>
          <span>{{ cp.path }}</span>
        </div>
      </div>
      <div class="detail-actions">
        <button @click="$emit('viewDiff', expandedRevision)">查看差异</button>
        <button @click="$emit('aiReview', expandedRevision)">AI 审查</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import type { LogEntry } from '../types/svn'

const props = defineProps<{
  store: {
    repoPath: string
    logEntries: LogEntry[]
    refreshLog: () => Promise<void>
  }
}>()

defineEmits<{
  viewDiff: [revision: number]
  aiReview: [revision: number]
}>()

const searchText = ref('')
const expandedRevision = ref<number | null>(null)

const filteredEntries = computed(() => {
  if (!searchText.value) return props.store.logEntries
  const text = searchText.value.toLowerCase()
  return props.store.logEntries.filter(
    (e) =>
      e.message.toLowerCase().includes(text) ||
      e.author.toLowerCase().includes(text),
  )
})

const expandedEntry = computed(() => {
  if (!expandedRevision.value) return null
  return props.store.logEntries.find((e) => e.revision === expandedRevision.value)
})

function toggleDetail(revision: number) {
  expandedRevision.value = expandedRevision.value === revision ? null : revision
}

function formatDate(dateStr: string) {
  try {
    return new Date(dateStr).toLocaleDateString('zh-CN')
  } catch {
    return dateStr
  }
}

function refresh() {
  props.store.refreshLog()
}

onMounted(() => {
  if (props.store.logEntries.length === 0) {
    refresh()
  }
})
</script>

<style scoped>
.log-view {
  display: flex;
  flex-direction: column;
  height: 100%;
}
.filter-bar {
  display: flex;
  gap: 8px;
  margin-bottom: 8px;
}
.search-input {
  flex: 1;
  padding: 6px 12px;
  border: 1px solid #d9d9d9;
  border-radius: 4px;
  font-size: 13px;
}
.refresh-btn {
  padding: 6px 12px;
  border: 1px solid #d9d9d9;
  background: #fff;
  border-radius: 4px;
  cursor: pointer;
}
.log-table {
  flex: 1;
  overflow: auto;
}
table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}
th, td {
  padding: 8px 12px;
  text-align: left;
  border-bottom: 1px solid #f0f0f0;
}
th {
  background: #fafafa;
  font-weight: 600;
  position: sticky;
  top: 0;
}
tr:hover {
  background: #f5f5f5;
  cursor: pointer;
}
tr.expanded {
  background: #e6f7ff;
}
.detail-panel {
  border-top: 1px solid #e8e8e8;
  padding: 12px;
  background: #fafafa;
}
.changed-paths {
  margin-top: 8px;
}
.changed-path {
  font-size: 12px;
  padding: 2px 0;
  font-family: monospace;
}
.action {
  display: inline-block;
  width: 20px;
  font-weight: bold;
  color: #1890ff;
}
.detail-actions {
  margin-top: 12px;
  display: flex;
  gap: 8px;
}
.detail-actions button {
  padding: 4px 12px;
  border: 1px solid #d9d9d9;
  background: #fff;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
}
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/views/LogView.vue
git commit -m "feat(frontend): add LogView with filter, table, detail expand"
```

---

## Task 13: LocalChangesView (with integrated commit)

**Files:**
- Create: `src/views/LocalChangesView.vue`

- [ ] **Step 1: Create LocalChangesView.vue**

```vue
<template>
  <div class="local-changes-view">
    <div class="left-panel">
      <div class="file-list-header">
        <label>
          <input type="checkbox" :checked="allSelected" @change="toggleAll" />
          全选
        </label>
        <span class="selected-count">已选 {{ selectedFiles.length }} 个文件</span>
        <button @click="$emit('refresh')" class="refresh-btn">刷新</button>
      </div>
      <div class="file-list">
        <div
          v-for="file in store.localChanges"
          :key="file.path"
          class="file-item"
          :class="{ selected: selectedPaths.has(file.path) }"
          @click="selectFile(file)"
        >
          <input
            type="checkbox"
            :checked="selectedPaths.has(file.path)"
            @click.stop="toggleFile(file.path)"
          />
          <span class="status-badge" :class="file.status">{{ file.status[0].toUpperCase() }}</span>
          <span class="file-path">{{ file.path }}</span>
        </div>
      </div>
      <div class="commit-section">
        <textarea
          v-model="commitMessage"
          placeholder="提交信息..."
          rows="3"
          class="commit-input"
        ></textarea>
        <div class="commit-actions">
          <button @click="generateAiMessage" :disabled="aiLoading" class="ai-btn">
            {{ aiLoading ? '生成中...' : 'AI 生成注释' }}
          </button>
          <button @click="cancelCommit" class="cancel-btn">取消</button>
          <button @click="submitCommit" :disabled="!canCommit" class="commit-btn">提交</button>
        </div>
      </div>
    </div>
    <div class="right-panel">
      <div v-if="diffContent" class="diff-content">
        <pre>{{ diffContent }}</pre>
      </div>
      <div v-else class="diff-placeholder">点击文件查看差异</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { FileStatus } from '../types/svn'

const props = defineProps<{
  store: {
    repoPath: string
    localChanges: FileStatus[]
    refreshLocalChanges: () => Promise<void>
  }
}>()

defineEmits<{
  refresh: []
}>()

const selectedPaths = ref(new Set<string>())
const commitMessage = ref('')
const diffContent = ref('')
const aiLoading = ref(false)

const selectedFiles = computed(() =>
  props.store.localChanges.filter((f) => selectedPaths.value.has(f.path)),
)

const allSelected = computed(
  () =>
    props.store.localChanges.length > 0 &&
    props.store.localChanges.every((f) => selectedPaths.value.has(f.path)),
)

const canCommit = computed(
  () => selectedPaths.value.size > 0 && commitMessage.value.trim().length > 0,
)

function toggleAll() {
  if (allSelected.value) {
    selectedPaths.value.clear()
  } else {
    props.store.localChanges.forEach((f) => selectedPaths.value.add(f.path))
  }
}

function toggleFile(path: string) {
  if (selectedPaths.value.has(path)) {
    selectedPaths.value.delete(path)
  } else {
    selectedPaths.value.add(path)
  }
}

async function selectFile(file: FileStatus) {
  try {
    diffContent.value = await invoke<string>('svn_diff', {
      path: props.store.repoPath,
      target: { type: 'File', data: { path: file.path } },
    })
  } catch (e) {
    diffContent.value = `获取差异失败: ${e}`
  }
}

async function generateAiMessage() {
  if (selectedPaths.value.size === 0) return
  aiLoading.value = true
  try {
    const diff = await invoke<string>('svn_diff', {
      path: props.store.repoPath,
      target: { type: 'File', data: { path: Array.from(selectedPaths.value)[0] } },
    })
    commitMessage.value = await invoke<string>('generate_commit_message', { diff })
  } catch (e) {
    console.error('AI generation failed:', e)
  } finally {
    aiLoading.value = false
  }
}

async function submitCommit() {
  if (!canCommit.value) return
  try {
    await invoke('svn_commit', {
      path: props.store.repoPath,
      message: commitMessage.value,
      files: Array.from(selectedPaths.value),
    })
    commitMessage.value = ''
    selectedPaths.value.clear()
    await props.store.refreshLocalChanges()
  } catch (e) {
    console.error('Commit failed:', e)
  }
}

function cancelCommit() {
  commitMessage.value = ''
  selectedPaths.value.clear()
}

onMounted(() => {
  props.store.refreshLocalChanges()
})
</script>

<style scoped>
.local-changes-view {
  display: flex;
  height: 100%;
  gap: 12px;
}
.left-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
}
.right-panel {
  flex: 1;
  border-left: 1px solid #e8e8e8;
  padding-left: 12px;
}
.file-list-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 8px;
  font-size: 13px;
}
.file-list {
  flex: 1;
  overflow: auto;
  border: 1px solid #e8e8e8;
  border-radius: 4px;
}
.file-item {
  display: flex;
  align-items: center;
  padding: 6px 8px;
  gap: 8px;
  cursor: pointer;
  font-size: 13px;
  border-bottom: 1px solid #f0f0f0;
}
.file-item:hover {
  background: #f5f5f5;
}
.file-item.selected {
  background: #e6f7ff;
}
.status-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  border-radius: 3px;
  font-size: 11px;
  font-weight: bold;
  color: #fff;
}
.status-badge.modified { background: #faad14; }
.status-badge.added { background: #52c41a; }
.status-badge.deleted { background: #ff4d4f; }
.status-badge.unversioned { background: #999; }
.status-badge.missing { background: #ff7a45; }
.status-badge.conflicted { background: #f5222d; }
.commit-section {
  margin-top: 12px;
}
.commit-input {
  width: 100%;
  padding: 8px;
  border: 1px solid #d9d9d9;
  border-radius: 4px;
  font-size: 13px;
  resize: vertical;
  font-family: inherit;
}
.commit-actions {
  display: flex;
  gap: 8px;
  margin-top: 8px;
  justify-content: flex-end;
}
.commit-actions button {
  padding: 6px 16px;
  border: 1px solid #d9d9d9;
  background: #fff;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
}
.commit-btn {
  background: #1890ff !important;
  color: #fff !important;
  border-color: #1890ff !important;
}
.commit-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.ai-btn {
  margin-right: auto;
}
.diff-content pre {
  font-family: monospace;
  font-size: 13px;
  white-space: pre-wrap;
  word-break: break-all;
}
.diff-placeholder {
  color: #999;
  text-align: center;
  margin-top: 40px;
}
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/views/LocalChangesView.vue
git commit -m "feat(frontend): add LocalChangesView with commit and AI message generation"
```

---

## Task 14: FileBrowserView + ShelveView

**Files:**
- Create: `src/views/FileBrowserView.vue`
- Create: `src/views/ShelveView.vue`

- [ ] **Step 1: Create FileBrowserView.vue**

```vue
<template>
  <div class="file-browser-view">
    <div class="browser-header">
      <button @click="refresh" class="refresh-btn">刷新</button>
    </div>
    <div class="browser-content">
      <div class="tree-panel">
        <div
          v-for="entry in store.fileTree"
          :key="entry.name"
          class="tree-item"
          @click="onEntryClick(entry)"
        >
          <span class="entry-icon">{{ entry.kind === 'dir' ? '📁' : '📄' }}</span>
          <span class="entry-name">{{ entry.name }}</span>
        </div>
      </div>
      <div class="content-panel">
        <pre v-if="fileContent" class="file-content">{{ fileContent }}</pre>
        <div v-else class="content-placeholder">点击文件查看内容</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { DirEntry } from '../types/svn'

const props = defineProps<{
  store: {
    repoPath: string
    fileTree: DirEntry[]
    refreshFileBrowser: (path?: string) => Promise<void>
  }
}>()

const fileContent = ref('')
const currentPath = ref('')

async function onEntryClick(entry: DirEntry) {
  if (entry.kind === 'dir') {
    const dirPath = currentPath.value
      ? `${currentPath.value}/${entry.name}`
      : entry.name
    currentPath.value = dirPath
    await props.store.refreshFileBrowser(`${props.store.repoPath}/${dirPath}`)
  } else {
    const filePath = currentPath.value
      ? `${currentPath.value}/${entry.name}`
      : entry.name
    try {
      fileContent.value = await invoke<string>('svn_cat', {
        path: `${props.store.repoPath}/${filePath}`,
      })
    } catch (e) {
      fileContent.value = `读取失败: ${e}`
    }
  }
}

function refresh() {
  fileContent.value = ''
  currentPath.value = ''
  props.store.refreshFileBrowser()
}

onMounted(() => {
  props.store.refreshFileBrowser()
})
</script>

<style scoped>
.file-browser-view {
  display: flex;
  flex-direction: column;
  height: 100%;
}
.browser-header {
  margin-bottom: 8px;
}
.browser-content {
  display: flex;
  flex: 1;
  gap: 12px;
}
.tree-panel {
  width: 250px;
  border: 1px solid #e8e8e8;
  border-radius: 4px;
  overflow: auto;
}
.tree-item {
  display: flex;
  align-items: center;
  padding: 6px 8px;
  gap: 8px;
  cursor: pointer;
  font-size: 13px;
}
.tree-item:hover {
  background: #f5f5f5;
}
.content-panel {
  flex: 1;
  border: 1px solid #e8e8e8;
  border-radius: 4px;
  overflow: auto;
  padding: 12px;
}
.file-content {
  font-family: monospace;
  font-size: 13px;
  white-space: pre-wrap;
}
.content-placeholder {
  color: #999;
  text-align: center;
  margin-top: 40px;
}
</style>
```

- [ ] **Step 2: Create ShelveView.vue**

```vue
<template>
  <div class="shelve-view">
    <div class="shelve-header">
      <button @click="showSaveDialog = true" class="save-btn">保存当前修改</button>
      <button @click="refresh" class="refresh-btn">刷新</button>
    </div>
    <div class="shelve-list">
      <table>
        <thead>
          <tr>
            <th>名称</th>
            <th>日期</th>
            <th>操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="shelve in store.shelves" :key="shelve.name">
            <td>{{ shelve.name }}</td>
            <td>{{ formatDate(shelve.date) }}</td>
            <td>
              <button @click="applyShelve(shelve.name)" class="action-btn">应用</button>
              <button @click="deleteShelve(shelve.name)" class="action-btn delete">删除</button>
            </td>
          </tr>
        </tbody>
      </table>
      <div v-if="store.shelves.length === 0" class="empty">暂无 Shelve</div>
    </div>
    <div v-if="showSaveDialog" class="dialog-overlay">
      <div class="dialog">
        <h3>保存 Shelve</h3>
        <input v-model="shelveName" placeholder="名称" class="dialog-input" />
        <div class="dialog-actions">
          <button @click="showSaveDialog = false" class="cancel-btn">取消</button>
          <button @click="saveShelve" :disabled="!shelveName.trim()" class="save-btn">保存</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps<{
  store: {
    repoPath: string
    shelves: { name: string; date: string }[]
    refreshShelves: () => Promise<void>
  }
}>()

const showSaveDialog = ref(false)
const shelveName = ref('')

function formatDate(dateStr: string) {
  try {
    return new Date(dateStr).toLocaleDateString('zh-CN')
  } catch {
    return dateStr
  }
}

async function saveShelve() {
  if (!shelveName.value.trim()) return
  try {
    await invoke('shelve_save', {
      path: props.store.repoPath,
      name: shelveName.value.trim(),
    })
    showSaveDialog.value = false
    shelveName.value = ''
    await props.store.refreshShelves()
  } catch (e) {
    console.error('Save shelve failed:', e)
  }
}

async function applyShelve(name: string) {
  try {
    await invoke('shelve_apply', {
      path: props.store.repoPath,
      name,
    })
    await props.store.refreshShelves()
  } catch (e) {
    console.error('Apply shelve failed:', e)
  }
}

async function deleteShelve(name: string) {
  if (!confirm(`确定要删除 '${name}' 吗？`)) return
  try {
    await invoke('shelve_delete', {
      path: props.store.repoPath,
      name,
    })
    await props.store.refreshShelves()
  } catch (e) {
    console.error('Delete shelve failed:', e)
  }
}

function refresh() {
  props.store.refreshShelves()
}

onMounted(() => {
  props.store.refreshShelves()
})
</script>

<style scoped>
.shelve-view {
  display: flex;
  flex-direction: column;
  height: 100%;
}
.shelve-header {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
}
.shelve-header button {
  padding: 6px 12px;
  border: 1px solid #d9d9d9;
  background: #fff;
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
}
.save-btn {
  background: #1890ff !important;
  color: #fff !important;
  border-color: #1890ff !important;
}
.shelve-list {
  flex: 1;
  overflow: auto;
}
table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}
th, td {
  padding: 8px 12px;
  text-align: left;
  border-bottom: 1px solid #f0f0f0;
}
th {
  background: #fafafa;
  font-weight: 600;
}
.action-btn {
  padding: 4px 8px;
  border: 1px solid #d9d9d9;
  background: #fff;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  margin-right: 4px;
}
.action-btn.delete {
  color: #ff4d4f;
  border-color: #ff4d4f;
}
.empty {
  color: #999;
  text-align: center;
  margin-top: 40px;
}
.dialog-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0,0,0,0.3);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}
.dialog {
  background: #fff;
  padding: 20px;
  border-radius: 8px;
  min-width: 300px;
}
.dialog h3 {
  margin: 0 0 12px;
}
.dialog-input {
  width: 100%;
  padding: 8px;
  border: 1px solid #d9d9d9;
  border-radius: 4px;
  font-size: 13px;
  box-sizing: border-box;
}
.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 12px;
}
.cancel-btn {
  padding: 6px 12px;
  border: 1px solid #d9d9d9;
  background: #fff;
  border-radius: 4px;
  cursor: pointer;
}
</style>
```

- [ ] **Step 3: Commit**

```bash
git add src/views/
git commit -m "feat(frontend): add FileBrowserView and ShelveView"
```

---

## Task 15: DiffViewer + AiReviewPanel + SettingsPage

**Files:**
- Create: `src/components/DiffViewer.vue`
- Create: `src/components/AiReviewPanel.vue`
- Create: `src/views/SettingsPage.vue`

- [ ] **Step 1: Create DiffViewer.vue**

```vue
<template>
  <div v-if="visible" class="diff-overlay" @click.self="$emit('close')">
    <div class="diff-modal">
      <div class="diff-header">
        <span>文件: {{ filePath }}</span>
        <div class="diff-mode-toggle">
          <button :class="{ active: mode === 'unified' }" @click="mode = 'unified'">统一视图</button>
          <button :class="{ active: mode === 'side_by_side' }" @click="mode = 'side_by_side'">并排视图</button>
        </div>
        <button class="close-btn" @click="$emit('close')">×</button>
      </div>
      <div class="diff-content">
        <pre>{{ diffText }}</pre>
      </div>
      <div class="diff-footer">
        <button @click="copyDiff">复制差异</button>
        <button @click="$emit('aiReview', diffText)">AI 审查</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

defineProps<{
  visible: boolean
  filePath: string
  diffText: string
}>()

defineEmits<{
  close: []
  aiReview: [diff: string]
}>()

const mode = ref<'unified' | 'side_by_side'>('unified')

function copyDiff() {
  // TODO: use clipboard API
}
</script>

<style scoped>
.diff-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0,0,0,0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 200;
}
.diff-modal {
  background: #fff;
  border-radius: 8px;
  width: 80%;
  height: 80%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
.diff-header {
  display: flex;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid #e8e8e8;
  gap: 12px;
}
.diff-mode-toggle {
  display: flex;
  gap: 4px;
}
.diff-mode-toggle button {
  padding: 4px 8px;
  border: 1px solid #d9d9d9;
  background: #fff;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
}
.diff-mode-toggle button.active {
  background: #1890ff;
  color: #fff;
  border-color: #1890ff;
}
.close-btn {
  margin-left: auto;
  border: none;
  background: transparent;
  font-size: 20px;
  cursor: pointer;
}
.diff-content {
  flex: 1;
  overflow: auto;
  padding: 16px;
}
.diff-content pre {
  font-family: monospace;
  font-size: 13px;
  white-space: pre-wrap;
}
.diff-footer {
  display: flex;
  gap: 8px;
  padding: 12px 16px;
  border-top: 1px solid #e8e8e8;
}
.diff-footer button {
  padding: 6px 12px;
  border: 1px solid #d9d9d9;
  background: #fff;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
}
</style>
```

- [ ] **Step 2: Create AiReviewPanel.vue**

```vue
<template>
  <div v-if="visible" class="ai-panel">
    <div class="panel-header">
      <span>AI 代码审查</span>
      <button @click="$emit('close')">×</button>
    </div>
    <div class="panel-content">
      <div v-if="loading" class="loading">AI 正在审查变更...</div>
      <div v-else class="review-text">{{ content }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  visible: boolean
  content: string
  loading: boolean
}>()

defineEmits<{
  close: []
}>()
</script>

<style scoped>
.ai-panel {
  width: 350px;
  border-left: 1px solid #e8e8e8;
  display: flex;
  flex-direction: column;
  background: #fafafa;
}
.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px;
  border-bottom: 1px solid #e8e8e8;
  font-weight: 600;
}
.panel-header button {
  border: none;
  background: transparent;
  font-size: 18px;
  cursor: pointer;
}
.panel-content {
  flex: 1;
  overflow: auto;
  padding: 12px;
  font-size: 13px;
  line-height: 1.6;
}
.loading {
  color: #1890ff;
}
</style>
```

- [ ] **Step 3: Create SettingsPage.vue**

```vue
<template>
  <div v-if="true" class="settings-overlay" @click.self="$emit('close')">
    <div class="settings-modal">
      <div class="settings-header">
        <h3>设置</h3>
        <button @click="$emit('close')">×</button>
      </div>
      <div class="settings-body">
        <div class="settings-tabs">
          <button
            v-for="tab in tabs"
            :key="tab.key"
            :class="{ active: activeTab === tab.key }"
            @click="activeTab = tab.key"
          >
            {{ tab.label }}
          </button>
        </div>
        <div class="settings-content">
          <div v-if="activeTab === 'general'">
            <label>主题</label>
            <select v-model="config.appearance.theme">
              <option value="light">亮色</option>
              <option value="dark">深色</option>
            </select>
            <label>UI 字体大小</label>
            <input type="number" v-model.number="config.appearance.uiFontSize" />
            <label>代码字体</label>
            <input v-model="config.appearance.codeFontFamily" />
          </div>
          <div v-if="activeTab === 'svn'">
            <label>SVN 可执行文件路径</label>
            <input v-model="config.svn.executable" placeholder="自动检测" />
          </div>
          <div v-if="activeTab === 'ai'">
            <label>API 端点</label>
            <input v-model="config.ai.endpoint" />
            <label>API 密钥</label>
            <input v-model="config.ai.apiKey" type="password" />
            <label>模型</label>
            <input v-model="config.ai.model" />
            <label>超时（秒）</label>
            <input type="number" v-model.number="config.ai.timeoutSecs" />
          </div>
          <div v-if="activeTab === 'advanced'">
            <label>SVN 超时（秒）</label>
            <input type="number" v-model.number="config.advanced.svnTimeoutSecs" />
            <label>日志级别</label>
            <select v-model="config.advanced.logLevel">
              <option value="error">Error</option>
              <option value="warn">Warn</option>
              <option value="info">Info</option>
              <option value="debug">Debug</option>
            </select>
          </div>
        </div>
      </div>
      <div class="settings-footer">
        <button @click="$emit('close')" class="cancel-btn">取消</button>
        <button @click="save" class="save-btn">保存</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { useConfigStore } from '../stores/configStore'

defineEmits<{
  close: []
}>()

const configStore = useConfigStore()
const activeTab = ref('general')

const tabs = [
  { key: 'general', label: '通用' },
  { key: 'svn', label: 'SVN' },
  { key: 'ai', label: 'AI' },
  { key: 'advanced', label: '高级' },
]

const config = reactive({
  appearance: { theme: 'light', uiFontSize: 14, codeFontFamily: 'monospace' },
  svn: { executable: '' },
  ai: { endpoint: 'https://api.openai.com/v1', apiKey: '', model: 'gpt-4o-mini', timeoutSecs: 30 },
  advanced: { svnTimeoutSecs: 60, logLevel: 'warn' },
})

onMounted(() => {
  if (configStore.config) {
    Object.assign(config.appearance, configStore.config.appearance)
    Object.assign(config.svn, configStore.config.svn)
    Object.assign(config.ai, configStore.config.ai)
    Object.assign(config.advanced, configStore.config.advanced)
  }
})

function save() {
  if (!configStore.config) return
  Object.assign(configStore.config.appearance, config.appearance)
  Object.assign(configStore.config.svn, config.svn)
  Object.assign(configStore.config.ai, config.ai)
  Object.assign(configStore.config.advanced, config.advanced)
  configStore.saveConfig()
}
</script>

<style scoped>
.settings-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0,0,0,0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 300;
}
.settings-modal {
  background: #fff;
  border-radius: 8px;
  width: 600px;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
}
.settings-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px;
  border-bottom: 1px solid #e8e8e8;
}
.settings-header h3 { margin: 0; }
.settings-header button {
  border: none;
  background: transparent;
  font-size: 20px;
  cursor: pointer;
}
.settings-body {
  display: flex;
  flex: 1;
  overflow: hidden;
}
.settings-tabs {
  width: 120px;
  border-right: 1px solid #e8e8e8;
  padding: 8px 0;
}
.settings-tabs button {
  display: block;
  width: 100%;
  padding: 8px 16px;
  border: none;
  background: transparent;
  text-align: left;
  cursor: pointer;
  font-size: 13px;
}
.settings-tabs button.active {
  background: #e6f7ff;
  color: #1890ff;
}
.settings-content {
  flex: 1;
  padding: 16px;
  overflow: auto;
}
.settings-content label {
  display: block;
  margin-top: 12px;
  margin-bottom: 4px;
  font-size: 13px;
  font-weight: 500;
}
.settings-content input,
.settings-content select {
  width: 100%;
  padding: 6px 8px;
  border: 1px solid #d9d9d9;
  border-radius: 4px;
  font-size: 13px;
  box-sizing: border-box;
}
.settings-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 12px 16px;
  border-top: 1px solid #e8e8e8;
}
.settings-footer button {
  padding: 6px 16px;
  border: 1px solid #d9d9d9;
  background: #fff;
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
}
.save-btn {
  background: #1890ff !important;
  color: #fff !important;
  border-color: #1890ff !important;
}
</style>
```

- [ ] **Step 4: Commit**

```bash
git add src/components/ src/views/SettingsPage.vue
git commit -m "feat(frontend): add DiffViewer, AiReviewPanel, and SettingsPage"
```

---

## Task 16: Update App.vue with DiffViewer and AiReviewPanel Integration

**Files:**
- Modify: `src/App.vue`

- [ ] **Step 1: Add DiffViewer and AiReviewPanel to App.vue**

Add imports and state to the existing App.vue:

```typescript
import DiffViewer from './components/DiffViewer.vue'
import AiReviewPanel from './components/AiReviewPanel.vue'

// Add to state:
const showDiff = ref(false)
const diffFilePath = ref('')
const diffText = ref('')
const showAiReview = ref(false)
const aiReviewContent = ref('')
const aiReviewLoading = ref(false)
```

Add to template (before closing `</template>`):

```html
<DiffViewer
  :visible="showDiff"
  :filePath="diffFilePath"
  :diffText="diffText"
  @close="showDiff = false"
  @aiReview="handleAiReview"
/>
<AiReviewPanel
  :visible="showAiReview"
  :content="aiReviewContent"
  :loading="aiReviewLoading"
  @close="showAiReview = false"
/>
```

Add handler functions:

```typescript
async function handleAiReview(diff: string) {
  showAiReview.value = true
  aiReviewContent.value = ''
  aiReviewLoading.value = true
  try {
    const { listen } = await import('@tauri-apps/api/event')
    const unlisten = await listen<{ content: string; done: boolean }>('review_chunk', (event) => {
      aiReviewContent.value += event.payload.content
      if (event.payload.done) {
        aiReviewLoading.value = false
        unlisten()
      }
    })
    await invoke('review_changes', { diff })
  } catch (e) {
    aiReviewContent.value = `AI 审查失败: ${e}`
    aiReviewLoading.value = false
  }
}
```

- [ ] **Step 2: Commit**

```bash
git add src/App.vue
git commit -m "feat(frontend): integrate DiffViewer and AiReviewPanel in App.vue"
```

---

## Task 17: Final Integration and Build Verification

- [ ] **Step 1: Verify frontend compiles**

```bash
cd D:/study/github/SourceSvn
pnpm build
```

Expected: Build succeeds with no errors.

- [ ] **Step 2: Verify backend compiles**

```bash
cargo build --manifest-path src-tauri/Cargo.toml
```

Expected: Build succeeds.

- [ ] **Step 3: Run all Rust tests**

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: All tests pass.

- [ ] **Step 4: Start dev mode and verify**

```bash
pnpm tauri dev
```

Expected: Application window opens, UI renders correctly.

- [ ] **Step 5: Final commit**

```bash
git add -A
git commit -m "feat: complete MVP implementation with all views and backend modules"
```

---

## Summary

| Task | Description | Files Created/Modified |
|------|-------------|----------------------|
| 1 | Project scaffolding | package.json, vite.config.ts, .gitignore, etc. |
| 2 | Backend common types | src-tauri/src/common/mod.rs |
| 3 | Config module | src-tauri/src/config/mod.rs |
| 4 | SVN status parser | src-tauri/src/svn/mod.rs, status.rs |
| 5 | SVN log parser | src-tauri/src/svn/log.rs |
| 6 | SVN diff/commit/list/cat | src-tauri/src/svn/diff.rs, commit.rs |
| 7 | AI service | src-tauri/src/ai/mod.rs, openai.rs |
| 8 | Shelve module | src-tauri/src/shelve/mod.rs |
| 9 | Tauri commands | src-tauri/src/commands/*.rs, main.rs |
| 10 | Frontend types/stores | src/types/*.ts, src/stores/*.ts |
| 11 | App + layout components | App.vue, GlobalTabBar, IconNavBar, Toolbar |
| 12 | LogView | src/views/LogView.vue |
| 13 | LocalChangesView | src/views/LocalChangesView.vue |
| 14 | FileBrowser + Shelve views | src/views/FileBrowserView.vue, ShelveView.vue |
| 15 | DiffViewer + AiReview + Settings | src/components/*.vue, src/views/SettingsPage.vue |
| 16 | Integration | src/App.vue modifications |
| 17 | Build verification | All files |
