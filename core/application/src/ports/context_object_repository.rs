use domain::{
    aggregates::ContextObject,
    value_objects::{ContextObjectId, GameSessionId},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContextObjectRepositoryError {
    #[error("ContextObject not found: {details}")]
    NotFound { details: String },
    #[error("ContextObject conflict: {details}")]
    Conflict { details: String },
    #[error("ContextObjectRepository unavailable: {details}")]
    Unavailable { details: String },
    #[error("ContextObjectRepository returned invalid data: {details}")]
    Internal { details: String },
}

pub type ContextObjectRepositoryResult<T> = Result<T, ContextObjectRepositoryError>;

#[async_trait::async_trait]
pub trait ContextObjectRepository: Send + Sync {
    async fn insert(&self, context_object: &ContextObject) -> ContextObjectRepositoryResult<()>;
    async fn get_by_id(
        &self,
        session_id: &GameSessionId,
        id: &ContextObjectId,
    ) -> ContextObjectRepositoryResult<ContextObject>;
    async fn update(&self, context_object: &ContextObject) -> ContextObjectRepositoryResult<()>;
}
