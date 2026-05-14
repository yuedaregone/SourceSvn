use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HookResult {
    Continue,
    Cancel,
    Modify(HashMap<String, serde_json::Value>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
