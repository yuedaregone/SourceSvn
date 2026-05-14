# Hook机制实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现基于事件总线的hook机制，支持异步执行、通知/日志记录和拦截/修改操作

**Architecture:** 使用事件总线架构，通过trait定义接口，支持多种hook类型和脚本执行环境

**Tech Stack:** Rust (后端), Vue 3 (前端), Tauri (桥接层), tokio (异步运行时)

---

## 文件结构

### 后端文件 (src-tauri/src/hook/)

- `mod.rs` - hook模块入口，导出公共接口
- `types.rs` - 核心类型定义（HookType, HookContext, HookResult等）
- `event_bus.rs` - 事件总线实现（EventBus trait和DefaultEventBus）
- `config.rs` - 配置管理（HooksConfig和HookConfigManager）
- `logger.rs` - 日志记录（Logger trait和文件日志）
- `handler.rs` - Hook处理程序接口和内置处理程序
- `script.rs` - 脚本执行环境（JavaScript/TypeScript和外部可执行文件）

### 命令文件 (src-tauri/src/commands/)

- `hook.rs` - Tauri命令，供前端调用

### 前端文件 (src/)

- `components/hook/HookConfig.vue` - hook配置界面
- `components/hook/HookList.vue` - hook列表组件
- `components/hook/HookEditor.vue` - hook详情编辑组件
- `stores/hook.ts` - hook状态管理

---

## Task 1: 定义核心类型

**Files:**
- Create: `src-tauri/src/hook/mod.rs`
- Create: `src-tauri/src/hook/types.rs`

- [ ] **Step 1: 创建hook模块目录和mod.rs**

```rust
// src-tauri/src/hook/mod.rs
pub mod types;
pub mod event_bus;
pub mod config;
pub mod logger;
pub mod handler;
pub mod script;

pub use types::*;
```

- [ ] **Step 2: 定义HookType枚举**

```rust
// src-tauri/src/hook/types.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum HookType {
    PreCommit,
    PostCommit,
    PreUpdate,
    PostUpdate,
    StatusChange,
    ConflictDetected,
    PreCheckout,
    PostCheckout,
    PreMerge,
    PostMerge,
}

impl HookType {
    pub fn as_str(&self) -> &'static str {
        match self {
            HookType::PreCommit => "PreCommit",
            HookType::PostCommit => "PostCommit",
            HookType::PreUpdate => "PreUpdate",
            HookType::PostUpdate => "PostUpdate",
            HookType::StatusChange => "StatusChange",
            HookType::ConflictDetected => "ConflictDetected",
            HookType::PreCheckout => "PreCheckout",
            HookType::PostCheckout => "PostCheckout",
            HookType::PreMerge => "PreMerge",
            HookType::PostMerge => "PostMerge",
        }
    }
}
```

- [ ] **Step 3: 定义HookContext结构体**

```rust
// src-tauri/src/hook/types.rs (续)
#[derive(Debug, Clone)]
pub struct HookContext {
    pub hook_type: HookType,
    pub repo_path: String,
    pub data: HashMap<String, serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

impl HookContext {
    pub fn new(hook_type: HookType, repo_path: String) -> Self {
        Self {
            hook_type,
            repo_path,
            data: HashMap::new(),
            timestamp: Utc::now(),
        }
    }

    pub fn with_data(mut self, key: String, value: serde_json::Value) -> Self {
        self.data.insert(key, value);
        self
    }
}
```

- [ ] **Step 4: 定义HookResult枚举**

```rust
// src-tauri/src/hook/types.rs (续)
#[derive(Debug, Clone)]
pub enum HookResult {
    Continue,
    Cancel,
    Modify(HashMap<String, serde_json::Value>),
}
```

- [ ] **Step 5: 定义HookEvent结构体**

```rust
// src-tauri/src/hook/types.rs (续)
#[derive(Debug, Clone)]
pub struct HookEvent {
    pub hook_type: HookType,
    pub context: HookContext,
    pub timestamp: DateTime<Utc>,
}

impl HookEvent {
    pub fn new(hook_type: HookType, context: HookContext) -> Self {
        Self {
            hook_type,
            context,
            timestamp: Utc::now(),
        }
    }
}
```

- [ ] **Step 6: 定义HookError枚举**

```rust
// src-tauri/src/hook/types.rs (续)
#[derive(Debug, thiserror::Error)]
pub enum HookError {
    #[error("Hook执行失败: {0}")]
    ExecutionFailed(String),
    #[error("Hook执行超时")]
    TimeoutExpired,
    #[error("Hook配置错误: {0}")]
    InvalidConfiguration(String),
    #[error("Hook脚本未找到: {0}")]
    ScriptNotFound(String),
}

impl HookError {
    pub fn user_message(&self) -> String {
        match self {
            HookError::ExecutionFailed(msg) => format!("Hook执行失败: {}", msg),
            HookError::TimeoutExpired => "Hook执行超时".to_string(),
            HookError::InvalidConfiguration(msg) => format!("Hook配置错误: {}", msg),
            HookError::ScriptNotFound(path) => format!("Hook脚本未找到: {}", path),
        }
    }
}
```

- [ ] **Step 7: 提交代码**

```bash
rtk git add src-tauri/src/hook/mod.rs src-tauri/src/hook/types.rs
rtk git commit -m "feat(hook): 定义核心类型"
```

