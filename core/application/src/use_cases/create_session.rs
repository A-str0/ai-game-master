use std::sync::Arc;

use domain::{
    Identifiable,
    aggregates::GameSession,
    value_objects::{GameSessionConfig, GameSessionId, UserId},
};

use crate::{
    ports::{GameSessionRepository, UserAccessPort},
    use_cases::{AppResult, UseCase},
};

pub struct CreateSessionCommand {
    pub owner_id: UserId,
}

pub struct CreateSessionResult {
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
impl UseCase<CreateSessionCommand, CreateSessionResult> for CreateSessionUseCase {
    async fn execute(&self, command: CreateSessionCommand) -> AppResult<CreateSessionResult> {
        if let Err(crate::ports::PortError::NotFound) = self.user_access.get_user().await {
            return Err(crate::AppError::OwnerNotFound);
        }

        let session = GameSession::new(command.owner_id, GameSessionConfig::default()); // TODO: change from default()
        self.sessions_repo.create(&session).await?;

        Ok(CreateSessionResult {
            session_id: *session.id(),
        })
    }
}
