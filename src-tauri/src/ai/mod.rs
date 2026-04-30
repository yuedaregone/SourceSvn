pub mod openai;

use crate::common::AppError;
use async_trait::async_trait;
use tauri::AppHandle;

#[async_trait]
pub trait AiProvider: Send + Sync {
    async fn generate_message(&self, diff: &str) -> Result<String, AppError>;
    async fn review_changes(&self, diff: &str, app_handle: &AppHandle) -> Result<(), AppError>;
}

pub fn create_provider(endpoint: &str, api_key: &str, model: &str, timeout_secs: u64) -> Box<dyn AiProvider> {
    Box::new(openai::OpenAiProvider::new(
        endpoint,
        api_key,
        model,
        timeout_secs,
    ))
}
