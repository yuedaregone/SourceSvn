use crate::ai;
use crate::app_state::AppState;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn generate_commit_message(
    state: State<'_, AppState>,
    diff: String,
) -> Result<String, String> {
    let (api_key, endpoint, model, timeout_secs) = {
        let config = state.config.lock().map_err(|e| e.to_string())?;
        (
            config.ai.api_key.clone(),
            config.ai.endpoint.clone(),
            config.ai.model.clone(),
            config.ai.timeout_secs,
        )
    };

    if api_key.is_empty() {
        return Err("[AI] API key not configured".to_string());
    }

    let provider = ai::create_provider(&endpoint, &api_key, &model, timeout_secs);
    provider
        .generate_message(&diff)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn review_changes(
    state: State<'_, AppState>,
    diff: String,
    app_handle: AppHandle,
) -> Result<(), String> {
    let (api_key, endpoint, model, timeout_secs) = {
        let config = state.config.lock().map_err(|e| e.to_string())?;
        (
            config.ai.api_key.clone(),
            config.ai.endpoint.clone(),
            config.ai.model.clone(),
            config.ai.timeout_secs,
        )
    };

    if api_key.is_empty() {
        return Err("[AI] API key not configured".to_string());
    }

    let provider = ai::create_provider(&endpoint, &api_key, &model, timeout_secs);
    provider
        .review_changes(&diff, &app_handle)
        .await
        .map_err(|e| e.to_string())
}
