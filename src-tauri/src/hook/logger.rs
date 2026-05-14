use std::time::Duration;

use super::types::*;

pub trait Logger: Send + Sync {
    fn log_hook_start(&self, hook_type: &HookType, handler_name: &str);
    fn log_hook_end(&self, hook_type: &HookType, handler_name: &str, duration: Duration);
    fn log_hook_error(&self, hook_type: &HookType, handler_name: &str, error: &HookError);
}

pub struct NoopLogger;

impl Logger for NoopLogger {
    fn log_hook_start(&self, _hook_type: &HookType, _handler_name: &str) {}
    fn log_hook_end(&self, _hook_type: &HookType, _handler_name: &str, _duration: Duration) {}
    fn log_hook_error(&self, _hook_type: &HookType, _handler_name: &str, _error: &HookError) {}
}
