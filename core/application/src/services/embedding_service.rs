use reqwest::{Client, StatusCode};
use serde::Deserialize;
use serde_json::json;
use thiserror::Error;

// TODO: move to config
const OPENROUTER_EMBEDDINGS_URL: &str = "https://openrouter.ai/api/v1/embeddings";

#[derive(Debug, Clone)]
pub struct EmbedderQuery {
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct EmbedderResponse {
    pub vector: Vec<f32>,
}

#[derive(Debug, Error)]
pub enum EmbedderError {
    #[error("Embedder backend unavailable")]
    Unavailable,
    #[error("Embedder backend received invalid query")]
    InvalidQuery,
    #[error("Embedder backend returned invalid output")]
    InvalidResponse,
}

pub type EmbedderResult<T> = Result<T, EmbedderError>;

#[async_trait::async_trait]
pub trait Embedder: Send + Sync {
    async fn create_embedding(&self, query: EmbedderQuery) -> EmbedderResult<EmbedderResponse>;
}

trait ReqwestResultExt<T> {
    fn into_embedder(self) -> EmbedderResult<T>;
}

impl<T> ReqwestResultExt<T> for Result<T, reqwest::Error> {
    fn into_embedder(self) -> EmbedderResult<T> {
        self.map_err(|_| EmbedderError::Unavailable)
    }
}

#[derive(Debug, Deserialize)]
struct EmbeddingsResponse {
    data: Vec<EmbeddingItem>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingItem {
    embedding: Vec<f32>,
}

pub struct EmbeddingService {
    client: Client,
    api_key: String,
    model: String,
}

impl EmbeddingService {
    pub async fn new() -> Result<Self, reqwest::Error> {
        let client = Client::builder().build()?;
        let api_key = std::env::var("OPENROUTER_API_KEY")
            .or_else(|_| std::env::var("LLM_API_KEY"))
            .unwrap_or_else(|_| String::from("API_KEY"));
        let model = std::env::var("EMBEDDING_MODEL")
            .unwrap_or_else(|_| String::from("nvidia/llama-nemotron-embed-vl-1b-v2:free"));

        Ok(Self {
            client,
            api_key,
            model,
        })
    }
}

#[async_trait::async_trait]
impl Embedder for EmbeddingService {
    // TODO: handle invalid query
    async fn create_embedding(&self, query: EmbedderQuery) -> EmbedderResult<EmbedderResponse> {
        let response = self
            .client
            .post(OPENROUTER_EMBEDDINGS_URL)
            .bearer_auth(&self.api_key)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .json(&json!({ // TODO: move to variable
                "model": self.model,
                "input": query.text,
                "encoding_format": "float",
            }))
            .send()
            .await
            .into_embedder()?;

        let status = response.status();
        if !status.is_success() {
            let _ = response // Body
                .text()
                .await
                .unwrap_or_else(|_| String::from("<failed to read response body>"));

            let error = match status {
                StatusCode::UNAUTHORIZED | StatusCode::TOO_MANY_REQUESTS => {
                    EmbedderError::Unavailable
                }
                _ => EmbedderError::InvalidResponse,
            };

            return Err(error);
        }

        let body: EmbeddingsResponse = response.json().await.into_embedder()?;

        let vector = body
            .data
            .into_iter()
            .next()
            .ok_or(EmbedderError::InvalidResponse)?
            .embedding;

        if vector.is_empty() {
            return Err(EmbedderError::InvalidResponse);
        }

        Ok(EmbedderResponse { vector })
    }
}
