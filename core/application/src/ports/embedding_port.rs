use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct EmbedderQuery {
    pub text: String,
}

#[derive(Debug, Clone, Deserialize)]
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
