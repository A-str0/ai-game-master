use std::sync::Arc;

use application::use_cases::{GetMessagesCommand, SendMessageCommand};
use axum::{
    Json,
    extract::{Path, Query, State},
};
use domain::value_objects::{GameSessionId, MessageId};
use uuid::Uuid;

use crate::app_service::ApiApplicationService;

use super::{
    auth::RequestUser,
    dto::{
        CreateSessionResponseDto, GetMessagesRequest, GetMessagesResponseDto,
        GetSessionResponseDto, MessageResponseDto, SendMessageRequest, SendMessageResponseDto,
    },
    error::{ApiError, ApiResult},
};

/// Handles `POST /api/sessions`.
pub async fn create_session_handle(
    State(application): State<Arc<dyn ApiApplicationService>>,
    request_user: RequestUser,
) -> ApiResult<Json<CreateSessionResponseDto>> {
    let response = application.create_session(request_user.user_id).await?;

    Ok(Json(CreateSessionResponseDto::from(response)))
}

/// Handles `GET /api/sessions/{session_id}`.
pub async fn get_session_handle(
    State(application): State<Arc<dyn ApiApplicationService>>,
    request_user: RequestUser,
    Path(session_id_raw): Path<String>,
) -> ApiResult<Json<GetSessionResponseDto>> {
    let session_id = parse_game_session_id(&session_id_raw)?;
    let response = application
        .get_session(request_user.user_id, session_id)
        .await?;

    Ok(Json(GetSessionResponseDto::from(response)))
}

/// Handles `GET /api/messages`.
pub async fn get_messages_handle(
    State(application): State<Arc<dyn ApiApplicationService>>,
    request_user: RequestUser,
    Query(query): Query<GetMessagesRequest>,
) -> ApiResult<Json<GetMessagesResponseDto>> {
    let response = application
        .get_messages(
            request_user.user_id,
            GetMessagesCommand {
                session_id: GameSessionId(query.session_id),
                limit: query.limit.unwrap_or(50),
            },
        )
        .await?;

    Ok(Json(GetMessagesResponseDto::from(response)))
}

/// Handles `GET /api/messages/{message_id}`.
pub async fn get_message_handle(
    State(application): State<Arc<dyn ApiApplicationService>>,
    request_user: RequestUser,
    Path(message_id_raw): Path<String>,
) -> ApiResult<Json<MessageResponseDto>> {
    let message_id = parse_message_id(&message_id_raw)?;
    let response = application
        .get_message(request_user.user_id, message_id)
        .await?;

    Ok(Json(MessageResponseDto::from(response)))
}

/// Handles `POST /api/messages`.
pub async fn send_message_handle(
    State(application): State<Arc<dyn ApiApplicationService>>,
    request_user: RequestUser,
    Json(body): Json<SendMessageRequest>,
) -> ApiResult<Json<SendMessageResponseDto>> {
    let response = application
        .send_message(
            request_user.user_id,
            SendMessageCommand {
                session_id: GameSessionId(body.session_id),
                text: body.text,
            },
        )
        .await?;

    Ok(Json(SendMessageResponseDto::from(response)))
}

fn parse_game_session_id(value: &str) -> ApiResult<GameSessionId> {
    let parsed = Uuid::parse_str(value).map_err(|_| ApiError::bad_request("invalid session_id"))?;

    Ok(GameSessionId(parsed))
}

fn parse_message_id(value: &str) -> ApiResult<MessageId> {
    let parsed = Uuid::parse_str(value).map_err(|_| ApiError::bad_request("invalid message_id"))?;

    Ok(MessageId(parsed))
}
