use domain::{
    aggregates::ContextObject,
    value_objects::{ContextObjectId, GameSessionId},
};
use thiserror::Error;

/// Errors returned by [`ContextObjectRepository`].
#[derive(Debug, Error)]
pub enum ContextObjectRepositoryError {
    /// Requested object does not exist.
    #[error("ContextObject not found: {details}")]
    NotFound {
        /// Repository-specific error details.
        details: String,
    },
    /// Insert or update would violate repository constraints.
    #[error("ContextObject conflict: {details}")]
    Conflict {
        /// Repository-specific error details.
        details: String,
    },
    /// Backing storage is temporarily unavailable.
    #[error("ContextObjectRepository unavailable: {details}")]
    Unavailable {
        /// Repository-specific error details.
        details: String,
    },
    /// Repository returned data that cannot be mapped into the domain model.
    #[error("ContextObjectRepository returned invalid data: {details}")]
    Internal {
        /// Repository-specific error details.
        details: String,
    },
}

/// Convenient result alias returned by [`ContextObjectRepository`].
pub type ContextObjectRepositoryResult<T> = Result<T, ContextObjectRepositoryError>;

/// Persistence port for durable session context objects.
#[async_trait::async_trait]
pub trait ContextObjectRepository: Send + Sync {
    /// Persists a newly created context object.
    async fn insert(&self, context_object: &ContextObject) -> ContextObjectRepositoryResult<()>;
    /// Loads a context object by session and identifier.
    async fn get_by_id(
        &self,
        session_id: &GameSessionId,
        id: &ContextObjectId,
    ) -> ContextObjectRepositoryResult<ContextObject>;
    /// Persists a modified context object.
    async fn update(&self, context_object: &ContextObject) -> ContextObjectRepositoryResult<()>;
}
