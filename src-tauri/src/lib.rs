pub mod ai;
pub mod app_state;
pub mod common;
pub mod commands;
pub mod config;
pub mod hook;
pub mod shelve;
pub mod svn;

use app_state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    let result = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let state = AppState::new(Some(app.handle().clone()));
            let window = app
                .get_webview_window("main")
                .ok_or("failed to get main window")?;
            {
                let config = state.config.read().map_err(|e| e.to_string())?;
                if config.window.maximized {
                    let _ = window.maximize();
                }
                if let Some(ref exe) = config.svn.executable {
                    if !exe.is_empty() {
                        svn::set_svn_path(exe.clone());
                    }
                }
            }
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::svn::svn_status,
            commands::svn::svn_info,
            commands::svn::svn_log,
            commands::svn::svn_log_server,
            commands::svn::svn_log_changed_paths,
            commands::svn::svn_diff,
            commands::svn::svn_commit,
            commands::svn::svn_list,
            commands::svn::svn_cat,
            commands::svn::svn_checkout,
            commands::svn::svn_update,
            commands::svn::svn_cleanup,
            commands::svn::svn_detect_executable,
            commands::svn::diff_unversioned_file,
            commands::svn::read_local_file,
            commands::svn::svn_cat_at_revision,
            commands::svn::svn_cat_in_dir,
            commands::svn::svn_revert,
            commands::svn::svn_add,
            commands::svn::svn_delete,
            commands::svn::svn_resolve,
            commands::svn::svn_blame,
            commands::svn::svn_update_to_revision,
            commands::svn::delete_files_from_disk,
            commands::svn::open_in_system,
            commands::svn::open_file_with_default_app,
            commands::svn::find_svn_root,
            commands::svn::file_size_diff,
            commands::ai::generate_commit_message,
            commands::ai::review_changes,
            commands::shelve::shelve_save,
            commands::shelve::shelve_list,
            commands::shelve::shelve_apply,
            commands::shelve::shelve_delete,
            commands::shelve::shelve_rename,
            commands::config::get_config,
            commands::config::set_config,
            commands::hook::hook_subscribe,
            commands::hook::hook_unsubscribe,
            commands::hook::hook_emit,
            commands::hook::hook_load_config,
            commands::hook::hook_save_config,
            commands::hook::hook_add_handler,
            commands::hook::hook_remove_handler,
            commands::hook::hook_update_handler,
        ])
        .run(tauri::generate_context!());

    if let Err(e) = result {
        eprintln!("Application error: {}", e);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::hook::*;
    use crate::hook::event_bus::HookHandler;
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    struct TestHandler {
        name: String,
        call_count: Arc<AtomicUsize>,
    }

    impl TestHandler {
        fn new(name: &str, call_count: Arc<AtomicUsize>) -> Self {
            Self {
                name: name.to_string(),
                call_count,
            }
        }
    }

    #[async_trait]
    impl HookHandler for TestHandler {
        async fn execute(&self, _context: &HookContext) -> Result<HookResult, HookError> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            Ok(HookResult::Continue)
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            "integration test handler"
        }
    }

    #[test]
    fn test_config_save_and_load_roundtrip() {
        let dir = std::env::temp_dir().join("sourcesvn_integration_test_config");
        let _ = std::fs::remove_dir_all(&dir);

        let manager = FileHookConfigManager::new(dir.join("hooks.toml"));

        let config = HooksConfig {
            enabled: true,
            handlers: vec![
                HookHandlerConfig {
                    name: "test-handler".to_string(),
                    script_path: "/path/to/script.js".to_string(),
                    enabled: true,
                },
                HookHandlerConfig {
                    name: "pre-handler".to_string(),
                    script_path: "/other/script.sh".to_string(),
                    enabled: false,
                },
            ],
        };

        manager.save_config(&config).unwrap();
        let loaded = manager.load_config().unwrap();

        assert!(loaded.enabled);
        assert_eq!(loaded.handlers.len(), 2);
        assert_eq!(loaded.handlers[0].name, "test-handler");
        assert_eq!(loaded.handlers[0].script_path, "/path/to/script.js");
        assert!(loaded.handlers[0].enabled);
        assert_eq!(loaded.handlers[1].name, "pre-handler");
        assert!(!loaded.handlers[1].enabled);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn test_event_bus_subscriber_receives_events() {
        let state = AppState::new(None);
        let call_count = Arc::new(AtomicUsize::new(0));
        let handler: Arc<dyn HookHandler> =
            Arc::new(TestHandler::new("integration-handler", call_count.clone()));

        state
            .hook_event_bus
            .subscribe(handler)
            .await;

        let context = HookContext::new(HookType::PostCommit, "/test/path".to_string());
        let event = HookEvent::new(HookType::PostCommit, context);
        let results = state.hook_event_bus.emit(event).await;

        assert_eq!(call_count.load(Ordering::SeqCst), 1);
        assert_eq!(results.len(), 1);
        assert!(results[0].is_ok());
    }

    #[tokio::test]
    async fn test_event_bus_handler_called_for_any_type() {
        let state = AppState::new(None);
        let call_count = Arc::new(AtomicUsize::new(0));
        let handler: Arc<dyn HookHandler> =
            Arc::new(TestHandler::new("handler", call_count.clone()));

        state
            .hook_event_bus
            .subscribe(handler)
            .await;

        let context = HookContext::new(HookType::PreCommit, "/test/path".to_string());
        let event = HookEvent::new(HookType::PreCommit, context);
        let results = state.hook_event_bus.emit(event).await;

        assert_eq!(call_count.load(Ordering::SeqCst), 1);
        assert_eq!(results.len(), 1);
        assert!(results[0].is_ok());
    }
}
