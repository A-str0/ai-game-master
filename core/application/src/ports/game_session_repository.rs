use domain::{aggregates::GameSession, value_objects::GameSessionId};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GameSessionRepositoryError {
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
    #[error("game session storage unavailable: {details}")]
    Unavailable { details: String },
    #[error("game session storage returned invalid data: {details}")]
    Internal { details: String },
}

impl GameSessionRepositoryError {
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

pub type GameSessionRepositoryResult<T> = Result<T, GameSessionRepositoryError>;

#[async_trait::async_trait]
pub trait GameSessionRepository: Send + Sync {
    async fn create(&self, session: &GameSession) -> GameSessionRepositoryResult<()>;
    async fn get_by_id(&self, id: &GameSessionId) -> GameSessionRepositoryResult<GameSession>;
    async fn update(&self, session: &GameSession) -> GameSessionRepositoryResult<()>;
}
