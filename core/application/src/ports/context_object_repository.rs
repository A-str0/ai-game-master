use domain::{
    aggregates::ContextObject,
    value_objects::{ContextObjectId, GameSessionId},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContextObjectRepositoryError {
    #[error("{resource} not found: {details}")]
    NotFound {
        resource: &'static str,
        details: String,
    },
    #[error("{resource} already exists: {details}")]
    Conflict {
        resource: &'static str,
        details: String,
    },
    #[error("context object storage unavailable: {details}")]
    Unavailable { details: String },
    #[error("context object storage returned invalid data: {details}")]
    Internal { details: String },
}

impl ContextObjectRepositoryError {
    pub fn not_found(resource: &'static str, details: impl Into<String>) -> Self {
        Self::NotFound {
            resource,
            details: details.into(),
        }
    }

    pub fn conflict(resource: &'static str, details: impl Into<String>) -> Self {
        Self::Conflict {
            resource,
            details: details.into(),
        }
    }
}

pub type ContextObjectRepositoryResult<T> = Result<T, ContextObjectRepositoryError>;

#[async_trait::async_trait]
pub trait ContextObjectRepository: Send + Sync {
    async fn create(&self, context_object: &ContextObject) -> ContextObjectRepositoryResult<()>;
    async fn get_by_id(
        &self,
        session_id: &GameSessionId,
        id: &ContextObjectId,
    ) -> ContextObjectRepositoryResult<ContextObject>;
    async fn update(&self, context_object: &ContextObject) -> ContextObjectRepositoryResult<()>;
}
