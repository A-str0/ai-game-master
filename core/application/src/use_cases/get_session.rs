use std::sync::Arc;

use domain::{
    Identifiable,
    value_objects::{GameSessionId, GameSessionMode, UserId},
};
// use serde::{Deserialize, Serialize};

use crate::{
    AppError,
    ports::{GameSessionRepository, PortError, RepoError, UserAccessPort},
    use_cases::{AppResult, UseCase},
};

pub struct GetSessionCommand(GameSessionId);

// TODO
// #[derive(Serialize, Deserialize)]
pub struct GetSessionResponse {
    pub id: GameSessionId,
    pub owner_id: UserId,
    pub retrivial_k: u8,
    pub memory_budget: u32,
    pub session_mode: GameSessionMode,
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
            .get_by_id(&command.0)
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
            id: *session.id(),
            owner_id: *session.owner_id(),
            retrivial_k: session.config().retrivial_k(),
            memory_budget: session.config().memory_budget(),
            session_mode: *session.config().session_mode(),
        })
    }
}
