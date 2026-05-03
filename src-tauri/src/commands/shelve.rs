use crate::app_state::AppState;
use crate::shelve;
use crate::svn::models::ShelveInfo;
use tauri::State;

#[tauri::command]
pub async fn shelve_save(
    state: State<'_, AppState>,
    path: String,
    name: String,
) -> Result<(), String> {
    shelve::validate_shelve_name(&name)?;
    let timeout = {
        let config = state.config.read().map_err(|e| e.to_string())?;
        config.advanced.svn_timeout_secs
    };
    shelve::shelve_save(&path, &name, timeout)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn shelve_list(path: String) -> Result<Vec<ShelveInfo>, String> {
    shelve::shelve_list(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn shelve_apply(
    state: State<'_, AppState>,
    path: String,
    name: String,
) -> Result<(), String> {
    let timeout = {
        let config = state.config.read().map_err(|e| e.to_string())?;
        config.advanced.svn_timeout_secs
    };
    shelve::shelve_apply(&path, &name, timeout)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn shelve_delete(path: String, name: String) -> Result<(), String> {
    shelve::validate_shelve_name(&name)?;
    shelve::shelve_delete(&path, &name).map_err(|e| e.to_string())
}
