use std::sync::Arc;

use anyhow::Context;
use application::{
    AppError,
    ports::{AgentOrchestrator, CurrentUser, GameSessionRepository, MessageRepository},
    services::{PromptAssembler, RetrivialService},
    use_cases::{
        CreateSessionCommand, CreateSessionResponse, CreateSessionUseCase, GameSessionModeDTO,
        GetSessionCommand, GetSessionResponse, GetSessionUseCase, SendMessageCommand,
        SendMessageResponse, SendMessageUseCase, UseCase,
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
    ports::{
        CurrentUserContext, DefaultAgentOrchestrator, RequestCurrentUser, UtcClock, UuidGenerator,
    },
    repositories::connecion::PgDatabase,
    services::{OpenRouterEmbeddingService, PromptAssembly, QdRetrivialService, QdSearchService},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    sessions_repo: Arc<dyn GameSessionRepository>,
    messages_repo: Arc<dyn MessageRepository>,
    retrivial: Arc<dyn RetrivialService>,
    agent: Arc<dyn AgentOrchestrator>,
    clock: Arc<dyn application::ports::Clock>,
    id_generator: Arc<dyn application::ports::IdGenerator>,
    prompt_assembly: Arc<dyn PromptAssembler>,
    current_user_id: UserId,
}

fn current_user(user_id: UserId) -> Arc<dyn CurrentUser> {
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

    let clock: Arc<dyn application::ports::Clock> = Arc::new(UtcClock::new());
    let sessions_repo: Arc<dyn GameSessionRepository> = Arc::new(database.game_sessions());
    let messages_repo: Arc<dyn MessageRepository> = Arc::new(database.messages());
    let context_object_repo = Arc::new(database.context_objects());
    let embedder = Arc::new(OpenRouterEmbeddingService::new().await?);
    let vector_searcher = Arc::new(QdSearchService);
    let retrivial: Arc<dyn RetrivialService> = Arc::new(QdRetrivialService::new(
        embedder,
        vector_searcher,
        context_object_repo,
        Arc::clone(&clock),
    ));
    let agent: Arc<dyn AgentOrchestrator> = Arc::new(DefaultAgentOrchestrator::new().await?);
    let id_generator: Arc<dyn application::ports::IdGenerator> = Arc::new(UuidGenerator);
    let prompt_assembly: Arc<dyn PromptAssembler> = Arc::new(PromptAssembly::new());
    let current_user_id = UserId(Uuid::new_v4());

    let state = AppState {
        sessions_repo,
        messages_repo,
        retrivial,
        agent,
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
    seed: i64,
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
) -> Result<Json<CreateSessionResponseDto>, ApiError> {
    let use_case = CreateSessionUseCase::new(
        Arc::clone(&state.sessions_repo),
        current_user(state.current_user_id),
        Arc::clone(&state.clock),
        Arc::clone(&state.id_generator),
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
) -> Result<Json<GetSessionResponseDto>, ApiError> {
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
) -> Result<Json<SendMessageResponseDto>, ApiError> {
    let use_case = SendMessageUseCase::new(
        Arc::clone(&state.sessions_repo),
        Arc::clone(&state.messages_repo),
        current_user(state.current_user_id),
        Arc::clone(&state.prompt_assembly),
        Arc::clone(&state.retrivial),
        Arc::clone(&state.agent),
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

fn parse_game_session_id(value: &str) -> Result<GameSessionId, ApiError> {
    let parsed = Uuid::parse_str(value)
        .map_err(|_| ApiError::BadRequest(String::from("invalid session_id")))?;
    Ok(GameSessionId(parsed))
}

#[derive(Debug)]
enum ApiError {
    App(AppError),
    BadRequest(String),
}

impl From<AppError> for ApiError {
    fn from(value: AppError) -> Self {
        Self::App(value)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error) = match self {
            ApiError::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
            ApiError::App(error) => {
                let status = match error {
                    AppError::NotFound { .. } => StatusCode::NOT_FOUND,
                    AppError::Unauthenticated => StatusCode::UNAUTHORIZED,
                    AppError::Forbidden => StatusCode::FORBIDDEN,
                    AppError::Conflict { .. } => StatusCode::CONFLICT,
                    AppError::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
                    AppError::Domain(_) => StatusCode::UNPROCESSABLE_ENTITY,
                };

                (status, error.to_string())
            }
        };

        (status, Json(ErrorResponse { error })).into_response()
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}
