use domain::{aggregates::GameSession, value_objects::GameSessionId};
use thiserror::Error;

/// Errors returned by [`GameSessionRepository`].
#[derive(Debug, Error)]
pub enum GameSessionRepositoryError {
    /// Requested session does not exist.
    #[error("GameSession not found: {details}")]
    NotFound {
        /// Repository-specific error details.
        details: String,
    },
    /// Insert or update would violate repository constraints.
    #[error("GameSession conflict: {details}")]
    Conflict {
        /// Repository-specific error details.
        details: String,
    },
    /// Backing storage is temporarily unavailable.
    #[error("GameSessionRepository unavailable: {details}")]
    Unavailable {
        /// Repository-specific error details.
        details: String,
    },
    /// Repository returned data that cannot be mapped into the domain model.
    #[error("GameSessionRepository returned invalid data: {details}")]
    Internal {
        /// Repository-specific error details.
        details: String,
    },
}

/// Convenient result alias returned by [`GameSessionRepository`].
pub type GameSessionRepositoryResult<T> = Result<T, GameSessionRepositoryError>;

/// Persistence port for game sessions.
#[async_trait::async_trait]
pub trait GameSessionRepository: Send + Sync {
    /// Persists a newly created session.
    async fn insert(&self, session: &GameSession) -> GameSessionRepositoryResult<()>;
    /// Loads a session by identifier.
    async fn get_by_id(&self, id: &GameSessionId) -> GameSessionRepositoryResult<GameSession>;
    /// Persists changes to an existing session.
    async fn update(&self, session: &GameSession) -> GameSessionRepositoryResult<()>;
}
