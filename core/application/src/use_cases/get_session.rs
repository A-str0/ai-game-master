use std::sync::Arc;

use domain::{Identifiable, aggregates::GameSession, value_objects::GameSessionId};

use crate::{
    AppError,
    ports::{GameSessionRepository, PortError, UserAccessPort},
    use_cases::{AppResult, UseCase},
};

pub struct GetSessionCommand(GameSessionId);

pub struct GetSessionOutput(GameSession);

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
impl UseCase<GetSessionCommand, GetSessionOutput> for GetSessionUseCase {
    async fn execute(&self, command: GetSessionCommand) -> AppResult<GetSessionOutput> {
        let current_user = self.user_access.get_user().await.map_err(|err| match err {
            PortError::NotFound => AppError::NotFound(command.0.into()),
            PortError::Forbidden => AppError::Forbidden,
            PortError::Unavailable => AppError::Unavailable,
        })?;

        let session = self
            .sessions_repo
            .get(&command.0)
            .await
            .map_err(|err| match err {
                crate::ports::RepoError::NotFound => AppError::NotFound(command.0.into()),
                crate::ports::RepoError::Conflict => AppError::Conflict,
                crate::ports::RepoError::Unavailable => AppError::Unavailable,
            })?;

        if session.owner_id() != current_user.id() {
            return Err(AppError::Forbidden);
        }

        Ok(GetSessionOutput(session))
    }
}
