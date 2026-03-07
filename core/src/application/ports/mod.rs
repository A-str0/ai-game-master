mod game_session_repository;

use crate::application::AppError;

pub use game_session_repository::GameSessionRepository;

pub type RepoResult<T> = Result<T, AppError>;
