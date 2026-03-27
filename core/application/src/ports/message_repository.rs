use domain::{aggregates::Message, value_objects::GameSessionId};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MessageRepositoryError {
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
    #[error("message storage unavailable: {details}")]
    Unavailable { details: String },
    #[error("message storage returned invalid data: {details}")]
    Internal { details: String },
}

impl MessageRepositoryError {
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
