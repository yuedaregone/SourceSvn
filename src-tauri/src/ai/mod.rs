pub mod openai;

use crate::common::AppError;
use async_trait::async_trait;
use tauri::AppHandle;

#[async_trait]
pub trait AiProvider: Send + Sync {
    async fn chat(&self, system: &str, user: &str) -> Result<String, AppError>;
    async fn chat_stream(
        &self,
        system: &str,
        user: &str,
        app_handle: &AppHandle,
        event_name: &str,
    ) -> Result<(), AppError>;
}

pub fn create_provider(
    provider_type: &str,
    endpoint: &str,
    api_key: &str,
    model: &str,
    client: &reqwest::Client,
) -> Result<Box<dyn AiProvider>, AppError> {
    match provider_type {
        "openai" => Ok(Box::new(openai::OpenAiProvider::new(
            endpoint, api_key, model, client,
        ))),
        _ => Err(AppError::Ai(format!(
            "Unsupported AI provider: '{}'. Only 'openai' is currently supported.",
            provider_type
        ))),
    }
}