---

## Task 2: 实现事件总线

**Files:**
- Create: `src-tauri/src/hook/event_bus.rs`

- [ ] **Step 1: 定义EventBus trait**

```rust
// src-tauri/src/hook/event_bus.rs
use async_trait::async_trait;
use super::types::*;

#[async_trait]
pub trait EventBus: Send + Sync {
    async fn emit(&self, event: HookEvent);
    fn subscribe(&self, hook_type: HookType, handler: Box<dyn HookHandler>);
    fn unsubscribe(&self, hook_type: HookType, handler_name: &str);
}
```

- [ ] **Step 2: 定义HookHandler trait**

```rust
// src-tauri/src/hook/event_bus.rs (续)
#[async_trait]
pub trait HookHandler: Send + Sync {
    async fn execute(&self, context: &HookContext) -> Result<HookResult, HookError>;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
}
```

- [ ] **Step 3: 实现DefaultEventBus**

```rust
// src-tauri/src/hook/event_bus.rs (续)
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use super::logger::Logger;

pub struct DefaultEventBus {
    handlers: Arc<RwLock<HashMap<HookType, Vec<Box<dyn HookHandler>>>>>,
    logger: Arc<dyn Logger>,
}

impl DefaultEventBus {
    pub fn new(logger: Arc<dyn Logger>) -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
            logger,
        }
    }
}

#[async_trait]
impl EventBus for DefaultEventBus {
    async fn emit(&self, event: HookEvent) {
        let handlers = self.handlers.read().await;
        if let Some(handlers) = handlers.get(&event.hook_type) {
            for handler in handlers {
                let start = std::time::Instant::now();
                self.logger.log_hook_start(&event.hook_type, handler.name());
                
                match handler.execute(&event.context).await {
                    Ok(_) => {
                        let duration = start.elapsed();
                        self.logger.log_hook_end(&event.hook_type, handler.name(), duration);
                    }
                    Err(e) => {
                        self.logger.log_hook_error(&event.hook_type, handler.name(), &e);
                    }
                }
            }
        }
    }

    fn subscribe(&self, hook_type: HookType, handler: Box<dyn HookHandler>) {
        let mut handlers = self.handlers.write().await;
        handlers.entry(hook_type).or_insert_with(Vec::new).push(handler);
    }

    fn unsubscribe(&self, hook_type: HookType, handler_name: &str) {
        let mut handlers = self.handlers.write().await;
        if let Some(handlers) = handlers.get_mut(&hook_type) {
            handlers.retain(|h| h.name() != handler_name);
        }
    }
}
```

- [ ] **Step 4: 提交代码**

```bash
rtk git add src-tauri/src/hook/event_bus.rs
rtk git commit -m "feat(hook): 实现事件总线"
```

---

## Task 3: 实现配置管理

**Files:**
- Create: `src-tauri/src/hook/config.rs`

- [ ] **Step 1: 定义配置结构体**

```rust
// src-tauri/src/hook/config.rs
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
```

- [ ] **Step 2: 定义HookConfigManager trait**

```rust
// src-tauri/src/hook/config.rs (续)
use std::path::PathBuf;

pub trait HookConfigManager: Send + Sync {
    fn load_config(&self) -> Result<HooksConfig, Box<dyn std::error::Error>>;
    fn save_config(&self, config: &HooksConfig) -> Result<(), Box<dyn std::error::Error>>;
    fn add_handler(&self, config: HookHandlerConfig) -> Result<(), Box<dyn std::error::Error>>;
    fn remove_handler(&self, name: &str) -> Result<(), Box<dyn std::error::Error>>;
    fn update_handler(&self, name: &str, config: HookHandlerConfig) -> Result<(), Box<dyn std::error::Error>>;
}
```

- [ ] **Step 3: 实现FileHookConfigManager**

```rust
// src-tauri/src/hook/config.rs (续)
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

    fn update_handler(&self, name: &str, config: HookHandlerConfig) -> Result<(), Box<dyn std::error::Error>> {
        let mut hooks_config = self.load_config()?;
        if let Some(handler) = hooks_config.handlers.iter_mut().find(|h| h.name == name) {
            *handler = config;
        }
        self.save_config(&hooks_config)
    }
}
```

- [ ] **Step 4: 提交代码**

```bash
rtk git add src-tauri/src/hook/config.rs
rtk git commit -m "feat(hook): 实现配置管理"
```

---

## Task 4: 实现日志记录

**Files:**
- Create: `src-tauri/src/hook/logger.rs`

- [ ] **Step 1: 定义Logger trait**

```rust
// src-tauri/src/hook/logger.rs
use std::time::Duration;
use super::types::{HookType, HookError};

pub trait Logger: Send + Sync {
    fn log_hook_start(&self, hook_type: &HookType, handler_name: &str);
    fn log_hook_end(&self, hook_type: &HookType, handler_name: &str, duration: Duration);
    fn log_hook_error(&self, hook_type: &HookType, handler_name: &str, error: &HookError);
    fn log_hook_cancel(&self, hook_type: &HookType, handler_name: &str, reason: &str);
}
```

- [ ] **Step 2: 实现FileLogger**

