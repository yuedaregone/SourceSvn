use std::sync::Arc;

use async_trait::async_trait;

use super::event_bus::HookHandler;
use super::script::ScriptExecutorManager;
use super::types::*;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handler_returns_correct_name() {
        let executor = Arc::new(ScriptExecutorManager::new());
        let handler = ScriptHookHandler::new(
            "my-hook".into(),
            "test hook".into(),
            "/path/to/script".into(),
            executor,
        );
        assert_eq!(handler.name(), "my-hook");
    }

    #[test]
    fn handler_returns_correct_description() {
        let executor = Arc::new(ScriptExecutorManager::new());
        let handler = ScriptHookHandler::new(
            "my-hook".into(),
            "some description".into(),
            "/path/to/script".into(),
            executor,
        );
        assert_eq!(handler.description(), "some description");
    }

    #[tokio::test]
    async fn handler_delegates_to_executor_and_returns_not_found_for_missing_script() {
        let executor = Arc::new(ScriptExecutorManager::new());
        let handler = ScriptHookHandler::new(
            "my-hook".into(),
            "test hook".into(),
            "/nonexistent/script".into(),
            executor,
        );
        let context = HookContext::new(HookType::PreCommit, "/tmp".into());
        let result = handler.execute(&context).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            HookError::ScriptNotFound(path) => assert_eq!(path, "/nonexistent/script"),
            other => panic!("expected ScriptNotFound, got {:?}", other),
        }
    }
}
