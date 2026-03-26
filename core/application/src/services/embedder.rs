use crate::AppResult;

pub type Vector = Vec<f32>; // TODO: move?? where??

#[derive(Debug, Clone)]
pub struct EmbedderQuery {
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct EmbedderResponse {
    pub vector: Vector,
}

#[async_trait::async_trait]
pub trait Embedder: Send + Sync {
    async fn create_embedding(&self, query: EmbedderQuery) -> AppResult<EmbedderResponse>;
}
