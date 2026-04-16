use std::sync::Arc;

use application::use_cases::SendMessageCommand;
use axum::{
    Json,
    extract::{Path, State},
};
use domain::value_objects::GameSessionId;
use uuid::Uuid;

use crate::app_service::ApiApplicationService;

use super::{
    auth::RequestUser,
    dto::{
        CreateSessionResponseDto, GetSessionResponseDto, SendMessageRequest, SendMessageResponseDto,
    },
    error::{ApiError, ApiResult},
};

pub async fn create_session_handle(
    State(application): State<Arc<dyn ApiApplicationService>>,
    request_user: RequestUser,
) -> ApiResult<Json<CreateSessionResponseDto>> {
    let response = application.create_session(request_user.user_id).await?;

    Ok(Json(CreateSessionResponseDto::from(response)))
}

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
