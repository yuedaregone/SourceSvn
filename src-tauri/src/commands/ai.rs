use crate::ai;
use crate::app_state::AppState;
use tauri::{AppHandle, State};

struct AiSetup {
    api_key: String,
    provider_type: String,
    endpoint: String,
    model: String,
}

fn read_ai_config(state: &State<'_, AppState>) -> Result<AiSetup, String> {
    let config = state.config.read().map_err(|e| e.to_string())?;
    Ok(AiSetup {
        api_key: config.ai.api_key.clone(),
        provider_type: config.ai.provider.clone(),
        endpoint: config.ai.endpoint.clone(),
        model: config.ai.model.clone(),
    })
}

fn create_ai_provider(state: &State<'_, AppState>) -> Result<(Box<dyn ai::AiProvider>, AiSetup), String> {
    let setup = read_ai_config(state)?;
    if setup.api_key.is_empty() {
        return Err("[AI] API key not configured".into());
    }
    let provider = ai::create_provider(&setup.provider_type, &setup.endpoint, &setup.api_key, &setup.model, state.http_client())
        .map_err(|e| e.to_string())?;
    Ok((provider, setup))
}

#[tauri::command]
pub async fn generate_commit_message(
    state: State<'_, AppState>,
    diff: String,
) -> Result<String, String> {
    let (provider, _) = create_ai_provider(&state)?;
    let system = {
        let config = state.config.read().map_err(|e| e.to_string())?;
        config.ai.commit_prompt.clone()
    };
    let user = format!("Generate a concise commit message for these changes:\n\n{}", diff);
    provider.chat(&system, &user).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn review_changes(
    state: State<'_, AppState>,
    diff: String,
    app_handle: AppHandle,
) -> Result<(), String> {
    let (provider, _) = create_ai_provider(&state)?;
    let system = {
        let config = state.config.read().map_err(|e| e.to_string())?;
        config.ai.review_prompt.clone()
    };
    let user = format!("Review these code changes:\n\n{}", diff);
    provider.chat_stream(&system, &user, &app_handle, "review_chunk").await.map_err(|e| e.to_string())
}
