use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
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

        let path = std::path::Path::new(script_path);
        let output = match path.extension().and_then(|e| e.to_str()) {
            Some("js") | Some("mjs") => {
                Command::new("node")
                    .arg(script_path)
                    .arg(&context_json)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .output()
                    .await
                    .map_err(|e| HookError::ExecutionFailed(e.to_string()))?
            }
            _ => {
                Command::new(script_path)
                    .arg(&context_json)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .output()
                    .await
                    .map_err(|e| HookError::ExecutionFailed(e.to_string()))?
            }
        };

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

pub struct JsScriptExecutor {
    app_handle: Option<tauri::AppHandle>,
}

impl JsScriptExecutor {
    pub fn new(app_handle: Option<tauri::AppHandle>) -> Self {
        Self { app_handle }
    }
}

fn execute_js(
    script_content: String,
    context_json: String,
    app_handle: Option<tauri::AppHandle>,
    rt_handle: tokio::runtime::Handle,
) -> Result<HookResult, HookError> {
    use boa_engine::js_string;
    use boa_engine::{Context, JsValue, Source};

    let mut context = Context::default();
    let globals = context.global_object();

    let context_val = JsValue::from(js_string!(context_json.clone()));
    globals
        .set(js_string!("__context_json"), context_val, false, &mut context)
        .map_err(|e| HookError::ExecutionFailed(format!("Set context: {}", e)))?;

    context
        .eval(Source::from_bytes("var __hook_context = JSON.parse(__context_json);"))
        .map_err(|e| HookError::ExecutionFailed(format!("Parse context: {}", e)))?;

    // Set 'context' as alias for '__hook_context' using globals.set to avoid var redefinition
    let hook_context_val = context
        .eval(Source::from_bytes("__hook_context"))
        .map_err(|e| HookError::ExecutionFailed(format!("Get hook context: {}", e)))?;
    globals
        .set(js_string!("context"), hook_context_val, true, &mut context)
        .map_err(|e| HookError::ExecutionFailed(format!("Set context alias: {}", e)))?;

    let app_handle_clone = app_handle.clone();
    // SAFETY: AppHandle is not GC-traced, safe to capture
    let toast_fn = unsafe {
        boa_engine::NativeFunction::from_closure(move |_this, args, ctx| {
            let t = args
                .get(0)
                .and_then(|v| v.to_string(ctx).ok())
                .map(|s| s.to_std_string_escaped())
                .unwrap_or_default();
            let m = args
                .get(1)
                .and_then(|v| v.to_string(ctx).ok())
                .map(|s| s.to_std_string_escaped())
                .unwrap_or_default();
            if let Some(ref h) = app_handle_clone {
                use tauri::Emitter;
                let _ = h.emit(
                    "hook-toast",
                    serde_json::json!({ "type": t, "message": m }),
                );
            }
            Ok(JsValue::undefined())
        })
    };
    let toast_val = JsValue::from(toast_fn.to_js_function(context.realm()));
    globals
        .set(js_string!("__toast_raw"), toast_val, false, &mut context)
        .map_err(|e| HookError::ExecutionFailed(format!("set toast: {}", e)))?;

    let h = rt_handle.clone();
    // SAFETY: tokio::runtime::Handle is not GC-traced
    let status_fn = unsafe {
        boa_engine::NativeFunction::from_closure(move |_this, args, ctx| {
            let path = args
                .get(0)
                .and_then(|v| v.to_string(ctx).ok())
                .map(|s| s.to_std_string_escaped())
                .unwrap_or_default();
            let result = h.block_on(crate::svn::status::svn_status(&path, 30));
            let json = match result {
                Ok(statuses) => serde_json::to_string(&serde_json::json!({"ok": true, "data": statuses}))
                    .unwrap_or_else(|_| r#"{"ok":false,"error":"serialize"}"#.into()),
                Err(e) => serde_json::to_string(&serde_json::json!({"ok": false, "error": e.to_string()}))
                    .unwrap_or_default(),
            };
            Ok(JsValue::from(js_string!(json)))
        })
    };
    let status_val = JsValue::from(status_fn.to_js_function(context.realm()));
    globals
        .set(js_string!("__svn_status_raw"), status_val, false, &mut context)
        .map_err(|e| HookError::ExecutionFailed(format!("set svnStatus: {}", e)))?;

    let h = rt_handle.clone();
    // SAFETY: tokio::runtime::Handle is not GC-traced
    let log_fn = unsafe {
        boa_engine::NativeFunction::from_closure(move |_this, args, ctx| {
            let path = args
                .get(0)
                .and_then(|v| v.to_string(ctx).ok())
                .map(|s| s.to_std_string_escaped())
                .unwrap_or_default();
            let limit = args
                .get(1)
                .and_then(|v| v.to_number(ctx).ok())
                .map(|n| n as u32)
                .unwrap_or(10);
            let result = h.block_on(crate::svn::log::svn_log(&path, Some(limit), None, 30));
            let json = match result {
                Ok(entries) => serde_json::to_string(&serde_json::json!({"ok": true, "data": entries}))
                    .unwrap_or_else(|_| r#"{"ok":false,"error":"serialize"}"#.into()),
                Err(e) => serde_json::to_string(&serde_json::json!({"ok": false, "error": e.to_string()}))
                    .unwrap_or_default(),
            };
            Ok(JsValue::from(js_string!(json)))
        })
    };
    let log_val = JsValue::from(log_fn.to_js_function(context.realm()));
    globals
        .set(js_string!("__svn_log_raw"), log_val, false, &mut context)
        .map_err(|e| HookError::ExecutionFailed(format!("set svnLog: {}", e)))?;

    let h = rt_handle.clone();
    // SAFETY: tokio::runtime::Handle is not GC-traced
    let info_fn = unsafe {
        boa_engine::NativeFunction::from_closure(move |_this, args, ctx| {
            let path = args
                .get(0)
                .and_then(|v| v.to_string(ctx).ok())
                .map(|s| s.to_std_string_escaped())
                .unwrap_or_default();
            let result = h.block_on(crate::svn::info::svn_info(&path, 30));
            let json = match result {
                Ok(info) => serde_json::to_string(&serde_json::json!({"ok": true, "data": info}))
                    .unwrap_or_else(|_| r#"{"ok":false,"error":"serialize"}"#.into()),
                Err(e) => serde_json::to_string(&serde_json::json!({"ok": false, "error": e.to_string()}))
                    .unwrap_or_default(),
            };
            Ok(JsValue::from(js_string!(json)))
        })
    };
    let info_val = JsValue::from(info_fn.to_js_function(context.realm()));
    globals
        .set(js_string!("__svn_info_raw"), info_val, false, &mut context)
        .map_err(|e| HookError::ExecutionFailed(format!("set svnInfo: {}", e)))?;

    // SAFETY: no captures
    let log_raw_fn = unsafe {
        boa_engine::NativeFunction::from_closure(|_this, args, ctx| {
            let level = args
                .get(0)
                .and_then(|v| v.to_string(ctx).ok())
                .map(|s| s.to_std_string_escaped())
                .unwrap_or_default();
            let msg = args
                .get(1)
                .and_then(|v| v.to_string(ctx).ok())
                .map(|s| s.to_std_string_escaped())
                .unwrap_or_default();
            eprintln!("[hook:{}] {}", level, msg);
            Ok(JsValue::undefined())
        })
    };
    let log_raw_val = JsValue::from(log_raw_fn.to_js_function(context.realm()));
    globals
        .set(js_string!("__log_raw"), log_raw_val, false, &mut context)
        .map_err(|e| HookError::ExecutionFailed(format!("set log: {}", e)))?;

    let preamble = r#"
var __hook_cancelled = false;
var __hook_cancel_reason = "";
var __hook_modified_json = null;

function cancel(reason) {
  __hook_cancelled = true;
  __hook_cancel_reason = reason || "";
}

function modify(data) {
  __hook_modified_json = JSON.stringify(data);
}

function toast(type, message) {
  __toast_raw(type, message);
}

function svnStatus(path) {
  var raw = __svn_status_raw(path);
  var parsed = JSON.parse(raw);
  if (!parsed.ok) throw new Error(parsed.error);
  return parsed.data;
}

function svnLog(path, limit) {
  var raw = __svn_log_raw(path, limit || 10);
  var parsed = JSON.parse(raw);
  if (!parsed.ok) throw new Error(parsed.error);
  return parsed.data;
}

function svnInfo(path) {
  var raw = __svn_info_raw(path);
  var parsed = JSON.parse(raw);
  if (!parsed.ok) throw new Error(parsed.error);
  return parsed.data;
}

function log(level, message) {
  __log_raw(level, message);
}
"#;

    context
        .eval(Source::from_bytes(preamble))
        .map_err(|e| HookError::ExecutionFailed(format!("Preamble: {}", e)))?;

    context
        .eval(Source::from_bytes(script_content.as_str()))
        .map_err(|e| HookError::ExecutionFailed(format!("Script error: {}", e)))?;

    let cancelled = context
        .eval(Source::from_bytes("__hook_cancelled"))
        .ok()
        .and_then(|v| v.as_boolean())
        .unwrap_or(false);

    if cancelled {
        return Ok(HookResult::Cancel);
    }

    let modified_json = context
        .eval(Source::from_bytes("__hook_modified_json"))
        .ok()
        .and_then(|v| {
            if v.is_null_or_undefined() {
                None
            } else {
                v.to_string(&mut context)
                    .ok()
                    .map(|s| s.to_std_string_escaped())
            }
        });

    if let Some(json_str) = modified_json {
        if let Ok(map) = serde_json::from_str::<HashMap<String, serde_json::Value>>(&json_str) {
            return Ok(HookResult::Modify(map));
        }
    }

    Ok(HookResult::Continue)
}

#[async_trait]
impl ScriptExecutor for JsScriptExecutor {
    async fn execute(&self, script_path: &str, context: &HookContext) -> Result<HookResult, HookError> {
        let script_content = tokio::fs::read_to_string(script_path)
            .await
            .map_err(|_| HookError::ScriptNotFound(script_path.to_string()))?;

        let context_json = serde_json::to_string(context)
            .map_err(|e| HookError::ExecutionFailed(format!("Context serialization: {}", e)))?;

        let app_handle = self.app_handle.clone();
        let rt_handle = tokio::runtime::Handle::current();

        let result = tokio::time::timeout(
            Duration::from_secs(30),
            tokio::task::spawn_blocking(move || {
                execute_js(script_content, context_json, app_handle, rt_handle)
            }),
        )
        .await
        .map_err(|_| HookError::TimeoutExpired)?;

        result.map_err(|e| HookError::ExecutionFailed(format!("Task join: {}", e)))?
    }

    fn supports(&self, script_path: &str) -> bool {
        let path = Path::new(script_path);
        if !path.exists() || !path.is_file() {
            return false;
        }
        matches!(path.extension().and_then(|e| e.to_str()), Some("js") | Some("mjs"))
    }
}

pub struct ScriptExecutorManager {
    executors: Vec<Box<dyn ScriptExecutor>>,
}

impl ScriptExecutorManager {
    pub fn new(app_handle: Option<tauri::AppHandle>) -> Self {
        Self {
            executors: vec![
                Box::new(JsScriptExecutor::new(app_handle)),
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

    #[test]
    fn js_executor_supports_js_files() {
        let executor = JsScriptExecutor::new(None);
        let dir = std::env::temp_dir().join("sourcesvn_js_test");
        let _ = std::fs::create_dir_all(&dir);
        let js_path = dir.join("test.js");
        std::fs::write(&js_path, "1 + 1").unwrap();
        assert!(executor.supports(js_path.to_str().unwrap()));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn js_executor_supports_mjs_files() {
        let executor = JsScriptExecutor::new(None);
        let dir = std::env::temp_dir().join("sourcesvn_mjs_test");
        let _ = std::fs::create_dir_all(&dir);
        let js_path = dir.join("test.mjs");
        std::fs::write(&js_path, "1 + 1").unwrap();
        assert!(executor.supports(js_path.to_str().unwrap()));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn js_executor_rejects_non_js_files() {
        let executor = JsScriptExecutor::new(None);
        assert!(!executor.supports("/path/to/script.sh"));
    }

    #[tokio::test]
    async fn js_executor_returns_continue_for_empty_script() {
        let executor = JsScriptExecutor::new(None);
        let dir = std::env::temp_dir().join("sourcesvn_js_test_empty");
        let _ = std::fs::create_dir_all(&dir);
        let js_path = dir.join("empty.js");
        std::fs::write(&js_path, "// empty script").unwrap();
        let context = HookContext::new(HookType::PreCommit, "/tmp".into());
        let result = executor.execute(js_path.to_str().unwrap(), &context).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), HookResult::Continue);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn js_executor_can_cancel() {
        let executor = JsScriptExecutor::new(None);
        let dir = std::env::temp_dir().join("sourcesvn_js_test_cancel");
        let _ = std::fs::create_dir_all(&dir);
        let js_path = dir.join("cancel.js");
        std::fs::write(&js_path, r#"cancel("user aborted");"#).unwrap();
        let context = HookContext::new(HookType::PreCommit, "/tmp".into());
        let result = executor.execute(js_path.to_str().unwrap(), &context).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), HookResult::Cancel);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn js_executor_can_modify() {
        let executor = JsScriptExecutor::new(None);
        let dir = std::env::temp_dir().join("sourcesvn_js_test_modify");
        let _ = std::fs::create_dir_all(&dir);
        let js_path = dir.join("modify.js");
        std::fs::write(&js_path, r#"modify({"key": "value"});"#).unwrap();
        let context = HookContext::new(HookType::PreCommit, "/tmp".into());
        let result = executor.execute(js_path.to_str().unwrap(), &context).await;
        assert!(result.is_ok());
        match result.unwrap() {
            HookResult::Modify(data) => {
                assert_eq!(data.get("key").unwrap(), &serde_json::json!("value"));
            }
            other => panic!("expected Modify, got {:?}", other),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn js_executor_exposes_context() {
        let executor = JsScriptExecutor::new(None);
        let dir = std::env::temp_dir().join("sourcesvn_js_test_context");
        let _ = std::fs::create_dir_all(&dir);
        let js_path = dir.join("context.js");
        std::fs::write(&js_path, r#"
            if (context.hook_type !== "PreCommit") cancel("wrong type");
            if (context.repo_path !== "/tmp") cancel("wrong path");
        "#).unwrap();
        let context = HookContext::new(HookType::PreCommit, "/tmp".into());
        let result = executor.execute(js_path.to_str().unwrap(), &context).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), HookResult::Continue);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn js_executor_handles_syntax_error() {
        let executor = JsScriptExecutor::new(None);
        let dir = std::env::temp_dir().join("sourcesvn_js_test_syntax");
        let _ = std::fs::create_dir_all(&dir);
        let js_path = dir.join("syntax.js");
        std::fs::write(&js_path, "function { invalid }").unwrap();
        let context = HookContext::new(HookType::PreCommit, "/tmp".into());
        let result = executor.execute(js_path.to_str().unwrap(), &context).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            HookError::ExecutionFailed(msg) => assert!(msg.contains("Script error")),
            other => panic!("expected ExecutionFailed, got {:?}", other),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn manager_returns_not_found_for_missing_script() {
        let manager = ScriptExecutorManager::new(None);
        let context = HookContext::new(HookType::PreCommit, "/tmp".to_string());
        let result = manager.execute("/nonexistent/script", &context).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            HookError::ScriptNotFound(path) => assert_eq!(path, "/nonexistent/script"),
            other => panic!("expected ScriptNotFound, got {:?}", other),
        }
    }
}
