mod game_session_repository;
mod user_access;

use thiserror::Error;

pub use game_session_repository::GameSessionRepository;
pub use user_access::{PortError, PortResult, UserAccessPort};

#[derive(Debug, Error)]
pub enum RepoError {
    #[error("entity not found")]
    NotFound,
    #[error("entity already exists")]
    Conflict,
    #[error("storage unavailable")]
    Unavailable,
}

pub type RepoResult<T> = Result<T, RepoError>;
