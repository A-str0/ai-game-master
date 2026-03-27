use std::sync::Arc;

use domain::{
    Identifiable,
    value_objects::{GameSessionId, GameSessionMode, UserId},
};

use crate::{
    AppError,
    ports::{CurrentUser, GameSessionRepository},
    use_cases::{AppResult, UseCase},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameSessionModeDTO {
    Solo,
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

pub struct GetSessionCommand {
    pub session_id: GameSessionId,
}

pub struct GetSessionResponse {
    pub id: GameSessionId,
    pub owner_id: UserId,
    pub retrivial_k: u8,
    pub memory_budget: u32,
    pub session_mode: GameSessionModeDTO,
    pub last_activity_ts: Option<chrono::DateTime<chrono::Utc>>,
}

pub struct GetSessionUseCase {
    sessions_repo: Arc<dyn GameSessionRepository>,
    current_user: Arc<dyn CurrentUser>,
}

impl GetSessionUseCase {
    pub fn new(
        sessions_repo: Arc<dyn GameSessionRepository>,
        current_user: Arc<dyn CurrentUser>,
    ) -> Self {
        Self {
            sessions_repo,
            current_user,
        }
    }
}

#[async_trait::async_trait]
impl UseCase<GetSessionCommand, GetSessionResponse> for GetSessionUseCase {
    async fn execute(&self, command: GetSessionCommand) -> AppResult<GetSessionResponse> {
        let current_user_id = self.current_user.current_user_id().await?;

        let session = self.sessions_repo.get_by_id(&command.session_id).await?;

        if session.owner_id() != &current_user_id {
            return Err(AppError::Forbidden {
                details: format!(
                    "user {} is not allowed to access session {} owned by {}",
                    current_user_id.0,
                    command.session_id.0,
                    session.owner_id().0
                ),
            });
        }

        Ok(GetSessionResponse {
            id: *session.id(),
            owner_id: *session.owner_id(),
            retrivial_k: session.config().retrivial_k(),
            memory_budget: session.config().memory_budget(),
            session_mode: GameSessionModeDTO::from(*session.config().session_mode()),
            last_activity_ts: session.last_activity_ts(),
        })
    }
}
