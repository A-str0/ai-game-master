use domain::value_objects::UserId;
use thiserror::Error;

/// Errors returned by [`UserPort`].
#[derive(Debug, Error)]
pub enum UserPortError {
    /// No authenticated user is available in the current request context.
    #[error("UserPort authentication required")]
    Unauthenticated,
    /// Current user is authenticated but not authorized for the action.
    #[error("UserPort access denied")]
    Forbidden,
    /// Authentication backend or request context is unavailable.
    #[error("UserPort authentication backend unavailable")]
    Unavailable,
}

/// Convenient result alias returned by [`UserPort`].
pub type UserPortResult<T> = Result<T, UserPortError>;

/// Port that resolves the current user identity.
#[async_trait::async_trait]
pub trait UserPort: Send + Sync {
    /// Returns the currently authenticated user identifier.
    async fn current_user_id(&self) -> UserPortResult<UserId>;
}
