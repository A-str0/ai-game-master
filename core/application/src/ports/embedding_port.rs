use serde::Deserialize;
use thiserror::Error;

/// Request sent to the embedding backend.
#[derive(Debug, Clone)]
pub struct EmbedderQuery {
    /// Input text that should be embedded.
    pub text: String,
}

/// Embedding vector returned by the backend.
#[derive(Debug, Clone, Deserialize)]
pub struct EmbedderResponse {
    /// Dense embedding values in backend-defined order.
    pub vector: Vec<f32>,
}

/// Errors returned by [`Embedder`].
#[derive(Debug, Error)]
pub enum EmbedderError {
    /// Backend could not be reached or is throttling requests.
    #[error("Embedder backend unavailable")]
    Unavailable,
    /// Request payload is not accepted by the backend.
    #[error("Embedder backend received invalid query")]
    InvalidQuery,
    /// Backend responded with malformed or unusable data.
    #[error("Embedder backend returned invalid output")]
    InvalidResponse,
}

/// Convenient result alias returned by [`Embedder`].
pub type EmbedderResult<T> = Result<T, EmbedderError>;

/// Port that produces embeddings for free-form text.
#[async_trait::async_trait]
pub trait Embedder: Send + Sync {
    /// Creates an embedding for the supplied text.
    async fn create_embedding(&self, query: EmbedderQuery) -> EmbedderResult<EmbedderResponse>;
}
