use domain::{aggregates::GameSession, value_objects::GameSessionId};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GameSessionRepositoryError {
    #[error("GameSession not found: {details}")]
    NotFound { details: String },
    #[error("GameSession conflict: {details}")]
    Conflict { details: String },
    #[error("GameSessionRepository unavailable: {details}")]
    Unavailable { details: String },
    #[error("GameSessionRepository returned invalid data: {details}")]
    Internal { details: String },
}

pub type GameSessionRepositoryResult<T> = Result<T, GameSessionRepositoryError>;

#[async_trait::async_trait]
pub trait GameSessionRepository: Send + Sync {
    async fn create(&self, session: &GameSession) -> GameSessionRepositoryResult<()>;
    async fn get_by_id(&self, id: &GameSessionId) -> GameSessionRepositoryResult<GameSession>;
    async fn update(&self, session: &GameSession) -> GameSessionRepositoryResult<()>;
}
