use crate::hook::event_bus::{DefaultEventBus, EventBus, HookHandler};
use crate::hook::logger::Logger;
use crate::hook::types::*;
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use std::time::Duration;

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
        self.logs
            .lock()
            .unwrap()
            .push(format!("start: {} - {}", hook_type.as_str(), handler_name));
    }

    fn log_hook_end(&self, hook_type: &HookType, handler_name: &str, _duration: Duration) {
        self.logs
            .lock()
            .unwrap()
            .push(format!("end: {} - {}", hook_type.as_str(), handler_name));
    }

    fn log_hook_error(&self, hook_type: &HookType, handler_name: &str, error: &HookError) {
        self.logs.lock().unwrap().push(format!(
            "error: {} - {} - {}",
            hook_type.as_str(),
            handler_name,
            error
        ));
    }

    fn log_hook_cancel(&self, hook_type: &HookType, handler_name: &str, reason: &str) {
        self.logs.lock().unwrap().push(format!(
            "cancel: {} - {} - {}",
            hook_type.as_str(),
            handler_name,
            reason
        ));
    }
}

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

#[tokio::test]
async fn test_event_bus_subscribe_and_emit() {
    let logger = Arc::new(MockLogger::new());
    let event_bus = DefaultEventBus::new(logger.clone());

    event_bus
        .subscribe(Arc::new(TestHandler))
        .await;

    let context = HookContext::new(HookType::PreCommit, "/test/path".to_string());
    let event = HookEvent::new(HookType::PreCommit, context);
    let results = event_bus.emit(event).await;

    assert_eq!(results.len(), 1);
    assert!(results[0].is_ok());

    let logs = logger.logs.lock().unwrap();
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
    assert_eq!(HookType::StatusChange.as_str(), "StatusChange");
    assert_eq!(HookType::ConflictDetected.as_str(), "ConflictDetected");
    assert_eq!(HookType::PreCheckout.as_str(), "PreCheckout");
    assert_eq!(HookType::PostCheckout.as_str(), "PostCheckout");
    assert_eq!(HookType::PreMerge.as_str(), "PreMerge");
    assert_eq!(HookType::PostMerge.as_str(), "PostMerge");
}

#[test]
fn test_hook_context_with_data() {
    let context = HookContext::new(HookType::PreCommit, "/test/path".to_string())
        .with_data(
            "key".to_string(),
            serde_json::Value::String("value".to_string()),
        );

    assert_eq!(context.data.len(), 1);
    assert_eq!(
        context.data.get("key").unwrap(),
        &serde_json::Value::String("value".to_string())
    );
}

#[test]
fn test_hook_error_user_message() {
    let error = HookError::ExecutionFailed("test error".to_string());
    assert_eq!(error.user_message(), "Hook执行失败: test error");

    let error = HookError::TimeoutExpired;
    assert_eq!(error.user_message(), "Hook执行超时");

    let error = HookError::InvalidConfiguration("bad config".to_string());
    assert_eq!(error.user_message(), "Hook配置错误: bad config");

    let error = HookError::ScriptNotFound("/path/to/script".to_string());
    assert_eq!(error.user_message(), "Hook脚本未找到: /path/to/script");
}
