pub mod ai;
pub mod app_state;
pub mod common;
pub mod commands;
pub mod config;
pub mod shelve;
pub mod svn;

use app_state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let state = AppState::new();
            let window = app
                .get_webview_window("main")
                .expect("failed to get main window");
            {
                let config = state.config.lock().expect("lock poisoned");
                if config.window.maximized {
                    let _ = window.maximize();
                }
            }
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::svn::svn_status,
            commands::svn::svn_info,
            commands::svn::svn_log,
            commands::svn::svn_diff,
            commands::svn::svn_commit,
            commands::svn::svn_list,
            commands::svn::svn_cat,
            commands::svn::svn_checkout,
            commands::svn::svn_update,
            commands::svn::svn_detect_executable,
            commands::ai::generate_commit_message,
            commands::ai::review_changes,
            commands::shelve::shelve_save,
            commands::shelve::shelve_list,
            commands::shelve::shelve_apply,
            commands::shelve::shelve_delete,
            commands::config::get_config,
            commands::config::set_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
