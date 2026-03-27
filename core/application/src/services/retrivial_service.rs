use domain::aggregates::{ContextObject, GameSession, Message};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct RetrivialObject {
    pub context_object: ContextObject,
    pub semantic_similarity: f32,
    pub combined_score: f32,
}

#[derive(Debug, Error)]
pub enum RetrivialServiceError {
    #[error("retrivial service unavailable: {details}")]
    Unavailable { details: String },
    #[error("retrivial service failed internally: {details}")]
    Internal { details: String },
}

pub type RetrivialServiceResult<T> = Result<T, RetrivialServiceError>;

#[async_trait::async_trait]
pub trait RetrivialService: Send + Sync {
    async fn find_for_message(
        &self,
        session: &GameSession,
        player_message: &Message,
    ) -> RetrivialServiceResult<Vec<RetrivialObject>>;
}
