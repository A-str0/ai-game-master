use thiserror::Error;

pub mod ports;
pub mod services;
pub mod use_cases;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{resource} not found: {details}")]
    NotFound {
        resource: &'static str,
        details: String,
    },
    #[error("authentication required: {details}")]
    Unauthenticated { details: String },
    #[error("access denied: {details}")]
    Forbidden { details: String },
    #[error("{resource} conflict: {details}")]
    Conflict {
        resource: &'static str,
        details: String,
    },
    #[error("{dependency} dependency unavailable: {details}")]
    Unavailable {
        dependency: &'static str,
        details: String,
    },
    #[error("{component} internal failure: {details}")]
    Internal {
        component: &'static str,
        details: String,
    },
    #[error(transparent)]
    Domain(#[from] domain::DomainError),
}

impl From<ports::CurrentUserError> for AppError {
    fn from(value: ports::CurrentUserError) -> Self {
        match value {
            ports::CurrentUserError::Unauthenticated { details } => {
                Self::Unauthenticated { details }
            }
            ports::CurrentUserError::Forbidden { details } => Self::Forbidden { details },
            ports::CurrentUserError::Unavailable { details } => Self::Unavailable {
                dependency: "UserPort",
                details,
            },
        }
    }
}

impl From<ports::GameSessionRepositoryError> for AppError {
    fn from(value: ports::GameSessionRepositoryError) -> Self {
        match value {
            ports::GameSessionRepositoryError::NotFound { resource, details } => {
                Self::NotFound { resource, details }
            }
            ports::GameSessionRepositoryError::Conflict { resource, details } => {
                Self::Conflict { resource, details }
            }
            ports::GameSessionRepositoryError::Unavailable { details } => Self::Unavailable {
                dependency: "GameSessionRepository",
                details,
            },
            ports::GameSessionRepositoryError::Internal { details } => Self::Internal {
                component: "GameSessionRepository",
                details,
            },
        }
    }
}

impl From<ports::MessageRepositoryError> for AppError {
    fn from(value: ports::MessageRepositoryError) -> Self {
        match value {
            ports::MessageRepositoryError::NotFound { resource, details } => {
                Self::NotFound { resource, details }
            }
            ports::MessageRepositoryError::Conflict { resource, details } => {
                Self::Conflict { resource, details }
            }
            ports::MessageRepositoryError::Unavailable { details } => Self::Unavailable {
                dependency: "MessageRepository",
                details,
            },
            ports::MessageRepositoryError::Internal { details } => Self::Internal {
                component: "MessageRepository",
                details,
            },
        }
    }
}

impl From<ports::ContextObjectRepositoryError> for AppError {
    fn from(value: ports::ContextObjectRepositoryError) -> Self {
        match value {
            ports::ContextObjectRepositoryError::NotFound { resource, details } => {
                Self::NotFound { resource, details }
            }
            ports::ContextObjectRepositoryError::Conflict { resource, details } => {
                Self::Conflict { resource, details }
            }
            ports::ContextObjectRepositoryError::Unavailable { details } => Self::Unavailable {
                dependency: "ContextObjectRepository",
                details,
            },
            ports::ContextObjectRepositoryError::Internal { details } => Self::Internal {
                component: "ContextObjectRepository",
                details,
            },
        }
    }
}

impl From<ports::AgentOrchestratorError> for AppError {
    fn from(value: ports::AgentOrchestratorError) -> Self {
        match value {
            ports::AgentOrchestratorError::Unavailable { details } => Self::Unavailable {
                dependency: "AgentOrchestrator",
                details,
            },
            ports::AgentOrchestratorError::InvalidResponse { details } => Self::Internal {
                component: "AgentOrchestrator",
                details,
            },
        }
    }
}

impl From<services::EmbedderError> for AppError {
    fn from(value: services::EmbedderError) -> Self {
        match value {
            services::EmbedderError::Unavailable { details } => Self::Unavailable {
                dependency: "Embedder",
                details,
            },
            services::EmbedderError::InvalidResponse { details } => Self::Internal {
                component: "Embedder",
                details,
            },
        }
    }
}

impl From<services::VectorSearcherError> for AppError {
    fn from(value: services::VectorSearcherError) -> Self {
        match value {
            services::VectorSearcherError::Unavailable { details } => Self::Unavailable {
                dependency: "VectorSearcher",
                details,
            },
            services::VectorSearcherError::InvalidResponse { details } => Self::Internal {
                component: "VectorSearcher",
                details,
            },
        }
    }
}

impl From<services::PromptAssemblerError> for AppError {
    fn from(value: services::PromptAssemblerError) -> Self {
        match value {
            services::PromptAssemblerError::Unavailable { details } => Self::Unavailable {
                dependency: "PromptAssembler",
                details,
            },
            services::PromptAssemblerError::InvalidInput { details } => Self::Internal {
                component: "PromptAssembler",
                details,
            },
        }
    }
}

impl From<services::RetrivialServiceError> for AppError {
    fn from(value: services::RetrivialServiceError) -> Self {
        match value {
            services::RetrivialServiceError::Unavailable { details } => Self::Unavailable {
                dependency: "RetrivialService",
                details,
            },
            services::RetrivialServiceError::Internal { details } => Self::Internal {
                component: "RetrivialService",
                details,
            },
        }
    }
}

impl From<services::WorldMemoryError> for AppError {
    fn from(value: services::WorldMemoryError) -> Self {
        match value {
            services::WorldMemoryError::Conflict { resource, details } => {
                Self::Conflict { resource, details }
            }
            services::WorldMemoryError::Unavailable { details } => Self::Unavailable {
                dependency: "WorldMemoryManager",
                details,
            },
            services::WorldMemoryError::Internal { details } => Self::Internal {
                component: "WorldMemoryManager",
                details,
            },
            services::WorldMemoryError::Domain(error) => Self::Domain(error),
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;
