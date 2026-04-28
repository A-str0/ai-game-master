use application::use_cases::{
    CreateSessionResponse, GameSessionModeDTO, GetMessageResponse, GetMessagesResponse,
    GetSessionResponse, MessageRoleDTO, SendMessageResponse,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// JSON payload returned after `POST /api/sessions`.
#[derive(Serialize)]
pub struct CreateSessionResponseDto {
    /// Newly created session identifier.
    pub session_id: Uuid,
    /// RNG seed assigned to the session.
    pub seed: i64,
    /// RFC3339 timestamp when the session was created.
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

/// Wire representation of a session mode.
#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionModeDto {
    /// Single-player session.
    Solo,
    /// Multiplayer session.
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

/// JSON payload returned by `GET /api/sessions/{session_id}`.
#[derive(Serialize)]
pub struct GetSessionResponseDto {
    /// Session identifier.
    pub id: Uuid,
    /// Owner identifier.
    pub owner_id: Uuid,
    /// Number of retrieval candidates requested before reranking.
    pub retrivial_k: u8,
    /// Prompt memory budget configured for the session.
    pub memory_budget: u32,
    /// Configured play mode.
    pub session_mode: SessionModeDto,
    /// RFC3339 timestamp of the latest recorded activity, when present.
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

/// Wire representation of a message role.
#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageRoleDto {
    /// Message authored by the player.
    Player,
    /// Message authored by the game master.
    Gm,
    /// Message authored by internal system logic.
    System,
}

impl From<MessageRoleDTO> for MessageRoleDto {
    fn from(value: MessageRoleDTO) -> Self {
        match value {
            MessageRoleDTO::Player => Self::Player,
            MessageRoleDTO::Gm => Self::Gm,
            MessageRoleDTO::System => Self::System,
        }
    }
}

/// JSON payload returned for one transcript message.
#[derive(Serialize)]
pub struct MessageResponseDto {
    /// Message identifier.
    pub id: Uuid,
    /// Session identifier.
    pub session_id: Uuid,
    /// Author role.
    pub role: MessageRoleDto,
    /// Raw stored text.
    pub text: String,
    /// RFC3339 timestamp when the message was created.
    pub ts: String,
}

impl From<GetMessageResponse> for MessageResponseDto {
    fn from(value: GetMessageResponse) -> Self {
        Self {
            id: value.id.0,
            session_id: value.session_id.0,
            role: MessageRoleDto::from(value.role),
            text: value.text,
            ts: value.ts.to_rfc3339(),
        }
    }
}

/// Query parameters accepted by `GET /api/messages`.
#[derive(Deserialize)]
pub struct GetMessagesRequest {
    /// Session whose messages should be returned.
    pub session_id: Uuid,
    /// Maximum number of messages to return.
    pub limit: Option<usize>,
}

/// JSON payload returned by `GET /api/messages`.
#[derive(Serialize)]
pub struct GetMessagesResponseDto {
    /// Messages ordered from newest to oldest.
    pub messages: Vec<MessageResponseDto>,
}

impl From<GetMessagesResponse> for GetMessagesResponseDto {
    fn from(value: GetMessagesResponse) -> Self {
        Self {
            messages: value
                .messages
                .into_iter()
                .map(MessageResponseDto::from)
                .collect(),
        }
    }
}

/// JSON body accepted by `POST /api/messages`.
#[derive(Deserialize)]
pub struct SendMessageRequest {
    /// Session identifier that should receive the message.
    pub session_id: Uuid,
    /// Raw player text.
    pub text: String,
}

/// JSON payload returned after `POST /api/messages`.
#[derive(Serialize)]
pub struct SendMessageResponseDto {
    /// Identifier of the stored player message.
    pub player_message_id: Uuid,
}

impl From<SendMessageResponse> for SendMessageResponseDto {
    fn from(value: SendMessageResponse) -> Self {
        Self {
            player_message_id: value.player_message_id.0,
        }
    }
}

/// Standard error payload returned by the HTTP API.
#[derive(Serialize)]
pub struct ErrorResponse {
    /// Human-readable error string.
    pub error: String,
}
