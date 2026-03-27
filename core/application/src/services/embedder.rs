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
    #[error("embedding backend unavailable: {details}")]
    Unavailable { details: String },
    #[error("embedding backend returned invalid output: {details}")]
    InvalidResponse { details: String },
}

pub type EmbedderResult<T> = Result<T, EmbedderError>;

#[async_trait::async_trait]
pub trait Embedder: Send + Sync {
    async fn create_embedding(&self, query: EmbedderQuery) -> EmbedderResult<EmbedderResponse>;
}