```rust
// src-tauri/src/hook/logger.rs (续)
use std::path::PathBuf;
use std::fs::OpenOptions;
use std::io::Write;
use chrono::Local;

pub struct FileLogger {
    log_path: PathBuf,
}

impl FileLogger {
    pub fn new(log_path: PathBuf) -> Self {
        Self { log_path }
    }

    pub fn default_path() -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(".sourcesvn").join("logs").join("hooks.log")
    }

    fn write_log(&self, message: &str) {
        if let Some(parent) = self.log_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
        {
            let _ = writeln!(file, "{}", message);
        }
    }
}

impl Logger for FileLogger {
    fn log_hook_start(&self, hook_type: &HookType, handler_name: &str) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        self.write_log(&format!("[{}] Hook开始执行: {} - {}", timestamp, hook_type.as_str(), handler_name));
    }

    fn log_hook_end(&self, hook_type: &HookType, handler_name: &str, duration: Duration) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        self.write_log(&format!("[{}] Hook执行完成: {} - {} (耗时: {:?})", timestamp, hook_type.as_str(), handler_name, duration));
    }

    fn log_hook_error(&self, hook_type: &HookType, handler_name: &str, error: &HookError) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        self.write_log(&format!("[{}] Hook执行失败: {} - {} (错误: {})", timestamp, hook_type.as_str(), handler_name, error));
    }

    fn log_hook_cancel(&self, hook_type: &HookType, handler_name: &str, reason: &str) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        self.write_log(&format!("[{}] Hook执行取消: {} - {} (原因: {})", timestamp, hook_type.as_str(), handler_name, reason));
    }
}
```

- [ ] **Step 3: 提交代码**

```bash
rtk git add src-tauri/src/hook/logger.rs
rtk git commit -m "feat(hook): 实现日志记录"
```

---

## Task 5: 实现脚本执行环境

**Files:**
- Create: `src-tauri/src/hook/script.rs`

- [ ] **Step 1: 定义ScriptExecutor trait**

```rust
// src-tauri/src/hook/script.rs
use async_trait::async_trait;
use super::types::*;

#[async_trait]
pub trait ScriptExecutor: Send + Sync {
    async fn execute(&self, script_path: &str, context: &HookContext) -> Result<HookResult, HookError>;
    fn supports(&self, script_path: &str) -> bool;
}
```

- [ ] **Step 2: 实现ExternalScriptExecutor**

```rust
// src-tauri/src/hook/script.rs (续)
use std::process::Stdio;
use tokio::process::Command;

pub struct ExternalScriptExecutor;

impl ExternalScriptExecutor {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ScriptExecutor for ExternalScriptExecutor {
    async fn execute(&self, script_path: &str, context: &HookContext) -> Result<HookResult, HookError> {
        let context_json = serde_json::to_string(context)
            .map_err(|e| HookError::ExecutionFailed(e.to_string()))?;

        let output = Command::new(script_path)
            .arg(&context_json)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| HookError::ExecutionFailed(e.to_string()))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.trim().is_empty() {
                Ok(HookResult::Continue)
            } else {
                serde_json::from_str(&stdout)
                    .unwrap_or(HookResult::Continue)
            }
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(HookError::ExecutionFailed(stderr.to_string()))
        }
    }

    fn supports(&self, script_path: &str) -> bool {
        let path = std::path::Path::new(script_path);
        path.exists() && path.is_file()
    }
}
```

- [ ] **Step 3: 实现ScriptExecutorManager**

```rust
// src-tauri/src/hook/script.rs (续)
pub struct ScriptExecutorManager {
    executors: Vec<Box<dyn ScriptExecutor>>,
}

impl ScriptExecutorManager {
    pub fn new() -> Self {
        Self {
            executors: vec![
                Box::new(ExternalScriptExecutor::new()),
            ],
        }
    }

    pub async fn execute(&self, script_path: &str, context: &HookContext) -> Result<HookResult, HookError> {
        for executor in &self.executors {
            if executor.supports(script_path) {
                return executor.execute(script_path, context).await;
            }
        }
        Err(HookError::ScriptNotFound(script_path.to_string()))
    }
}
```

- [ ] **Step 4: 提交代码**

```bash
rtk git add src-tauri/src/hook/script.rs
rtk git commit -m "feat(hook): 实现脚本执行环境"
```

---

## Task 6: 实现Hook处理程序

**Files:**
- Create: `src-tauri/src/hook/handler.rs`

- [ ] **Step 1: 定义内置处理程序**

```rust
// src-tauri/src/hook/handler.rs
use async_trait::async_trait;
use super::types::*;
use super::event_bus::HookHandler;
use super::script::ScriptExecutorManager;
use std::sync::Arc;

pub struct ScriptHookHandler {
    name: String,
    description: String,
    script_path: String,
    executor: Arc<ScriptExecutorManager>,
}

impl ScriptHookHandler {
    pub fn new(
        name: String,
        description: String,
        script_path: String,
        executor: Arc<ScriptExecutorManager>,
    ) -> Self {
        Self {
            name,
            description,
            script_path,
            executor,
        }
    }
}

#[async_trait]
impl HookHandler for ScriptHookHandler {
    async fn execute(&self, context: &HookContext) -> Result<HookResult, HookError> {
        self.executor.execute(&self.script_path, context).await
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }
}
```

- [ ] **Step 2: 提交代码**

```bash
rtk git add src-tauri/src/hook/handler.rs
rtk git commit -m "feat(hook): 实现Hook处理程序"
```

---

