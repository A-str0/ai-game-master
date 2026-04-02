use std::sync::Arc;

use anyhow::Context;
use application::{
    ports::{
        AgentOrchestrator, AgentOrchestratorError, ContextObjectRepository,
        ContextObjectRepositoryError, GameSessionRepository, GameSessionRepositoryError,
        MessageRepository, MessageRepositoryError, UserPort, UserPortError, VectorSearcher,
        VectorSearcherError,
    },
    services::{
        Clock, Embedder, EmbedderError, EmbeddingService, IdGenerator, PromptAssembler,
        PromptAssemblerError, PromptAssemblyService, RetrivialService, RetrivialServiceError,
        RetrivialServicePort, UtcClock, UuidGenerator,
    },
    use_cases::{
        CreateSessionCommand, CreateSessionResponse, CreateSessionUseCase, GameSessionModeDTO,
        GetSessionCommand, GetSessionResponse, GetSessionUseCase, SendMessageCommand,
        SendMessageResponse, SendMessageUseCase, UseCase, UseCaseError,
    },
};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use domain::value_objects::{GameSessionId, UserId};
use infrastructure::{
    ports::{CurrentUserContext, DefaultAgentOrchestrator, RequestCurrentUser},
    repositories::connecion::PgDatabase,
    services::QdSearchService,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

type ApiResult<T> = std::result::Result<T, ApiError>;

#[derive(Clone)]
struct AppState {
    sessions_repo: Arc<dyn GameSessionRepository>,
    messages_repo: Arc<dyn MessageRepository>,
    context_object_repo: Arc<dyn ContextObjectRepository>,
    retrivial: Arc<dyn RetrivialServicePort>,
    agent: Arc<dyn AgentOrchestrator>,
    embedder: Arc<dyn Embedder>,
    vector_searcher: Arc<dyn VectorSearcher>,
    clock: Arc<dyn Clock>,
    id_generator: Arc<dyn IdGenerator>,
    prompt_assembly: Arc<dyn PromptAssembler>,
    current_user_id: UserId,
}

fn current_user(user_id: UserId) -> Arc<dyn UserPort> {
    let context = CurrentUserContext::from(user_id);
    Arc::new(RequestCurrentUser::new(context))
}

// TODO: fix DI
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        String::from("postgres://postgres:postgres@127.0.0.1:5432/ai_game_master")
    });
    let database = PgDatabase::new(&database_url).with_context(|| {
        format!("failed to initialize postgres database from DATABASE_URL: {database_url}")
    })?;

    let clock: Arc<dyn Clock> = Arc::new(UtcClock::new());
    let sessions_repo: Arc<dyn GameSessionRepository> = Arc::new(database.game_sessions());
    let messages_repo: Arc<dyn MessageRepository> = Arc::new(database.messages());
    let context_object_repo: Arc<dyn ContextObjectRepository> =
        Arc::new(database.context_objects());
    let embedder: Arc<dyn Embedder> = Arc::new(EmbeddingService::new().await?);
    let vector_searcher: Arc<dyn VectorSearcher> = Arc::new(QdSearchService::new());
    let retrivial: Arc<dyn RetrivialServicePort> = Arc::new(RetrivialService::new());
    let agent: Arc<dyn AgentOrchestrator> = Arc::new(DefaultAgentOrchestrator::new().await?);
    let id_generator: Arc<dyn IdGenerator> = Arc::new(UuidGenerator);
    let prompt_assembly: Arc<dyn PromptAssembler> = Arc::new(PromptAssemblyService::new());
    let current_user_id = UserId(Uuid::new_v4());

    let state = AppState {
        sessions_repo,
        messages_repo,
        context_object_repo,
        retrivial,
        agent,
        embedder,
        vector_searcher,
        clock,
        id_generator,
        prompt_assembly,
        current_user_id,
    };

    let app = Router::new()
        .route("/api/sessions", post(create_session_handle))
        .route("/api/sessions/{session_id}", get(get_session_handle))
        .route("/api/messages", post(send_message_handle))
        .with_state(state);

    let bind_addr = std::env::var("API_ADDR").unwrap_or_else(|_| String::from("0.0.0.0:3000"));
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .with_context(|| format!("failed to bind API listener on {bind_addr}"))?;

    axum::serve(listener, app)
        .await
        .context("api server failed")?;

    Ok(())
}

#[derive(Serialize)]
struct CreateSessionResponseDto {
    session_id: Uuid,
    seed: u64,
    created_ts: String,
}

impl From<CreateSessionResponse> for CreateSessionResponseDto {
    fn from(value: CreateSessionResponse) -> Self {
        Self {
            session_id: value.session_id.0,
            seed: value.seed,
            created_ts: value.created_ts.to_rfc3339(),
        }
    }
}

