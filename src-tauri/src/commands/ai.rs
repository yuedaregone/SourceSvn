use crate::ai;
use crate::config::load_config;
use tauri::AppHandle;

#[tauri::command]
pub async fn generate_commit_message(diff: String) -> Result<String, String> {
    let config = load_config();
    if config.ai.api_key.is_empty() {
        return Err("[AI] API key not configured".to_string());
    }

    let provider = ai::create_provider(
        &config.ai.endpoint,
        &config.ai.api_key,
        &config.ai.model,
        config.ai.timeout_secs,
    );

    provider.generate_message(&diff).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn review_changes(diff: String, app_handle: AppHandle) -> Result<(), String> {
    let config = load_config();
    if config.ai.api_key.is_empty() {
        return Err("[AI] API key not configured".to_string());
    }

    let provider = ai::create_provider(
        &config.ai.endpoint,
        &config.ai.api_key,
        &config.ai.model,
        config.ai.timeout_secs,
    );

    provider.review_changes(&diff, &app_handle).await.map_err(|e| e.to_string())
}
