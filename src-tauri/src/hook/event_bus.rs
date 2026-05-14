use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use super::logger::Logger;
use super::types::*;

#[async_trait]
pub trait EventBus: Send + Sync {
    async fn emit(&self, event: HookEvent) -> Vec<Result<HookResult, HookError>>;
    async fn subscribe(&self, hook_type: HookType, handler: Arc<dyn HookHandler>);
    async fn unsubscribe(&self, hook_type: HookType, handler_name: &str);
}

#[async_trait]
pub trait HookHandler: Send + Sync {
    async fn execute(&self, context: &HookContext) -> Result<HookResult, HookError>;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
}

pub struct DefaultEventBus {
    handlers: Arc<RwLock<HashMap<HookType, Vec<Arc<dyn HookHandler>>>>>,
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
    async fn emit(&self, event: HookEvent) -> Vec<Result<HookResult, HookError>> {
        let handlers = {
            let guard = self.handlers.read().await;
            guard.get(&event.hook_type).cloned()
        };

        let Some(handlers) = handlers else {
            return Vec::new();
        };

        let mut results = Vec::with_capacity(handlers.len());
        for handler in &handlers {
            let start = std::time::Instant::now();
            self.logger.log_hook_start(&event.hook_type, handler.name());

            match handler.execute(&event.context).await {
                Ok(result) => {
                    let duration = start.elapsed();
                    self.logger
                        .log_hook_end(&event.hook_type, handler.name(), duration);
                    results.push(Ok(result));
                }
                Err(e) => {
                    self.logger
                        .log_hook_error(&event.hook_type, handler.name(), &e);
                    results.push(Err(e));
                }
            }
        }
        results
    }

    async fn subscribe(&self, hook_type: HookType, handler: Arc<dyn HookHandler>) {
        let mut handlers = self.handlers.write().await;
        handlers.entry(hook_type).or_default().push(handler);
    }