## Task 7: 实现Tauri命令

**Files:**
- Create: `src-tauri/src/commands/hook.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 创建hook命令模块**

```rust
// src-tauri/src/commands/hook.rs
use tauri::State;
use crate::app_state::AppState;
use crate::hook::{HookType, HookContext, HookEvent, HookHandlerConfig};
use std::collections::HashMap;

#[tauri::command]
pub async fn hook_subscribe(
    state: State<'_, AppState>,
    hook_type: HookType,
    handler_name: String,
    script_path: String,
) -> Result<(), String> {
    // TODO: 实现订阅逻辑
    Ok(())
}

#[tauri::command]
pub async fn hook_unsubscribe(
    state: State<'_, AppState>,
    hook_type: HookType,
    handler_name: String,
) -> Result<(), String> {
    // TODO: 实现取消订阅逻辑
    Ok(())
}

#[tauri::command]
pub async fn hook_emit(
    state: State<'_, AppState>,
    hook_type: HookType,
    repo_path: String,
    data: HashMap<String, serde_json::Value>,
) -> Result<(), String> {
    // TODO: 实现事件发射逻辑
    Ok(())
}

#[tauri::command]
pub async fn hook_load_config(
    state: State<'_, AppState>,
) -> Result<crate::hook::HooksConfig, String> {
    // TODO: 实现加载配置逻辑
    Ok(crate::hook::HooksConfig::default())
}

#[tauri::command]
pub async fn hook_save_config(
    state: State<'_, AppState>,
    config: crate::hook::HooksConfig,
) -> Result<(), String> {
    // TODO: 实现保存配置逻辑
    Ok(())
}

#[tauri::command]
pub async fn hook_add_handler(
    state: State<'_, AppState>,
    handler: HookHandlerConfig,
) -> Result<(), String> {
    // TODO: 实现添加处理程序逻辑
    Ok(())
}

#[tauri::command]
pub async fn hook_remove_handler(
    state: State<'_, AppState>,
    name: String,
) -> Result<(), String> {
    // TODO: 实现删除处理程序逻辑
    Ok(())
}

#[tauri::command]
pub async fn hook_update_handler(
    state: State<'_, AppState>,
    name: String,
    handler: HookHandlerConfig,
) -> Result<(), String> {
    // TODO: 实现更新处理程序逻辑
    Ok(())
}
```

- [ ] **Step 2: 更新commands/mod.rs**

```rust
// src-tauri/src/commands/mod.rs
pub mod ai;
pub mod config;
pub mod hook;
pub mod shelve;
pub mod svn;
```

- [ ] **Step 3: 更新lib.rs注册命令**

```rust
// src-tauri/src/lib.rs (续)
.invoke_handler(tauri::generate_handler![
    // ... 现有命令 ...
    commands::hook::hook_subscribe,
    commands::hook::hook_unsubscribe,
    commands::hook::hook_emit,
    commands::hook::hook_load_config,
    commands::hook::hook_save_config,
    commands::hook::hook_add_handler,
    commands::hook::hook_remove_handler,
    commands::hook::hook_update_handler,
])
```

- [ ] **Step 4: 提交代码**

```bash
rtk git add src-tauri/src/commands/hook.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
rtk git commit -m "feat(hook): 实现Tauri命令"
```

---

## Task 8: 更新AppState支持Hook

**Files:**
- Modify: `src-tauri/src/app_state.rs`

- [ ] **Step 1: 添加Hook相关字段**

```rust
// src-tauri/src/app_state.rs (续)
use crate::hook::{DefaultEventBus, EventBus, FileHookConfigManager, FileLogger, ScriptExecutorManager};
use std::sync::Arc;

pub struct AppState {
    // ... 现有字段 ...
    pub hook_event_bus: Arc<dyn EventBus>,
    pub hook_config_manager: Arc<FileHookConfigManager>,
    pub hook_script_executor: Arc<ScriptExecutorManager>,
}

impl AppState {
    pub fn new() -> Self {
        // ... 现有初始化代码 ...
        let logger = Arc::new(FileLogger::new(FileLogger::default_path()));
        let event_bus = Arc::new(DefaultEventBus::new(logger));
        let config_manager = Arc::new(FileHookConfigManager::new(FileHookConfigManager::default_path()));
        let script_executor = Arc::new(ScriptExecutorManager::new());

        Self {
            // ... 现有字段 ...
            hook_event_bus: event_bus,
            hook_config_manager: config_manager,
            hook_script_executor: script_executor,
        }
    }
}
```

- [ ] **Step 2: 提交代码**

```bash
rtk git add src-tauri/src/app_state.rs
rtk git commit -m "feat(hook): 更新AppState支持Hook"
```

---

## Task 9: 实现前端状态管理

**Files:**
- Create: `src/stores/hook.ts`

- [ ] **Step 1: 创建hook store**

```typescript
// src/stores/hook.ts
import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface HookHandlerConfig {
  name: string
  hook_type: string
  script_path: string
  enabled: boolean
}

export interface HooksConfig {
  enabled: boolean
  handlers: HookHandlerConfig[]
}

