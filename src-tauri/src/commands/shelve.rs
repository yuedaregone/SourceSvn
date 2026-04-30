use crate::common::ShelveInfo;
use crate::shelve;

#[tauri::command]
pub fn shelve_save(path: String, name: String) -> Result<(), String> {
    shelve::shelve_save(&path, &name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn shelve_list(path: String) -> Result<Vec<ShelveInfo>, String> {
    shelve::shelve_list(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn shelve_apply(path: String, name: String) -> Result<(), String> {
    shelve::shelve_apply(&path, &name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn shelve_delete(path: String, name: String) -> Result<(), String> {
    shelve::shelve_delete(&path, &name).map_err(|e| e.to_string())
}
