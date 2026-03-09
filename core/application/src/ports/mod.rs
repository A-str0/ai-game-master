mod clock;
mod game_session_repository;
mod id_generator;
mod message_repository;
mod user_access;

use thiserror::Error;

pub use clock::*;
pub use game_session_repository::*;
pub use id_generator::*;
pub use message_repository::*;
pub use user_access::*;

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

#[derive(Debug, Error)]
pub enum PortError {
    #[error("user not found")]
    NotFound,
    #[error("access denied")]
    Forbidden,
    #[error("access backend unavailable")]
    Unavailable,
}

pub type PortResult<T> = Result<T, PortError>;
