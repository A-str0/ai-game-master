use domain::aggregates::{ContextObject, GameSession, Message};
use thiserror::Error;

use crate::ports::{
    ContextObjectRepositoryError, GameSessionRepositoryError, MessageRepositoryError,
};

/// Errors returned while managing a transactional unit of work.
#[derive(Debug, Error)]
pub enum UnitOfWorkError {
    /// Transaction could not be started, committed, or rolled back.
    #[error("UnitOfWork unavailable: {details}")]
    Unavailable {
        /// Infrastructure-specific error details.
        details: String,
    },
    /// Transaction manager returned invalid internal state.
    #[error("UnitOfWork internal error: {details}")]
    Internal {
        /// Infrastructure-specific error details.
        details: String,
    },
}

/// Convenient result alias returned by unit-of-work APIs.
pub type UnitOfWorkResult<T> = Result<T, UnitOfWorkError>;

/// Mutable transactional write context.
#[async_trait::async_trait]
pub trait UnitOfWork: Send {
    /// Persists a newly created session inside the active transaction.
    async fn insert_session(
        &mut self,
        session: &GameSession,
    ) -> Result<(), GameSessionRepositoryError>;
    /// Persists a newly created message inside the active transaction.
    async fn insert_message(&mut self, message: &Message) -> Result<(), MessageRepositoryError>;
    /// Persists a newly created context object inside the active transaction.
    async fn insert_context_object(
        &mut self,
        context_object: &ContextObject,
    ) -> Result<(), ContextObjectRepositoryError>;
    /// Persists a modified session inside the active transaction.
    async fn update_session(
        &mut self,
        session: &GameSession,
    ) -> Result<(), GameSessionRepositoryError>;
    /// Commits the active transaction.
    async fn commit(self: Box<Self>) -> UnitOfWorkResult<()>;
}

/// Factory that opens transactional write contexts.
#[async_trait::async_trait]
pub trait UnitOfWorkFactory: Send + Sync {
    /// Starts a new unit of work.
    async fn begin(&self) -> UnitOfWorkResult<Box<dyn UnitOfWork>>;
}
