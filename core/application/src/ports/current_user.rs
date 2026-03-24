use domain::value_objects::UserId;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CurrentUserError {
    #[error("authentication required")]
    Unauthenticated,
    #[error("access denied")]
    Forbidden,
    #[error("authentication backend unavailable")]
    Unavailable,
}

pub type CurrentUserResult<T> = Result<T, CurrentUserError>;

#[async_trait::async_trait]
pub trait CurrentUserPort: Send + Sync {
    async fn current_user_id(&self) -> CurrentUserResult<UserId>;
}
