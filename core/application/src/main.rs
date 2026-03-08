use thiserror::Error;
use uuid::Uuid;

pub mod ports;
pub mod service;
pub mod use_cases;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("resource not found")]
    NotFound(Uuid),
    #[error("access denied")]
    Forbidden,
    #[error("conflict")]
    Conflict,
    #[error("dependency unavailable")]
    Unavailable,
    #[error("domain error: {0}")]
    Domain(#[from] domain::DomainError),
}

pub type AppResult<T> = Result<T, AppError>;

#[tokio::main]
async fn main() {}