async fn create_session_handle(
    State(state): State<AppState>,
) -> ApiResult<Json<CreateSessionResponseDto>> {
    let use_case = CreateSessionUseCase::new(
        Arc::clone(&state.sessions_repo),
        current_user(state.current_user_id),
        Arc::clone(&state.clock),
        Arc::clone(&state.id_generator),
        Arc::clone(&state.vector_searcher),
    );

    let response = use_case.execute(CreateSessionCommand).await?;

    Ok(Json(CreateSessionResponseDto::from(response)))
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum SessionModeDto {
    Solo,
    Multi,
}

impl From<GameSessionModeDTO> for SessionModeDto {
    fn from(value: GameSessionModeDTO) -> Self {
        match value {
            GameSessionModeDTO::Solo => SessionModeDto::Solo,
            GameSessionModeDTO::Multi => SessionModeDto::Multi,
        }
    }
}

#[derive(Serialize)]
struct GetSessionResponseDto {
    id: Uuid,
    owner_id: Uuid,
    retrivial_k: u8,
    memory_budget: u32,
    session_mode: SessionModeDto,
    last_activity_ts: Option<String>,
}

impl From<GetSessionResponse> for GetSessionResponseDto {
    fn from(value: GetSessionResponse) -> Self {
        Self {
            id: value.id.0,
            owner_id: value.owner_id.0,
            retrivial_k: value.retrivial_k,
            memory_budget: value.memory_budget,
            session_mode: SessionModeDto::from(value.session_mode),
            last_activity_ts: value.last_activity_ts.map(|ts| ts.to_rfc3339()),
        }
    }
}

async fn get_session_handle(
    State(state): State<AppState>,
    Path(session_id_raw): Path<String>,
) -> ApiResult<Json<GetSessionResponseDto>> {
    let session_id = parse_game_session_id(&session_id_raw)?;
    let use_case = GetSessionUseCase::new(
        Arc::clone(&state.sessions_repo),
        current_user(state.current_user_id),
    );
    let response = use_case.execute(GetSessionCommand { session_id }).await?;

    Ok(Json(GetSessionResponseDto::from(response)))
}

#[derive(Deserialize)]
struct SendMessageRequest {
    session_id: Uuid,
    text: String,
}

#[derive(Serialize)]
struct SendMessageResponseDto {
    player_message_id: Uuid,
}

impl From<SendMessageResponse> for SendMessageResponseDto {
    fn from(value: SendMessageResponse) -> Self {
        Self {
            player_message_id: value.player_message_id.0,
        }
    }
}

async fn send_message_handle(
    State(state): State<AppState>,
    Json(body): Json<SendMessageRequest>,
) -> ApiResult<Json<SendMessageResponseDto>> {
    let use_case = SendMessageUseCase::new(
        Arc::clone(&state.sessions_repo),
        Arc::clone(&state.messages_repo),
        Arc::clone(&state.context_object_repo),
        current_user(state.current_user_id),
        Arc::clone(&state.prompt_assembly),
        Arc::clone(&state.retrivial),
        Arc::clone(&state.agent),
        Arc::clone(&state.embedder),
        Arc::clone(&state.vector_searcher),
        Arc::clone(&state.clock),
        Arc::clone(&state.id_generator),
    );

    let response = use_case
        .execute(SendMessageCommand {
            session_id: GameSessionId(body.session_id),
            text: body.text,
        })
        .await?;

    Ok(Json(SendMessageResponseDto::from(response)))
}

fn parse_game_session_id(value: &str) -> ApiResult<GameSessionId> {
    let parsed = Uuid::parse_str(value)
        .map_err(|_| ApiError::BadRequest(String::from("invalid session_id")))?;
    Ok(GameSessionId(parsed))
}

#[derive(Debug)]
enum ApiError {
    UseCase(UseCaseError),
    BadRequest(String),
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

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
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

status_code_impl!(AgentOrchestratorError {
    Self::Unavailable { .. } => StatusCode::SERVICE_UNAVAILABLE,
    Self::InvalidQuery { .. } => StatusCode::INTERNAL_SERVER_ERROR,
    Self::InvalidResponse { .. } => StatusCode::INTERNAL_SERVER_ERROR,
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
    Self::Agent(error) => error.status_code(),
    Self::Embedder(error) => error.status_code(),
    Self::PromptAssembler(error) => error.status_code(),
    Self::Retrivial(error) => error.status_code(),
    Self::VectorSearcher(error) => error.status_code(),
    Self::IntegerConversion(_) => StatusCode::INTERNAL_SERVER_ERROR,
});

fn status_code_for_use_case_error(error: &UseCaseError) -> StatusCode {
    error.status_code()
}
