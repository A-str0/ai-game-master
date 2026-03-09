use std::sync::Arc;

use domain::Identifiable;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    AppError,
    ports::{GameSessionRepository, PortError, RepoError, UserAccessPort},
    use_cases::{AppResult, UseCase},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum GameSessionModeDTO {
    Solo,
    Multi,
}

#[derive(Serialize, Deserialize)]
pub struct GetSessionCommand(Uuid);

#[derive(Serialize, Deserialize)]
pub struct GetSessionResponse {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub retrivial_k: u8,
    pub memory_budget: u32,
    pub session_mode: GameSessionModeDTO,
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
            PortError::NotFound => AppError::NotFound(command.0.into()),
            PortError::Forbidden => AppError::Forbidden,
            PortError::Unavailable => AppError::Unavailable,
        })?;

        let session = self
            .sessions_repo
            .get_by_id(&command.0.into())
            .await
            .map_err(|err| match err {
                RepoError::NotFound => AppError::NotFound(command.0.into()),
                RepoError::Conflict => AppError::Conflict,
                RepoError::Unavailable => AppError::Unavailable,
            })?;

        if session.owner_id() != current_user.id() {
            return Err(AppError::Forbidden);
        }

        Ok(GetSessionResponse {
            id: (*session.id()).into(),
            owner_id: (*session.owner_id()).into(),
            retrivial_k: session.config().retrivial_k(),
            memory_budget: session.config().memory_budget(),
            session_mode: match *session.config().session_mode() {
                domain::value_objects::GameSessionMode::Solo => GameSessionModeDTO::Solo,
                domain::value_objects::GameSessionMode::Multi => GameSessionModeDTO::Multi,
            },
        })
    }
}
