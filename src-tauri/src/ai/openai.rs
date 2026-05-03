use super::AiProvider;
use crate::common::AppError;
use crate::svn::models::ReviewChunkEvent;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

pub struct OpenAiProvider {
    client: Client,
    endpoint: String,
    api_key: String,
    model: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

#[derive(Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Deserialize)]
struct StreamChunk {
    choices: Vec<StreamChoice>,
}

#[derive(Deserialize)]
struct StreamChoice {
    delta: Option<StreamDelta>,
}

#[derive(Deserialize)]
struct StreamDelta {
    content: Option<String>,
}

/// Max diff size to send to AI (~100KB, roughly 25k tokens).
const MAX_DIFF_BYTES: usize = 100 * 1024;

fn truncate_diff(diff: &str) -> String {
    if diff.len() <= MAX_DIFF_BYTES {
        diff.to_string()
    } else {
        let truncated = &diff[..MAX_DIFF_BYTES];
        format!(
            "{}\n\n[... diff truncated at {}KB — total size: {}KB ...]",
            truncated,
            MAX_DIFF_BYTES / 1024,
            diff.len() / 1024,
        )
    }
}

/// Process a single SSE data line. Returns `Ok(true)` if `[DONE]` was received.
fn process_sse_line(line: &str, app_handle: &AppHandle) -> Result<bool, AppError> {
    if !line.starts_with("data: ") {
        return Ok(false);
    }
    let data = &line[6..];
    if data == "[DONE]" {
        let _ = app_handle.emit(
            "review_chunk",
            ReviewChunkEvent {
                content: String::new(),
                done: true,
            },
        );
        return Ok(true);
    }
    if let Ok(chunk) = serde_json::from_str::<StreamChunk>(data) {
        if let Some(choice) = chunk.choices.first() {
            if let Some(delta) = &choice.delta {
                if let Some(content) = &delta.content {
                    let _ = app_handle.emit(
                        "review_chunk",
                        ReviewChunkEvent {
                            content: content.clone(),
                            done: false,
                        },
                    );
                }
            }
        }
    }
    Ok(false)
}

impl OpenAiProvider {
    pub fn new(endpoint: &str, api_key: &str, model: &str, client: &Client) -> Self {
        Self {
            client: client.clone(),
            endpoint: endpoint.to_string(),
            api_key: api_key.to_string(),
            model: model.to_string(),
        }
    }

    async fn chat_completion(&self, messages: Vec<ChatMessage>) -> Result<String, AppError> {
        let request = ChatRequest {
            model: self.model.clone(),
            messages,
            stream: false,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.endpoint))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Ai(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Ai(format!("API error {}: {}", status, body)));
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .map_err(|e| AppError::Ai(format!("Failed to parse response: {}", e)))?;

        chat_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| AppError::Ai("No response from AI".to_string()))
    }
}

#[async_trait]
impl AiProvider for OpenAiProvider {
    async fn generate_message(&self, diff: &str) -> Result<String, AppError> {
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are a helpful assistant that generates concise commit messages for code changes. Output ONLY the commit message, no explanation.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: format!("Generate a concise commit message for these changes:\n\n{}", truncate_diff(diff)),
            },
        ];
        self.chat_completion(messages).await
    }

    async fn review_changes(&self, diff: &str, app_handle: &AppHandle) -> Result<(), AppError> {
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are a senior code reviewer. Review the following code changes and provide constructive feedback on potential issues, bugs, and improvements. Be concise.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: format!("Review these code changes:\n\n{}", truncate_diff(diff)),
            },
        ];

        let request = ChatRequest {
            model: self.model.clone(),
            messages,
            stream: true,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.endpoint))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Ai(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Ai(format!("API error {}: {}", status, body)));
        }

        let mut buffer = String::new();
        let mut stream = response.bytes_stream();

        use futures_util::StreamExt;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| AppError::Ai(format!("Stream error: {}", e)))?;
            let text = String::from_utf8_lossy(&chunk);
            buffer.push_str(&text);

            while let Some(line_end) = buffer.find('\n') {
                let line = buffer[..line_end].trim().to_string();
                buffer = buffer[line_end + 1..].to_string();

                if process_sse_line(&line, app_handle)? {
                    return Ok(());
                }
            }
        }

        // Process any remaining data in buffer (last line may not have trailing newline)
        if !buffer.trim().is_empty() {
            let _ = process_sse_line(buffer.trim(), app_handle)?;
        }

        let _ = app_handle.emit(
            "review_chunk",
            ReviewChunkEvent {
                content: String::new(),
                done: true,
            },
        );
        Ok(())
    }
}
