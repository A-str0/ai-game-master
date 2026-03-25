use thiserror::Error;

pub mod ports;
pub mod services;
pub mod use_cases;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{resource} not found")]
    NotFound { resource: &'static str },
    #[error("authentication required")]
    Unauthenticated,
    #[error("access denied")]
    Forbidden,
    #[error("{resource} conflict")]
    Conflict { resource: &'static str },
    #[error("dependency unavailable")]
    Unavailable,
    #[error(transparent)]
    Domain(#[from] domain::DomainError),
}

impl From<ports::CurrentUserError> for AppError {
    fn from(value: ports::CurrentUserError) -> Self {
        match value {
            ports::CurrentUserError::Unauthenticated => Self::Unauthenticated,
            ports::CurrentUserError::Forbidden => Self::Forbidden,
            ports::CurrentUserError::Unavailable => Self::Unavailable,
        }
    }
}

impl From<ports::RepoError> for AppError {
    fn from(value: ports::RepoError) -> Self {
        match value {
            ports::RepoError::NotFound { resource } => Self::NotFound { resource },
            ports::RepoError::Conflict { resource } => Self::Conflict { resource },
            ports::RepoError::Unavailable => Self::Unavailable,
        }
    }
}

impl From<ports::AgentError> for AppError {
    fn from(_: ports::AgentError) -> Self {
        Self::Unavailable
    }
}

pub type AppResult<T> = Result<T, AppError>;
