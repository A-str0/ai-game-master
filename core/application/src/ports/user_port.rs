use domain::value_objects::UserId;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UserPortError {
    #[error("UserPort authentication required")]
    Unauthenticated,
    #[error("UserPort access denied")]
    Forbidden,
    #[error("UserPort authentication backend unavailable")]
    Unavailable,
}

pub type UserPortResult<T> = Result<T, UserPortError>;

#[async_trait::async_trait]
pub trait UserPort: Send + Sync {
    async fn current_user_id(&self) -> UserPortResult<UserId>;
}