    async fn unsubscribe(&self, hook_type: HookType, handler_name: &str) {
        let mut handlers = self.handlers.write().await;
        if let Some(handlers) = handlers.get_mut(&hook_type) {
            handlers.retain(|h| h.name() != handler_name);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    struct MockHandler {
        handler_name: String,
        call_count: Arc<AtomicUsize>,
    }

    impl MockHandler {
        fn new(name: &str, call_count: Arc<AtomicUsize>) -> Self {
            Self {
                handler_name: name.to_string(),
                call_count,
            }
        }
    }

    #[async_trait]
    impl HookHandler for MockHandler {
        async fn execute(&self, _context: &HookContext) -> Result<HookResult, HookError> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            Ok(HookResult::Continue)
        }

        fn name(&self) -> &str {
            &self.handler_name
        }

        fn description(&self) -> &str {
            "mock handler"
        }
    }

    struct FailingHandler;

    #[async_trait]
    impl HookHandler for FailingHandler {
        async fn execute(&self, _context: &HookContext) -> Result<HookResult, HookError> {
            Err(HookError::ExecutionFailed("test error".into()))
        }

        fn name(&self) -> &str {
            "failing"
        }

        fn description(&self) -> &str {
            "handler that always fails"
        }
    }

    struct RecordingLogger {
        start_count: Arc<AtomicUsize>,
        end_count: Arc<AtomicUsize>,
        error_count: Arc<AtomicUsize>,
    }

    impl RecordingLogger {
        fn new() -> (Self, Arc<AtomicUsize>, Arc<AtomicUsize>, Arc<AtomicUsize>) {
            let start = Arc::new(AtomicUsize::new(0));
            let end = Arc::new(AtomicUsize::new(0));
            let error = Arc::new(AtomicUsize::new(0));
            (
                Self {
                    start_count: start.clone(),
                    end_count: end.clone(),
                    error_count: error.clone(),
                },
                start,
                end,
                error,
            )
        }
    }

    impl Logger for RecordingLogger {
        fn log_hook_start(&self, _hook_type: &HookType, _handler_name: &str) {
            self.start_count.fetch_add(1, Ordering::SeqCst);
        }
        fn log_hook_end(
            &self,
            _hook_type: &HookType,
            _handler_name: &str,
            _duration: Duration,
        ) {
            self.end_count.fetch_add(1, Ordering::SeqCst);
        }
        fn log_hook_error(
            &self,
            _hook_type: &HookType,
            _handler_name: &str,
            _error: &HookError,
        ) {
            self.error_count.fetch_add(1, Ordering::SeqCst);
        }
    }

    fn make_logger() -> (Arc<dyn Logger>, Arc<AtomicUsize>, Arc<AtomicUsize>, Arc<AtomicUsize>) {
        let (logger, start, end, error) = RecordingLogger::new();
        (Arc::new(logger), start, end, error)
    }

    #[tokio::test]
    async fn test_emit_calls_subscribed_handler() {
        let (logger, _, _, _) = make_logger();
        let bus = DefaultEventBus::new(logger);
        let call_count = Arc::new(AtomicUsize::new(0));
        let handler: Arc<dyn HookHandler> = Arc::new(MockHandler::new("test", call_count.clone()));

        bus.subscribe(HookType::PreCommit, handler).await;

        let context = HookContext::new(HookType::PreCommit, "/repo".into());
        let event = HookEvent::new(HookType::PreCommit, context);
        let results = bus.emit(event).await;

        assert_eq!(call_count.load(Ordering::SeqCst), 1);
        assert_eq!(results.len(), 1);
        assert!(results[0].is_ok());
    }

    #[tokio::test]
    async fn test_emit_does_not_call_handler_for_different_hook_type() {
        let (logger, _, _, _) = make_logger();
        let bus = DefaultEventBus::new(logger);
        let call_count = Arc::new(AtomicUsize::new(0));
        let handler: Arc<dyn HookHandler> = Arc::new(MockHandler::new("test", call_count.clone()));

        bus.subscribe(HookType::PreCommit, handler).await;

        let context = HookContext::new(HookType::PostCommit, "/repo".into());
        let event = HookEvent::new(HookType::PostCommit, context);
        let results = bus.emit(event).await;

        assert_eq!(call_count.load(Ordering::SeqCst), 0);
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_unsubscribe_removes_handler() {
        let (logger, _, _, _) = make_logger();
        let bus = DefaultEventBus::new(logger);
        let call_count = Arc::new(AtomicUsize::new(0));
        let handler: Arc<dyn HookHandler> = Arc::new(MockHandler::new("test", call_count.clone()));

        bus.subscribe(HookType::PreCommit, handler).await;
        bus.unsubscribe(HookType::PreCommit, "test").await;

        let context = HookContext::new(HookType::PreCommit, "/repo".into());
        let event = HookEvent::new(HookType::PreCommit, context);
        let results = bus.emit(event).await;

        assert_eq!(call_count.load(Ordering::SeqCst), 0);
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_emit_logs_success() {
        let (logger, start_count, end_count, error_count) = make_logger();
        let bus = DefaultEventBus::new(logger);
        let handler: Arc<dyn HookHandler> = Arc::new(MockHandler::new("test", Arc::new(AtomicUsize::new(0))));

        bus.subscribe(HookType::PreCommit, handler).await;

        let context = HookContext::new(HookType::PreCommit, "/repo".into());
        let event = HookEvent::new(HookType::PreCommit, context);
        bus.emit(event).await;

        assert_eq!(start_count.load(Ordering::SeqCst), 1);
        assert_eq!(end_count.load(Ordering::SeqCst), 1);
        assert_eq!(error_count.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn test_emit_logs_error_on_failure() {
        let (logger, start_count, end_count, error_count) = make_logger();
        let bus = DefaultEventBus::new(logger);
        let handler: Arc<dyn HookHandler> = Arc::new(FailingHandler);

        bus.subscribe(HookType::PreCommit, handler).await;

        let context = HookContext::new(HookType::PreCommit, "/repo".into());
        let event = HookEvent::new(HookType::PreCommit, context);
        let results = bus.emit(event).await;

        assert_eq!(start_count.load(Ordering::SeqCst), 1);
        assert_eq!(end_count.load(Ordering::SeqCst), 0);
        assert_eq!(error_count.load(Ordering::SeqCst), 1);
        assert_eq!(results.len(), 1);
        assert!(results[0].is_err());
    }

    #[tokio::test]
    async fn test_multiple_handlers_for_same_type() {
        let (logger, _, _, _) = make_logger();
        let bus = DefaultEventBus::new(logger);
        let count1 = Arc::new(AtomicUsize::new(0));
        let count2 = Arc::new(AtomicUsize::new(0));

        bus.subscribe(
            HookType::PreCommit,
            Arc::new(MockHandler::new("h1", count1.clone())),
        ).await;
        bus.subscribe(
            HookType::PreCommit,
            Arc::new(MockHandler::new("h2", count2.clone())),
        ).await;

        let context = HookContext::new(HookType::PreCommit, "/repo".into());
        let event = HookEvent::new(HookType::PreCommit, context);
        let results = bus.emit(event).await;

        assert_eq!(count1.load(Ordering::SeqCst), 1);
        assert_eq!(count2.load(Ordering::SeqCst), 1);
        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
    }
}
