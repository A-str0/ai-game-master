use std::sync::Arc;

use domain::{
    Identifiable,
    aggregates::GameSession,
    value_objects::{GameSessionConfig, GameSessionId, UserId},
};

use crate::{
    AppError,
    ports::{GameSessionRepository, PortError, UserAccessPort},
    use_cases::{AppResult, UseCase},
};

pub struct CreateSessionCommand {
    pub owner_id: UserId,
}

pub struct CreateSessionResponse {
    pub session_id: GameSessionId,
}

pub struct CreateSessionUseCase {
    sessions_repo: Arc<dyn GameSessionRepository>,
    user_access: Arc<dyn UserAccessPort>,
}

impl CreateSessionUseCase {
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
impl UseCase<CreateSessionCommand, CreateSessionResponse> for CreateSessionUseCase {
    async fn execute(&self, command: CreateSessionCommand) -> AppResult<CreateSessionResponse> {
        let current_user = self.user_access.get_user().await.map_err(|err| match err {
            PortError::NotFound => AppError::NotFound(command.owner_id.into()),
            PortError::Forbidden => AppError::Forbidden,
            PortError::Unavailable => AppError::Unavailable,
        })?;

        if current_user.id() != &command.owner_id {
            return Err(AppError::Forbidden);
        }

        let session = GameSession::new(command.owner_id, GameSessionConfig::default()); // TODO: change from default()
        self.sessions_repo
            .create(&session)
            .await
            .map_err(|err| match err {
                crate::ports::RepoError::NotFound => AppError::NotFound(command.owner_id.into()),
                crate::ports::RepoError::Conflict => AppError::Conflict,
                crate::ports::RepoError::Unavailable => AppError::Unavailable,
            })?;

        Ok(CreateSessionResponse {
            session_id: *session.id(),
        })
    }
}
