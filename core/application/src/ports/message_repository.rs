use domain::{aggregates::Message, value_objects::GameSessionId};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MessageRepositoryError {
    #[error("Message not found: {details}")]
    NotFound { details: String },
    #[error("Message conflict: {details}")]
    Conflict { details: String },
    #[error("MessageRepository unavailable: {details}")]
    Unavailable { details: String },
    #[error("MessageRepository returned invalid data: {details}")]
    Internal { details: String },
}

pub type MessageRepositoryResult<T> = Result<T, MessageRepositoryError>;

#[async_trait::async_trait]
pub trait MessageRepository: Send + Sync {
    async fn create(&self, message: &Message) -> MessageRepositoryResult<()>;
    async fn list_recent(
        &self,
        session_id: &GameSessionId,
        limit: usize,
    ) -> MessageRepositoryResult<Vec<Message>>;
}
