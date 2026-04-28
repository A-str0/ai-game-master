use domain::{
    aggregates::Message,
    value_objects::{GameSessionId, MessageId},
};
use thiserror::Error;

/// Errors returned by [`MessageRepository`].
#[derive(Debug, Error)]
pub enum MessageRepositoryError {
    /// Requested message does not exist.
    #[error("Message not found: {details}")]
    NotFound {
        /// Repository-specific error details.
        details: String,
    },
    /// Insert or update would violate repository constraints.
    #[error("Message conflict: {details}")]
    Conflict {
        /// Repository-specific error details.
        details: String,
    },
    /// Backing storage is temporarily unavailable.
    #[error("MessageRepository unavailable: {details}")]
    Unavailable {
        /// Repository-specific error details.
        details: String,
    },
    /// Repository returned data that cannot be mapped into the domain model.
    #[error("MessageRepository returned invalid data: {details}")]
    Internal {
        /// Repository-specific error details.
        details: String,
    },
}

/// Convenient result alias returned by [`MessageRepository`].
pub type MessageRepositoryResult<T> = Result<T, MessageRepositoryError>;

/// Persistence port for session transcript messages.
#[async_trait::async_trait]
pub trait MessageRepository: Send + Sync {
    /// Persists a newly created message.
    async fn insert(&self, message: &Message) -> MessageRepositoryResult<()>;
    /// Loads one message by identifier.
    async fn get_by_id(&self, id: &MessageId) -> MessageRepositoryResult<Message>;
    /// Returns messages for a session in reverse chronological order.
    async fn list_by_session(
        &self,
        session_id: &GameSessionId,
        limit: usize,
    ) -> MessageRepositoryResult<Vec<Message>>;
    /// Returns the most recent messages for a session in reverse chronological order.
    async fn list_recent(
        &self,
        session_id: &GameSessionId,
        limit: usize,
    ) -> MessageRepositoryResult<Vec<Message>>;
}
