use thiserror::Error;

pub mod ports;
pub mod use_cases;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("owner not found")]
    OwnerNotFound,
    #[error("domain validation failed: {0}")]
    Domain(#[from] domain::DomainError),
    #[error("repository error: {0}")]
    Repository(#[from] ports::RepoError),
}

#[tokio::main]
async fn main() {}
