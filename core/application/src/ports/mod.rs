mod agent_orchestrator;
mod clock;
mod context_object_repository;
mod current_user;
mod game_session_repository;
mod id_generator;
mod message_repository;

use thiserror::Error;

pub use agent_orchestrator::*;
pub use clock::*;
pub use context_object_repository::*;
pub use current_user::*;
pub use game_session_repository::*;
pub use id_generator::*;
pub use message_repository::*;

#[derive(Debug, Error)]
pub enum RepoError {
    #[error("{resource} not found")]
    NotFound { resource: &'static str },
    #[error("{resource} already exists")]
    Conflict { resource: &'static str },
    #[error("storage unavailable")]
    Unavailable,
}

impl RepoError {
    pub const fn not_found(resource: &'static str) -> Self {
        Self::NotFound { resource }
    }

    pub const fn conflict(resource: &'static str) -> Self {
        Self::Conflict { resource }
    }
}

pub type RepoResult<T> = Result<T, RepoError>;
