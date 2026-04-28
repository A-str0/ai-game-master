use application::{
    ports::{
        ContextObjectRepositoryError, EmbedderError, GameSessionRepositoryError,
        MemoryExtractorError, MessageRepositoryError, NarratorError, UnitOfWorkError,
        UserPortError, VectorSearcherError,
    },
    services::{AgentOrchestrationServiceError, PromptAssemblerError, RetrivialServiceError},
    use_cases::UseCaseError,
};
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use super::dto::ErrorResponse;

/// Result alias returned by HTTP handlers.
pub type ApiResult<T> = Result<T, ApiError>;

/// Error type converted into HTTP responses.
#[derive(Debug)]
pub enum ApiError {
    /// Wrapped application-layer error.
    UseCase(UseCaseError),
    /// Client supplied malformed request data.
    BadRequest(String),
}

impl ApiError {
    /// Creates a `400 Bad Request` error with the supplied message.
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest(message.into())
    }

    #[cfg(test)]
    pub fn message(&self) -> &str {
        match self {
            Self::UseCase(_) => "use_case_error",
            Self::BadRequest(message) => message,
        }
    }
}

impl From<UseCaseError> for ApiError {
    fn from(value: UseCaseError) -> Self {
        Self::UseCase(value)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error) = match self {
            ApiError::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
            ApiError::UseCase(error) => (status_code_for_use_case_error(&error), error.to_string()),
        };

        (status, Json(ErrorResponse { error })).into_response()
    }
}

trait HttpStatusCode {
    fn status_code(&self) -> StatusCode;
}

macro_rules! status_code_impl {
    ($type:ty { $($pattern:pat => $status:expr),+ $(,)? }) => {
        impl HttpStatusCode for $type {
            fn status_code(&self) -> StatusCode {
                match self {
                    $($pattern => $status),+
                }
            }
        }
    };
}

status_code_impl!(UserPortError {
    Self::Unauthenticated => StatusCode::UNAUTHORIZED,
    Self::Forbidden => StatusCode::FORBIDDEN,
    Self::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
});

status_code_impl!(GameSessionRepositoryError {
    Self::NotFound { .. } => StatusCode::NOT_FOUND,
    Self::Conflict { .. } => StatusCode::CONFLICT,
    Self::Unavailable { .. } => StatusCode::SERVICE_UNAVAILABLE,
    Self::Internal { .. } => StatusCode::INTERNAL_SERVER_ERROR,
});

status_code_impl!(MessageRepositoryError {
    Self::NotFound { .. } => StatusCode::NOT_FOUND,
    Self::Conflict { .. } => StatusCode::CONFLICT,
    Self::Unavailable { .. } => StatusCode::SERVICE_UNAVAILABLE,
    Self::Internal { .. } => StatusCode::INTERNAL_SERVER_ERROR,
});

status_code_impl!(ContextObjectRepositoryError {
    Self::NotFound { .. } => StatusCode::NOT_FOUND,
    Self::Conflict { .. } => StatusCode::CONFLICT,
    Self::Unavailable { .. } => StatusCode::SERVICE_UNAVAILABLE,
    Self::Internal { .. } => StatusCode::INTERNAL_SERVER_ERROR,
});

status_code_impl!(UnitOfWorkError {
    Self::Unavailable { .. } => StatusCode::SERVICE_UNAVAILABLE,
    Self::Internal { .. } => StatusCode::INTERNAL_SERVER_ERROR,
});

status_code_impl!(NarratorError {
    Self::Unavailable { .. } => StatusCode::SERVICE_UNAVAILABLE,
    Self::InvalidQuery { .. } => StatusCode::INTERNAL_SERVER_ERROR,
    Self::InvalidResponse { .. } => StatusCode::INTERNAL_SERVER_ERROR,
});

status_code_impl!(MemoryExtractorError {
    Self::Unavailable { .. } => StatusCode::SERVICE_UNAVAILABLE,
    Self::InvalidQuery { .. } => StatusCode::INTERNAL_SERVER_ERROR,
    Self::InvalidResponse { .. } => StatusCode::INTERNAL_SERVER_ERROR,
});

status_code_impl!(AgentOrchestrationServiceError {
    Self::Narrator(error) => error.status_code(),
    Self::MemoryExtractor(error) => error.status_code(),
    Self::InvalidData { .. } => StatusCode::INTERNAL_SERVER_ERROR,
});

status_code_impl!(EmbedderError {
    Self::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
    Self::InvalidQuery => StatusCode::INTERNAL_SERVER_ERROR,
    Self::InvalidResponse => StatusCode::INTERNAL_SERVER_ERROR,
});

status_code_impl!(PromptAssemblerError {
    Self::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
    Self::InvalidInput => StatusCode::INTERNAL_SERVER_ERROR,
});

status_code_impl!(RetrivialServiceError {
    Self::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
    Self::Internal { .. } => StatusCode::INTERNAL_SERVER_ERROR,
});

status_code_impl!(VectorSearcherError {
    Self::Unavailable { .. } => StatusCode::SERVICE_UNAVAILABLE,
    Self::InvalidResponse { .. } => StatusCode::INTERNAL_SERVER_ERROR,
});

status_code_impl!(UseCaseError {
    Self::Domain(_) => StatusCode::UNPROCESSABLE_ENTITY,
    Self::User(error) => error.status_code(),
    Self::GameSessionRepository(error) => error.status_code(),
    Self::MessageRepository(error) => error.status_code(),
    Self::ContextObjectRepository(error) => error.status_code(),
    Self::UnitOfWork(error) => error.status_code(),
    Self::AgentOrchestration(error) => error.status_code(),
    Self::Embedder(error) => error.status_code(),
    Self::PromptAssembler(error) => error.status_code(),
    Self::Retrivial(error) => error.status_code(),
    Self::VectorSearcher(error) => error.status_code(),
});

fn status_code_for_use_case_error(error: &UseCaseError) -> StatusCode {
    error.status_code()
}