export const useHookStore = defineStore('hook', () => {
  const config = ref<HooksConfig>({
    enabled: true,
    handlers: []
  })

  const loading = ref(false)
  const error = ref<string | null>(null)

  async function loadConfig() {
    loading.value = true
    error.value = null
    try {
      config.value = await invoke('hook_load_config')
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function saveConfig() {
    loading.value = true
    error.value = null
    try {
      await invoke('hook_save_config', { config: config.value })
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function addHandler(handler: HookHandlerConfig) {
    loading.value = true
    error.value = null
    try {
      await invoke('hook_add_handler', { handler })
      await loadConfig()
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function removeHandler(name: string) {
    loading.value = true
    error.value = null
    try {
      await invoke('hook_remove_handler', { name })
      await loadConfig()
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function updateHandler(name: string, handler: HookHandlerConfig) {
    loading.value = true
    error.value = null
    try {
      await invoke('hook_update_handler', { name, handler })
      await loadConfig()
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  return {
    config,
    loading,
    error,
    loadConfig,
    saveConfig,
    addHandler,
    removeHandler,
    updateHandler
  }
})
```

- [ ] **Step 2: 提交代码**

```bash
rtk git add src/stores/hook.ts
rtk git commit -m "feat(hook): 实现前端状态管理"
```

---

## Task 10: 实现前端配置界面

**Files:**
- Create: `src/components/hook/HookConfig.vue`
- Create: `src/components/hook/HookList.vue`
- Create: `src/components/hook/HookEditor.vue`

- [ ] **Step 1: 创建HookList组件**

```vue
<!-- src/components/hook/HookList.vue -->
<template>
  <div class="hook-list">
    <div class="hook-list-header">
      <input
        v-model="searchQuery"
        placeholder="搜索hook..."
        class="search-input"
      />
      <button @click="$emit('add')" class="add-button">
        添加Hook
      </button>
    </div>
    <div class="hook-items">
      <div
        v-for="handler in filteredHandlers"
        :key="handler.name"
        :class="['hook-item', { active: selectedName === handler.name }]"
        @click="$emit('select', handler.name)"
      >
        <div class="hook-item-info">
          <span class="hook-name">{{ handler.name }}</span>
          <span class="hook-type">{{ handler.hook_type }}</span>
        </div>
        <div class="hook-item-actions">
          <button
            @click.stop="$emit('toggle', handler.name)"
            :class="['toggle-button', { enabled: handler.enabled }]"
          >
            {{ handler.enabled ? '启用' : '禁用' }}
          </button>
          <button
            @click.stop="$emit('delete', handler.name)"
            class="delete-button"
          >
            删除
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import type { HookHandlerConfig } from '@/stores/hook'

const props = defineProps<{
  handlers: HookHandlerConfig[]
  selectedName: string | null
}>()

defineEmits<{
  add: []
  select: [name: string]
  toggle: [name: string]
  delete: [name: string]
}>()

const searchQuery = ref('')

const filteredHandlers = computed(() => {
  if (!searchQuery.value) return props.handlers
  const query = searchQuery.value.toLowerCase()
  return props.handlers.filter(
    h => h.name.toLowerCase().includes(query) || h.hook_type.toLowerCase().includes(query)
  )
})
</script>

<style scoped>
.hook-list {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.hook-list-header {
  display: flex;
  gap: 8px;
  padding: 12px;
  border-bottom: 1px solid var(--border-color);
}

.search-input {
  flex: 1;
  padding: 8px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
}

.add-button {
  padding: 8px 16px;
  background: var(--primary-color);
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}

.hook-items {
  flex: 1;
  overflow-y: auto;
}

.hook-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px;
  border-bottom: 1px solid var(--border-color);
  cursor: pointer;
}

.hook-item:hover {
  background: var(--hover-color);
}

.hook-item.active {
  background: var(--active-color);
}

.hook-item-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.hook-name {
  font-weight: 500;
}

.hook-type {
  font-size: 12px;
  color: var(--text-secondary);
}

.hook-item-actions {
  display: flex;
  gap: 8px;
}

.toggle-button {
  padding: 4px 8px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background: transparent;
  cursor: pointer;
}

.toggle-button.enabled {
  background: var(--success-color);
  color: white;
  border-color: var(--success-color);
}

.delete-button {
  padding: 4px 8px;
  border: 1px solid var(--danger-color);
  border-radius: 4px;
  background: transparent;
  color: var(--danger-color);
  cursor: pointer;
}

.delete-button:hover {
  background: var(--danger-color);
  color: white;
}
</style>
```

- [ ] **Step 2: 创建HookEditor组件**

```vue
<!-- src/components/hook/HookEditor.vue -->
<template>
  <div class="hook-editor">
    <div class="editor-header">
      <h3>{{ isNew ? '添加Hook' : '编辑Hook' }}</h3>
    </div>
    <div class="editor-content">
      <div class="form-group">
        <label>名称</label>
        <input
          v-model="formData.name"
          :disabled="!isNew"
          placeholder="输入hook名称"
        />
      </div>
      <div class="form-group">
        <label>类型</label>
        <select v-model="formData.hook_type">
          <option value="PreCommit">PreCommit</option>
          <option value="PostCommit">PostCommit</option>
          <option value="PreUpdate">PreUpdate</option>
          <option value="PostUpdate">PostUpdate</option>
          <option value="StatusChange">StatusChange</option>
          <option value="ConflictDetected">ConflictDetected</option>
          <option value="PreCheckout">PreCheckout</option>
          <option value="PostCheckout">PostCheckout</option>
          <option value="PreMerge">PreMerge</option>
          <option value="PostMerge">PostMerge</option>
        </select>
      </div>
      <div class="form-group">
        <label>脚本路径</label>
        <div class="path-input">
          <input
            v-model="formData.script_path"
            placeholder="选择脚本文件"
          />
          <button @click="selectFile" class="select-button">
            选择
          </button>
        </div>
      </div>
      <div class="form-group">
        <label>
          <input
            type="checkbox"
            v-model="formData.enabled"
          />
          启用
        </label>
      </div>
    </div>
    <div class="editor-actions">
      <button @click="$emit('cancel')" class="cancel-button">
        取消
      </button>
      <button @click="handleSave" class="save-button">
        保存
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import type { HookHandlerConfig } from '@/stores/hook'

const props = defineProps<{
  handler: HookHandlerConfig | null
  isNew: boolean
}>()

const emit = defineEmits<{
  save: [handler: HookHandlerConfig]
  cancel: []
}>()

const formData = ref<HookHandlerConfig>({
  name: '',
  hook_type: 'PostCommit',
  script_path: '',
  enabled: true
})

watch(() => props.handler, (newHandler) => {
  if (newHandler) {
    formData.value = { ...newHandler }
  } else {
    formData.value = {
      name: '',
      hook_type: 'PostCommit',
      script_path: '',
      enabled: true
    }
  }
}, { immediate: true })

function selectFile() {
  // TODO: 实现文件选择对话框
}

function handleSave() {
  emit('save', { ...formData.value })
}
</script>

<style scoped>
.hook-editor {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.editor-header {
  padding: 16px;
  border-bottom: 1px solid var(--border-color);
}

.editor-header h3 {
  margin: 0;
}

.editor-content {
  flex: 1;
  padding: 16px;
  overflow-y: auto;
}

.form-group {
  margin-bottom: 16px;
}

.form-group label {
  display: block;
  margin-bottom: 8px;
  font-weight: 500;
}

.form-group input,
.form-group select {
  width: 100%;
  padding: 8px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
}

.path-input {
  display: flex;
  gap: 8px;
}

.path-input input {
  flex: 1;
}

.select-button {
  padding: 8px 16px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background: transparent;
  cursor: pointer;
}

.editor-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 16px;
  border-top: 1px solid var(--border-color);
}

.cancel-button {
  padding: 8px 16px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background: transparent;
  cursor: pointer;
}

.save-button {
  padding: 8px 16px;
  background: var(--primary-color);
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}
</style>
```

- [ ] **Step 3: 创建HookConfig组件**

```vue
<!-- src/components/hook/HookConfig.vue -->
<template>
  <div class="hook-config">
    <div class="config-header">
      <h2>Hook配置</h2>
      <div class="config-actions">
        <label>
          <input
            type="checkbox"
            v-model="config.enabled"
            @change="saveConfig"
          />
          启用Hook系统
        </label>
      </div>
    </div>
    <div class="config-content">
      <HookList
        :handlers="config.handlers"
        :selected-name="selectedName"
        @add="handleAdd"
        @select="handleSelect"
        @toggle="handleToggle"
        @delete="handleDelete"
      />
      <HookEditor
        v-if="selectedHandler || isNew"
        :handler="selectedHandler"
        :is-new="isNew"
        @save="handleSave"
        @cancel="handleCancel"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useHookStore } from '@/stores/hook'
import HookList from './HookList.vue'
import HookEditor from './HookEditor.vue'
import type { HookHandlerConfig } from '@/stores/hook'

const hookStore = useHookStore()

const selectedName = ref<string | null>(null)
const isNew = ref(false)

const config = computed(() => hookStore.config)

const selectedHandler = computed(() => {
  if (!selectedName.value) return null
  return config.value.handlers.find(h => h.name === selectedName.value) || null
})

onMounted(() => {
  hookStore.loadConfig()
})

function handleAdd() {
  selectedName.value = null
  isNew.value = true
}

function handleSelect(name: string) {
  selectedName.value = name
  isNew.value = false
}

async function handleToggle(name: string) {
  const handler = config.value.handlers.find(h => h.name === name)
  if (handler) {
    await hookStore.updateHandler(name, { ...handler, enabled: !handler.enabled })
  }
}

async function handleDelete(name: string) {
  if (confirm(`确定要删除hook "${name}" 吗？`)) {
    await hookStore.removeHandler(name)
    if (selectedName.value === name) {
      selectedName.value = null
      isNew.value = false
    }
  }
}

async function handleSave(handler: HookHandlerConfig) {
  if (isNew.value) {
    await hookStore.addHandler(handler)
  } else {
    await hookStore.updateHandler(handler.name, handler)
  }
  isNew.value = false
  selectedName.value = handler.name
}

function handleCancel() {
  isNew.value = false
  if (!selectedName.value) {
    selectedName.value = null
  }
}

async function saveConfig() {
  await hookStore.saveConfig()
}
</script>

<style scoped>
.hook-config {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.config-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  border-bottom: 1px solid var(--border-color);
}

.config-header h2 {
  margin: 0;
}

.config-content {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.config-content > *:first-child {
  width: 300px;
  border-right: 1px solid var(--border-color);
}

.config-content > *:last-child {
  flex: 1;
}
</style>
```

- [ ] **Step 4: 提交代码**

```bash
rtk git add src/components/hook/HookConfig.vue src/components/hook/HookList.vue src/components/hook/HookEditor.vue
rtk git commit -m "feat(hook): 实现前端配置界面"
```

---

## Task 11: 集成Hook到现有命令

**Files:**
- Modify: `src-tauri/src/commands/svn.rs`

- [ ] **Step 1: 在svn_commit中添加pre-commit hook**

```rust
// src-tauri/src/commands/svn.rs (续)
#[tauri::command]
pub async fn svn_commit(
    state: State<'_, AppState>,
    path: String,
    message: String,
    files: Vec<String>,
) -> Result<CommitResult, String> {
    // 触发pre-commit hook
    let mut context = HookContext::new(HookType::PreCommit, path.clone());
    context = context.with_data("message".to_string(), serde_json::Value::String(message.clone()));
    context = context.with_data("files".to_string(), serde_json::to_value(&files).unwrap());
    
    let event = HookEvent::new(HookType::PreCommit, context);
    state.hook_event_bus.emit(event).await;
    
    // 执行实际的svn commit
    let result = crate::svn::commit::svn_commit(&path, &message, &files).await?;
    
    // 触发post-commit hook
    let mut context = HookContext::new(HookType::PostCommit, path.clone());
    context = context.with_data("revision".to_string(), serde_json::Value::String(result.revision.clone()));
    
    let event = HookEvent::new(HookType::PostCommit, context);
    state.hook_event_bus.emit(event).await;
    
    Ok(result)
}
```

- [ ] **Step 2: 在svn_update中添加pre-update和post-update hook**

```rust
// src-tauri/src/commands/svn.rs (续)
#[tauri::command]
pub async fn svn_update(
    state: State<'_, AppState>,
    path: String,
) -> Result<UpdateResult, String> {
    // 触发pre-update hook
    let context = HookContext::new(HookType::PreUpdate, path.clone());
    let event = HookEvent::new(HookType::PreUpdate, context);
    state.hook_event_bus.emit(event).await;
    
    // 执行实际的svn update
    let result = crate::svn::update::svn_update(&path).await?;
    
    // 触发post-update hook
    let mut context = HookContext::new(HookType::PostUpdate, path.clone());
    context = context.with_data("revision".to_string(), serde_json::Value::String(result.revision.clone()));
    
    let event = HookEvent::new(HookType::PostUpdate, context);
    state.hook_event_bus.emit(event).await;
    
    Ok(result)
}
```

- [ ] **Step 3: 提交代码**

```bash
rtk git add src-tauri/src/commands/svn.rs
rtk git commit -m "feat(hook): 集成Hook到现有命令"
```

---

## Task 12: 添加测试

**Files:**
- Create: `src-tauri/src/hook/tests.rs`

- [ ] **Step 1: 创建测试模块**

```rust
// src-tauri/src/hook/tests.rs
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    struct MockLogger {
        logs: Arc<Mutex<Vec<String>>>,
    }

    impl MockLogger {
        fn new() -> Self {
            Self {
                logs: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    impl Logger for MockLogger {
        fn log_hook_start(&self, hook_type: &HookType, handler_name: &str) {
            let mut logs = self.logs.blocking_lock();
            logs.push(format!("start: {} - {}", hook_type.as_str(), handler_name));
        }

        fn log_hook_end(&self, hook_type: &HookType, handler_name: &str, _duration: Duration) {
            let mut logs = self.logs.blocking_lock();
            logs.push(format!("end: {} - {}", hook_type.as_str(), handler_name));
        }

        fn log_hook_error(&self, hook_type: &HookType, handler_name: &str, error: &HookError) {
            let mut logs = self.logs.blocking_lock();
            logs.push(format!("error: {} - {} - {}", hook_type.as_str(), handler_name, error));
        }

        fn log_hook_cancel(&self, hook_type: &HookType, handler_name: &str, reason: &str) {
            let mut logs = self.logs.blocking_lock();
            logs.push(format!("cancel: {} - {} - {}", hook_type.as_str(), handler_name, reason));
        }
    }

    #[tokio::test]
    async fn test_event_bus_subscribe_and_emit() {
        let logger = Arc::new(MockLogger::new());
        let event_bus = DefaultEventBus::new(logger.clone());
        
        // 创建测试处理程序
        struct TestHandler;
        
        #[async_trait]
        impl HookHandler for TestHandler {
            async fn execute(&self, _context: &HookContext) -> Result<HookResult, HookError> {
                Ok(HookResult::Continue)
            }
            
            fn name(&self) -> &str {
                "test-handler"
            }
            
            fn description(&self) -> &str {
                "Test handler"
            }
        }
        
        // 订阅事件
        event_bus.subscribe(HookType::PreCommit, Box::new(TestHandler));
        
        // 发射事件
        let context = HookContext::new(HookType::PreCommit, "/test/path".to_string());
        let event = HookEvent::new(HookType::PreCommit, context);
        event_bus.emit(event).await;
        
        // 验证日志
        let logs = logger.logs.lock().await;
        assert_eq!(logs.len(), 2);
        assert!(logs[0].contains("start: PreCommit - test-handler"));
        assert!(logs[1].contains("end: PreCommit - test-handler"));
    }

    #[test]
    fn test_hook_type_as_str() {
        assert_eq!(HookType::PreCommit.as_str(), "PreCommit");
        assert_eq!(HookType::PostCommit.as_str(), "PostCommit");
        assert_eq!(HookType::PreUpdate.as_str(), "PreUpdate");
        assert_eq!(HookType::PostUpdate.as_str(), "PostUpdate");
    }

    #[test]
    fn test_hook_context_with_data() {
        let context = HookContext::new(HookType::PreCommit, "/test/path".to_string())
            .with_data("key".to_string(), serde_json::Value::String("value".to_string()));
        
        assert_eq!(context.data.len(), 1);
        assert_eq!(context.data.get("key").unwrap(), &serde_json::Value::String("value".to_string()));
    }

    #[test]
    fn test_hook_error_user_message() {
        let error = HookError::ExecutionFailed("test error".to_string());
        assert_eq!(error.user_message(), "Hook执行失败: test error");
        
        let error = HookError::TimeoutExpired;
        assert_eq!(error.user_message(), "Hook执行超时");
    }
}
```

- [ ] **Step 2: 更新mod.rs添加测试模块**

```rust
// src-tauri/src/hook/mod.rs (续)
#[cfg(test)]
mod tests;
```

- [ ] **Step 3: 提交代码**

```bash
rtk git add src-tauri/src/hook/tests.rs src-tauri/src/hook/mod.rs
rtk git commit -m "test(hook): 添加测试"
```

---

## Task 13: 更新Cargo.toml添加依赖

**Files:**
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: 添加必要依赖**

```toml
# src-tauri/Cargo.toml (续)
[dependencies]
# ... 现有依赖 ...
async-trait = "0.1"
chrono = { version = "0.4", features = ["serde"] }
dirs = "5.0"
toml = "0.8"
thiserror = "1.0"
```

- [ ] **Step 2: 提交代码**

```bash
rtk git add src-tauri/Cargo.toml
rtk git commit -m "chore(hook): 添加必要依赖"
```

---

## Task 14: 完整集成测试

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 添加集成测试**

```rust
// src-tauri/src/lib.rs (续)
#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::hook::*;

    #[tokio::test]
    async fn test_hook_integration() {
        // 创建AppState
        let state = AppState::new();
        
        // 测试配置管理
        let config = HooksConfig {
            enabled: true,
            handlers: vec![
                HookHandlerConfig {
                    name: "test-handler".to_string(),
                    hook_type: HookType::PostCommit,
                    script_path: "/path/to/script.js".to_string(),
                    enabled: true,
                },
            ],
        };
        
        // 测试事件总线
        let context = HookContext::new(HookType::PostCommit, "/test/path".to_string());
        let event = HookEvent::new(HookType::PostCommit, context);
        state.hook_event_bus.emit(event).await;
    }
}
```

- [ ] **Step 2: 提交代码**

```bash
rtk git add src-tauri/src/lib.rs
rtk git commit -m "test(hook): 添加集成测试"
```

---

## Task 15: 文档更新

**Files:**
- Modify: `docs/FRAMEWORK.md`

- [ ] **Step 1: 更新架构文档**

```markdown
# docs/FRAMEWORK.md (续)

## 七、Hook机制

### 7.1 架构概述

Hook机制基于事件总线架构，支持异步执行、通知/日志记录和拦截/修改操作。

### 7.2 核心组件

- **EventBus**: 事件总线，负责分发hook事件
- **HookHandler**: hook处理程序接口
- **HookConfigManager**: 配置管理器
- **Logger**: 日志记录器
- **ScriptExecutor**: 脚本执行器

### 7.3 支持的Hook类型

- `PreCommit`: 提交前触发
- `PostCommit`: 提交后触发
- `PreUpdate`: 更新前触发
- `PostUpdate`: 更新后触发
- `StatusChange`: 文件状态变更时触发
- `ConflictDetected`: 冲突发生时触发
- `PreCheckout`: 检出前触发
- `PostCheckout`: 检出后触发
- `PreMerge`: 合并前触发
- `PostMerge`: 合并后触发

### 7.4 配置文件

配置文件路径：`~/.sourcesvn/hooks.toml`

```toml
[hooks]
enabled = true

[[hooks.handlers]]
name = "commit-notifier"
type = "PostCommit"
script_path = "~/.sourcesvn/hooks/commit-notifier.js"
enabled = true
```

### 7.5 日志文件

日志文件路径：`~/.sourcesvn/logs/hooks.log`
```

- [ ] **Step 2: 提交代码**

```bash
rtk git add docs/FRAMEWORK.md
rtk git commit -m "docs(hook): 更新架构文档"
```

---

## 总结

本实施计划将hook机制分解为15个bite-sized任务，每个任务都包含具体的代码实现和测试步骤。通过TDD方法，确保每个功能都有对应的测试覆盖。

### 主要特点

1. **模块化设计**：每个组件都有明确的职责和接口
2. **异步支持**：使用tokio实现异步执行
3. **配置驱动**：通过配置文件管理hook
4. **完整测试**：包含单元测试和集成测试
5. **文档完善**：提供详细的架构文档

### 执行顺序

建议按以下顺序执行任务：

1. Task 1-4: 核心类型和基础设施
2. Task 5-8: 脚本执行和Tauri命令
3. Task 9-10: 前端实现
4. Task 11-12: 集成和测试
5. Task 13-15: 依赖和文档

每个任务完成后，都应该运行测试确保功能正常。