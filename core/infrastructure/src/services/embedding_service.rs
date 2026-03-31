use application::ports::{
    Embedder, EmbedderError, EmbedderQuery, EmbedderResponse, EmbedderResult,
};
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use serde_json::json;

const OPENROUTER_EMBEDDINGS_URL: &str = "https://openrouter.ai/api/v1/embeddings";

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

pub struct OpenRouterEmbeddingService {
    client: Client,
    api_key: String,
    model: String,
}

impl OpenRouterEmbeddingService {
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
impl Embedder for OpenRouterEmbeddingService {
    // TODO: handle invalid query
    async fn create_embedding(&self, query: EmbedderQuery) -> EmbedderResult<EmbedderResponse> {
        let response = self
            .client
            .post(OPENROUTER_EMBEDDINGS_URL)
            .bearer_auth(&self.api_key)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .json(&json!({
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
