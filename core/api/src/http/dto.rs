use application::use_cases::{
    CreateSessionResponse, GameSessionModeDTO, GetSessionResponse, SendMessageResponse,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize)]
pub struct CreateSessionResponseDto {
    pub session_id: Uuid,
    pub seed: i64,
    pub created_ts: String,
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

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionModeDto {
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
pub struct GetSessionResponseDto {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub retrivial_k: u8,
    pub memory_budget: u32,
    pub session_mode: SessionModeDto,
    pub last_activity_ts: Option<String>,
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

#[derive(Deserialize)]
pub struct SendMessageRequest {
    pub session_id: Uuid,
    pub text: String,
}

#[derive(Serialize)]
pub struct SendMessageResponseDto {
    pub player_message_id: Uuid,
}

impl From<SendMessageResponse> for SendMessageResponseDto {
    fn from(value: SendMessageResponse) -> Self {
        Self {
            player_message_id: value.player_message_id.0,
        }
    }
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}
