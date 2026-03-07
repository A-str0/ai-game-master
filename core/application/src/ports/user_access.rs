use domain::aggregates::User;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PortError {
    #[error("user not found")]
    NotFound,
    #[error("access denied")]
    AccessDenied,
    #[error("access backend unavailable")]
    Unavailable,
}

pub type PortResult<T> = Result<T, PortError>;

#[async_trait::async_trait]
pub trait UserAccessPort: Send + Sync {
    async fn get_user(&self) -> PortResult<User>;
}
