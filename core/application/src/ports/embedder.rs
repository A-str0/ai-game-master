use thiserror::Error;

pub type Vector = Vec<f32>; // TODO: move?? where??

#[derive(Debug, Clone)]
pub struct EmbedderQuery {
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct EmbedderResponse {
    pub vector: Vector,
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
