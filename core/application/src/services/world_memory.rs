use domain::{
    aggregates::{ContextObject, GameSession},
    value_objects::GameSessionId,
};
use thiserror::Error;

use crate::ports::ProposedContextObject;

#[derive(Debug, Error)]
pub enum WorldMemoryError {
    #[error("{resource} conflict: {details}")]
    Conflict {
        resource: &'static str,
        details: String,
    },
    #[error("world memory unavailable: {details}")]
    Unavailable { details: String },
    #[error("world memory internal failure: {details}")]
    Internal { details: String },
    #[error(transparent)]
    Domain(#[from] domain::DomainError),
}

pub type WorldMemoryResult<T> = Result<T, WorldMemoryError>;

#[async_trait::async_trait]
pub trait WorldMemoryManager: Send + Sync {
    async fn initialize_session(&self, session_id: GameSessionId) -> WorldMemoryResult<()>;

    async fn create_context_object(
        &self,
        session: &GameSession,
        object: ProposedContextObject,
    ) -> WorldMemoryResult<ContextObject>;
}
