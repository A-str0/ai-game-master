use std::sync::Arc;

use domain::{
    Identifiable,
    aggregates::GameSession,
    value_objects::{GameSessionConfig, GameSessionId, UserId},
};

use crate::{
    AppError,
    ports::{Clock, GameSessionRepository, IdGenerator, PortError, UserAccessPort},
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
    clock: Arc<dyn Clock>,
    id_generator: Arc<dyn IdGenerator>,
}

impl CreateSessionUseCase {
    pub fn new(
        sessions_repo: Arc<dyn GameSessionRepository>,
        user_access: Arc<dyn UserAccessPort>,
        clock: Arc<dyn Clock>,
        id_generator: Arc<dyn IdGenerator>,
    ) -> Self {
        Self {
            sessions_repo,
            user_access,
            clock,
            id_generator,
        }
    }
}

#[async_trait::async_trait]
impl UseCase<CreateSessionCommand, CreateSessionResponse> for CreateSessionUseCase {
    async fn execute(&self, command: CreateSessionCommand) -> AppResult<CreateSessionResponse> {
        let current_user = self.user_access.get_user().await.map_err(|err| match err {
            PortError::NotFound => AppError::NotFound(command.owner_id.0),
            PortError::Forbidden => AppError::Forbidden,
            PortError::Unavailable => AppError::Unavailable,
        })?;

        if current_user.id() != &command.owner_id {
            return Err(AppError::Forbidden);
        }

        let session = GameSession::new(
            self.id_generator.next_game_session_id().await,
            command.owner_id,
            GameSessionConfig::default(),
            self.clock.now().await,
        );
        self.sessions_repo
            .create(&session)
            .await
            .map_err(|err| match err {
                crate::ports::RepoError::NotFound => AppError::NotFound(command.owner_id.0),
                crate::ports::RepoError::Conflict => AppError::Conflict,
                crate::ports::RepoError::Unavailable => AppError::Unavailable,
            })?;

        Ok(CreateSessionResponse {
            session_id: *session.id(),
        })
    }
}
