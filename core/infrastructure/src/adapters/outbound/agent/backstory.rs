use application::ports::{
    BackstoryGenerationRequest, BackstoryGenerationResponse, BackstoryGeneratorError,
    BackstoryGeneratorPort, BackstoryGeneratorResult,
};
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use serde_json::json;

use super::prompts::build_backstory_task;

const DEFAULT_BASE_URL: &str = "http://127.0.0.1:8000/v1";
const DEFAULT_MODEL: &str = "Qwen/Qwen2.5-7B-Instruct";
const DEFAULT_TEMPERATURE: f32 = 0.75;
const DEFAULT_MAX_TOKENS: u32 = 450;
const SYSTEM_PROMPT: &str = "You are a tabletop RPG continuity writer. You write compact NPC backstories that can be stored as long-term campaign memory.";

/// OpenAI-compatible adapter that talks to a locally hosted Qwen model.
pub struct QwenBackstoryAdapter {
    client: Client,
    base_url: String,
    api_key: Option<String>,
    model: String,
    temperature: f32,
    max_tokens: u32,
}

impl QwenBackstoryAdapter {
    /// Creates the adapter from environment-driven configuration.
    ///
    /// Supported variables:
    /// - `BACKSTORY_LLM_BASE_URL`
    /// - `BACKSTORY_LLM_API_KEY`
    /// - `BACKSTORY_LLM_MODEL`
    /// - `BACKSTORY_LLM_TEMPERATURE`
    /// - `BACKSTORY_LLM_MAX_TOKENS`
    pub fn new() -> Result<Self, reqwest::Error> {
        let client = Client::builder().build()?;
        let base_url = std::env::var("BACKSTORY_LLM_BASE_URL")
            .unwrap_or_else(|_| String::from(DEFAULT_BASE_URL))
            .trim_end_matches('/')
            .to_owned();
        let api_key = std::env::var("BACKSTORY_LLM_API_KEY")
            .ok()
            .filter(|value| !value.trim().is_empty());
        let model =
            std::env::var("BACKSTORY_LLM_MODEL").unwrap_or_else(|_| String::from(DEFAULT_MODEL));
        let temperature = std::env::var("BACKSTORY_LLM_TEMPERATURE")
            .ok()
            .and_then(|value| value.parse::<f32>().ok())
            .unwrap_or(DEFAULT_TEMPERATURE);
        let max_tokens = std::env::var("BACKSTORY_LLM_MAX_TOKENS")
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(DEFAULT_MAX_TOKENS);

        Ok(Self {
            client,
            base_url,
            api_key,
            model,
            temperature,
            max_tokens,
        })
    }

    fn chat_completions_url(&self) -> String {
        format!("{}/chat/completions", self.base_url)
    }
}

#[async_trait::async_trait]
impl BackstoryGeneratorPort for QwenBackstoryAdapter {
    async fn generate_backstory(
        &self,
        request: BackstoryGenerationRequest,
    ) -> BackstoryGeneratorResult<BackstoryGenerationResponse> {
        if request.context_object.title.trim().is_empty()
            || request.context_object.short_desc.trim().is_empty()
        {
            return Err(BackstoryGeneratorError::InvalidQuery {
                details: String::from("NPC title and short_desc must be non-empty"),
            });
        }

        let payload = json!({
            "model": self.model,
            "temperature": self.temperature,
            "max_tokens": self.max_tokens,
            "messages": [
                {
                    "role": "system",
                    "content": SYSTEM_PROMPT,
                },
                {
                    "role": "user",
                    "content": build_backstory_task(&request),
                }
            ]
        });

        let mut http_request = self
            .client
            .post(self.chat_completions_url())
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .json(&payload);

        if let Some(api_key) = &self.api_key {
            http_request = http_request.bearer_auth(api_key);
        }

        let response =
            http_request
                .send()
                .await
                .map_err(|error| BackstoryGeneratorError::Unavailable {
                    details: error.to_string(),
                })?;

        let status = response.status();
        if !status.is_success() {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| String::from("<failed to read response body>"));

            let error = match status {
                StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY => {
                    BackstoryGeneratorError::InvalidQuery { details: body }
                }
                StatusCode::UNAUTHORIZED
                | StatusCode::FORBIDDEN
                | StatusCode::TOO_MANY_REQUESTS => {
                    BackstoryGeneratorError::Unavailable { details: body }
                }
                _ => BackstoryGeneratorError::InvalidResponse { details: body },
            };

            return Err(error);
        }

        let body: ChatCompletionResponse =
            response
                .json()
                .await
                .map_err(|error| BackstoryGeneratorError::InvalidResponse {
                    details: error.to_string(),
                })?;
        let backstory = body
            .choices
            .into_iter()
            .next()
            .map(|choice| choice.message.content.into_text())
            .unwrap_or_default()
            .trim()
            .to_owned();

        if backstory.is_empty() {
            return Err(BackstoryGeneratorError::InvalidResponse {
                details: String::from("model returned an empty completion"),
            });
        }

        Ok(BackstoryGenerationResponse { backstory })
    }
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Debug, Deserialize)]
struct ChatMessage {
    content: ChatMessageContent,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ChatMessageContent {
    Text(String),
    Parts(Vec<ChatMessagePart>),
}

impl ChatMessageContent {
    fn into_text(self) -> String {
        match self {
            Self::Text(value) => value,
            Self::Parts(parts) => parts
                .into_iter()
                .filter_map(|part| match part {
                    ChatMessagePart::Text { text } => Some(text),
                    ChatMessagePart::Refusal { refusal } => Some(refusal),
                    ChatMessagePart::Unknown => None,
                })
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum ChatMessagePart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "refusal")]
    Refusal { refusal: String },
    #[serde(other)]
    Unknown,
}
