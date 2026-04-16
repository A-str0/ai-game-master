use std::sync::Arc;

use domain::{
    Identifiable,
    value_objects::{GameSessionId, GameSessionMode, UserId},
};

use crate::{
    ports::{GameSessionRepository, UserPort, UserPortError},
    use_cases::{UseCase, UseCaseResult},
};

/// Transport-friendly representation of [`GameSessionMode`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameSessionModeDTO {
    /// Single-player session.
    Solo,
    /// Multiplayer session.
    Multi,
}

impl From<GameSessionMode> for GameSessionModeDTO {
    fn from(val: GameSessionMode) -> Self {
        match val {
            GameSessionMode::Solo => GameSessionModeDTO::Solo,
            GameSessionMode::Multi => GameSessionModeDTO::Multi,
        }
    }
}

/// Command that fetches one session visible to the current user.
pub struct GetSessionCommand {
    /// Session identifier to load.
    pub session_id: GameSessionId,
}

/// Read model returned by [`GetSessionUseCase`].
pub struct GetSessionResponse {
    /// Session identifier.
    pub id: GameSessionId,
    /// Session owner identifier.
    pub owner_id: UserId,
    /// Number of vector-search candidates requested before reranking.
    pub retrivial_k: u8,
    /// Prompt memory budget configured for the session.
    pub memory_budget: u32,
    /// Configured play mode.
    pub session_mode: GameSessionModeDTO,
    /// Timestamp of the latest recorded activity, if any.
    pub last_activity_ts: Option<chrono::DateTime<chrono::Utc>>,
}

/// Use case that loads one session and enforces ownership.
pub struct GetSessionUseCase {
    sessions_repo: Arc<dyn GameSessionRepository>,
    current_user: Arc<dyn UserPort>,
}

impl GetSessionUseCase {
    /// Creates the use case with its required dependencies.
    pub fn new(
        sessions_repo: Arc<dyn GameSessionRepository>,
        current_user: Arc<dyn UserPort>,
    ) -> Self {
        Self {
            sessions_repo,
            current_user,
        }
    }
}

#[async_trait::async_trait]
impl UseCase<GetSessionCommand, GetSessionResponse> for GetSessionUseCase {
    async fn execute(&self, command: GetSessionCommand) -> UseCaseResult<GetSessionResponse> {
        let current_user_id = self.current_user.current_user_id().await?;
        let session = self.sessions_repo.get_by_id(&command.session_id).await?;

        if session.owner_id() != &current_user_id {
            return Err(UserPortError::Forbidden.into());
        }

        Ok(GetSessionResponse {
            id: *session.id(),
            owner_id: *session.owner_id(),
            retrivial_k: session.config().retrivial_k(),
            memory_budget: session.config().memory_budget(),
            session_mode: GameSessionModeDTO::from(*session.config().session_mode()),
            last_activity_ts: session.updated_ts(),
        })
    }
}
