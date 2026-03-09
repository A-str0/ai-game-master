use std::sync::Arc;

use domain::{
    Identifiable,
    value_objects::{GameSessionId, GameSessionMode, UserId},
};

use crate::{
    AppError,
    ports::{GameSessionRepository, PortError, RepoError, UserAccessPort},
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
    pub requester_id: UserId,
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
    user_access: Arc<dyn UserAccessPort>,
}

impl GetSessionUseCase {
    pub fn new(
        sessions_repo: Arc<dyn GameSessionRepository>,
        user_access: Arc<dyn UserAccessPort>,
    ) -> Self {
        Self {
            sessions_repo,
            user_access,
        }
    }
}

#[async_trait::async_trait]
impl UseCase<GetSessionCommand, GetSessionResponse> for GetSessionUseCase {
    async fn execute(&self, command: GetSessionCommand) -> AppResult<GetSessionResponse> {
        let current_user = self.user_access.get_user().await.map_err(|err| match err {
            PortError::NotFound => AppError::NotFound(command.requester_id.0),
            PortError::Forbidden => AppError::Forbidden,
            PortError::Unavailable => AppError::Unavailable,
        })?;

        let session = self
            .sessions_repo
            .get_by_id(&command.session_id)
            .await
            .map_err(|err| match err {
                RepoError::NotFound => AppError::NotFound(command.session_id.0),
                RepoError::Conflict => AppError::Conflict,
                RepoError::Unavailable => AppError::Unavailable,
            })?;

        if current_user.id() != &command.requester_id {
            return Err(AppError::Forbidden);
        }

        if session.owner_id() != current_user.id() {
            return Err(AppError::Forbidden);
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
