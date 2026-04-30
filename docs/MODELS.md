# 数据模型定义

本文档定义前后端共享的核心数据结构。Rust 端使用 `serde` 序列化，TypeScript 前端使用对应接口。

## 共享基础类型

```rust
// Rust (common/src/lib.rs)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileStatus {
    pub path: String,
    pub status: FileStatusType,
    pub is_directory: bool,
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
    pub date: String,  // ISO 8601: "2026-04-30T10:30:00Z"
    pub message: String,
    pub changed_paths: Option<Vec<ChangedPath>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChangedPath {
    pub path: String,
    pub action: PathAction,
    pub copy_from_path: Option<String>,
    pub copy_from_rev: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum PathAction {
    A,  // Added
    M,  // Modified
    D,  // Deleted
    R,  // Replaced
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
    File { path: String, revision: Option<String> },
    Revisions { old_rev: String, new_rev: String },
}
```

## 前端 TypeScript 定义

```typescript
// src/types/svn.ts
export type FileStatusType = 'modified' | 'added' | 'deleted' | 'unversioned' | 'missing' | 'conflicted';

export interface FileStatus {
  path: string;
  status: FileStatusType;
  isDirectory: boolean;
  copied?: boolean;
}

export interface LogEntry {
  revision: number;
  author: string;
  date: string;
  message: string;
  changedPaths?: ChangedPath[];
}

export interface ChangedPath {
  path: string;
  action: 'A' | 'M' | 'D' | 'R';
  copyFromPath?: string;
  copyFromRev?: number;
}

export interface RepoInfo {
  url: string;
  root: string;
  revision: number;
  lastChangedRev: number;
  lastChangedDate: string;
  lastChangedAuthor: string;
}

export interface DirEntry {
  name: string;
  kind: 'file' | 'dir';
  size?: number;
  revision: number;
  author: string;
  date: string;
}

export interface ShelveInfo {
  name: string;
  date: string;
}

export type DiffTarget =
  | { type: 'File'; filePath: string; revision?: string }
  | { type: 'Revisions'; oldRev: string; newRev: string };

export interface CommitResult {
  revision: number;
  success: boolean;
  errors?: string[];
}

export interface UpdateResult {
  revision: number;
  updatedFiles: string[];
  mergedFiles: string[];
  conflicts: string[];
}
```

## 配置模型

### Rust 端

```rust
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
    pub theme: String, // "light" | "dark"
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
    pub provider: String,  // "openai" | "local" | "claude" (MVP: "openai")
    pub endpoint: String,
    pub api_key: String,
    pub model: String,
    pub timeout_secs: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DiffConfig {
    pub context_lines: u32,
    pub ignore_whitespace: bool,
    pub view_mode: String, // "unified" | "side_by_side"
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
    pub log_level: String, // "error" | "warn" | "info" | "debug"
}
```

### TypeScript 对应

```typescript
// src/types/config.ts
export interface AppConfig {
  configVersion: number;
  window: WindowConfig;
  appearance: AppearanceConfig;
  session: SessionConfig;
  svn: SvnConfig;
  ai: AiConfig;
  diff: DiffConfig;
  log: LogConfig;
  commit: CommitConfig;
  fileBrowser: FileBrowserConfig;
  behavior: BehaviorConfig;
  advanced: AdvancedConfig;
}

export interface WindowConfig {
  width: number;
  height: number;
  x?: number;
  y?: number;
  maximized: boolean;
}

export interface AppearanceConfig {
  theme: 'light' | 'dark';
  uiFontFamily: string;
  uiFontSize: number;
  codeFontFamily: string;
  codeFontSize: number;
  iconSize: number;
}

export interface SessionConfig {
  openTabs: TabInfo[];
  activeTabIndex: number;
  recentRepos: RepoEntry[];
  maxRecentRepos: number;
}

export interface TabInfo {
  repoPath: string;
  activeView: 'log' | 'localChanges' | 'fileBrowser' | 'shelve';
}

export interface RepoEntry {
  path: string;
  lastOpened: string;
}

export interface SvnConfig {
  executable?: string;
}

export interface AiConfig {
  provider: string;
  endpoint: string;
  apiKey: string;
  model: string;
  timeoutSecs: number;
}

export interface DiffConfig {
  contextLines: number;
  ignoreWhitespace: boolean;
  viewMode: 'unified' | 'side_by_side';
}

export interface LogConfig {
  fetchLimit: number;
  showChangedPaths: boolean;
}

export interface CommitConfig {
  template?: string;
}

export interface FileBrowserConfig {
  showHidden: boolean;
}

export interface BehaviorConfig {
  confirmBeforeCommit: boolean;
  confirmBeforeRevert: boolean;
  autoRefreshSecs?: number;
}

export interface AdvancedConfig {
  svnTimeoutSecs: number;
  logLevel: 'error' | 'warn' | 'info' | 'debug';
}
```

## 状态管理 Store 定义 (Pinia)

```typescript
// src/stores/tabStore.ts
import { defineStore } from 'pinia';

export const useTabStore = defineStore(`tab-${id}`, {
  state: () => ({
    repoPath: '' as string,
    activeView: 'log' as ActiveView,
    logEntries: [] as LogEntry[],
    fileTree: [] as DirEntry[],
    localChanges: [] as FileStatus[],
    shelves: [] as ShelveInfo[],
    // 缓存和分页状态
    logPage: 1,
    hasMoreLogs: true,
  }),
  actions: {
    async refreshLog(limit?: number) { /* invoke svn_log */ },
    async refreshLocalChanges() { /* invoke svn_status */ },
    async refreshFileBrowser(path?: string) { /* invoke svn_list */ },
    async refreshShelves() { /* invoke shelve_list */ },
  },
});

// src/stores/configStore.ts
export const useConfigStore = defineStore('config', {
  state: () => ({
    config: null as AppConfig | null,
  }),
  actions: {
    async loadConfig() { /* invoke get_config */ },
    async saveConfig() { /* invoke set_config */ },
    updateTheme(theme: 'light' | 'dark') { /* mutation and save */ },
  },
});
```
