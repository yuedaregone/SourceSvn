use async_trait::async_trait;
use std::process::Stdio;
use tokio::process::Command;

use super::types::*;

#[async_trait]
pub trait ScriptExecutor: Send + Sync {
    async fn execute(&self, script_path: &str, context: &HookContext) -> Result<HookResult, HookError>;
    fn supports(&self, script_path: &str) -> bool;
}

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
                Ok(serde_json::from_str::<HookResult>(&stdout)
                    .unwrap_or(HookResult::Continue))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn external_executor_supports_existing_file() {
        let executor = ExternalScriptExecutor::new();
        let path = format!("{}/src/hook/script.rs", env!("CARGO_MANIFEST_DIR"));
        assert!(executor.supports(&path));
    }

    #[test]
    fn external_executor_rejects_nonexistent_path() {
        let executor = ExternalScriptExecutor::new();
        assert!(!executor.supports("/nonexistent/path/to/script"));
    }

    #[test]
    fn external_executor_rejects_directory() {
        let executor = ExternalScriptExecutor::new();
        let path = format!("{}/src/hook", env!("CARGO_MANIFEST_DIR"));
        assert!(!executor.supports(&path));
    }

    #[tokio::test]
    async fn manager_returns_not_found_for_missing_script() {
        let manager = ScriptExecutorManager::new();
        let context = HookContext::new(HookType::PreCommit, "/tmp".to_string());
        let result = manager.execute("/nonexistent/script", &context).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            HookError::ScriptNotFound(path) => assert_eq!(path, "/nonexistent/script"),
            other => panic!("expected ScriptNotFound, got {:?}", other),
        }
    }
}
