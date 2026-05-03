use crate::ai;
use crate::app_state::AppState;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn generate_commit_message(
    state: State<'_, AppState>,
    diff: String,
) -> Result<String, String> {
    let (api_key, provider_type, endpoint, model) = {
        let config = state.config.read().map_err(|e| e.to_string())?;
        (
            config.ai.api_key.clone(),
            config.ai.provider.clone(),
            config.ai.endpoint.clone(),
            config.ai.model.clone(),
        )
    };

    if api_key.is_empty() {
        return Err("[AI] API key not configured".to_string());
    }

    let provider = ai::create_provider(&provider_type, &endpoint, &api_key, &model, &state.http_client)
        .map_err(|e| e.to_string())?;
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
    let (api_key, provider_type, endpoint, model) = {
        let config = state.config.read().map_err(|e| e.to_string())?;
        (
            config.ai.api_key.clone(),
            config.ai.provider.clone(),
            config.ai.endpoint.clone(),
            config.ai.model.clone(),
        )
    };

    if api_key.is_empty() {
        return Err("[AI] API key not configured".to_string());
    }

    let provider = ai::create_provider(&provider_type, &endpoint, &api_key, &model, &state.http_client)
        .map_err(|e| e.to_string())?;
    provider
        .review_changes(&diff, &app_handle)
        .await
        .map_err(|e| e.to_string())
}
