mod agent;
mod clock;
mod current_user;
mod game_session_repository;
mod id_generator;
mod message_repository;

use thiserror::Error;

pub use agent::*;
pub use clock::*;
pub use current_user::*;
pub use game_session_repository::*;
pub use id_generator::*;
pub use message_repository::*;

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
