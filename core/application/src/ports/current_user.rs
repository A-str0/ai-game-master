use domain::value_objects::UserId;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CurrentUserError {
    #[error("authentication required: {details}")]
    Unauthenticated { details: String },
    #[error("access denied: {details}")]
    Forbidden { details: String },
    #[error("authentication backend unavailable: {details}")]
    Unavailable { details: String },
}

pub type CurrentUserResult<T> = Result<T, CurrentUserError>;

#[async_trait::async_trait]
pub trait CurrentUser: Send + Sync {
    async fn current_user_id(&self) -> CurrentUserResult<UserId>;
}
