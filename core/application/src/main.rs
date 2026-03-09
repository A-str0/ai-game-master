use thiserror::Error;

pub mod ports;
pub mod services;
pub mod use_cases;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("resource not found")]
    NotFound(uuid::Uuid),
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

fn main() {}
