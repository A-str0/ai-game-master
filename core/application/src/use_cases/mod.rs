mod create_session;
mod get_session;
mod send_message;

use crate::{
    ports::{
        ContextObjectRepositoryError, EmbedderError, GameSessionRepositoryError,
        MessageRepositoryError, UserPortError, VectorSearcherError,
    },
    services::{AgentOrchestrationServiceError, PromptAssemblerError, RetrivialServiceError},
};
use domain::DomainError;
use thiserror::Error;

pub use create_session::*;
pub use get_session::*;
pub use send_message::*;

#[derive(Debug, Error)]
pub enum UseCaseError {
    #[error(transparent)]
    Domain(#[from] DomainError),
    #[error(transparent)]
    User(#[from] UserPortError),
    #[error(transparent)]
    GameSessionRepository(#[from] GameSessionRepositoryError),
    #[error(transparent)]
    MessageRepository(#[from] MessageRepositoryError),
    #[error(transparent)]
    ContextObjectRepository(#[from] ContextObjectRepositoryError),
    #[error(transparent)]
    AgentOrchestration(#[from] AgentOrchestrationServiceError),
    #[error(transparent)]
    Embedder(#[from] EmbedderError),
    #[error(transparent)]
    PromptAssembler(#[from] PromptAssemblerError),
    #[error(transparent)]
    Retrivial(#[from] RetrivialServiceError),
    #[error(transparent)]
    VectorSearcher(#[from] VectorSearcherError),
}

pub type UseCaseResult<T> = Result<T, UseCaseError>;

#[async_trait::async_trait]
pub trait UseCase<C, O> {
    async fn execute(&self, command: C) -> UseCaseResult<O>;
}
