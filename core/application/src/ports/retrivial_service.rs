use domain::aggregates::{ContextObject, GameSession};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct RetrivialCandidate {
    pub context_object: ContextObject,
    pub semantic_similarity: f32,
}

#[derive(Debug, Clone)]
pub struct RetrivialObject {
    pub context_object: ContextObject,
    pub semantic_similarity: f32,
    pub combined_score: f32,
}

#[derive(Debug, Error)]
pub enum RetrivialServiceError {
    #[error("RetrivialService unavailable")]
    Unavailable,
    #[error("RetrivialService returned invalid data: {details}")]
    Internal { details: String },
}

pub type RetrivialServiceResult<T> = Result<T, RetrivialServiceError>;

#[async_trait::async_trait]
pub trait RetrivialService: Send + Sync {
    async fn rerank(
        &self,
        session: &GameSession,
        candidates: Vec<RetrivialCandidate>,
        now: chrono::DateTime<chrono::Utc>,
    ) -> RetrivialServiceResult<Vec<RetrivialObject>>;
}
