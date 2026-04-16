//! Primary application use cases.
//!
//! These traits and types define the commands executed by the API and any other
//! driving adapters.

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

/// Unified error surface for application use cases.
#[derive(Debug, Error)]
pub enum UseCaseError {
    /// Domain validation failed.
    #[error(transparent)]
    Domain(#[from] DomainError),
    /// User lookup or authorization failed.
    #[error(transparent)]
    User(#[from] UserPortError),
    /// Session persistence failed.
    #[error(transparent)]
    GameSessionRepository(#[from] GameSessionRepositoryError),
    /// Message persistence failed.
    #[error(transparent)]
    MessageRepository(#[from] MessageRepositoryError),
    /// Context object persistence failed.
    #[error(transparent)]
    ContextObjectRepository(#[from] ContextObjectRepositoryError),
    /// Agent orchestration failed.
    #[error(transparent)]
    AgentOrchestration(#[from] AgentOrchestrationServiceError),
    /// Embedding generation failed.
    #[error(transparent)]
    Embedder(#[from] EmbedderError),
    /// Prompt assembly failed.
    #[error(transparent)]
    PromptAssembler(#[from] PromptAssemblerError),
    /// Retrieval reranking failed.
    #[error(transparent)]
    Retrivial(#[from] RetrivialServiceError),
    /// Vector search failed.
    #[error(transparent)]
    VectorSearcher(#[from] VectorSearcherError),
}

/// Convenient result alias returned by use cases.
pub type UseCaseResult<T> = Result<T, UseCaseError>;

/// Generic async use case contract.
#[async_trait::async_trait]
pub trait UseCase<C, O> {
    /// Executes the use case for the supplied command object.
    async fn execute(&self, command: C) -> UseCaseResult<O>;
}
