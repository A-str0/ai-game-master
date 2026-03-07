use std::sync::Arc;

use domain::{
    Identifiable,
    aggregates::GameSession,
    value_objects::{GameSessionConfig, GameSessionId, GameSessionMode, UserId},
};

use crate::{
    ports::GameSessionRepository,
    use_cases::{AppResult, UseCase},
};

pub struct CreateSessionCommand {
    pub owner_id: UserId,
    pub session_mode: GameSessionMode,
    pub config: GameSessionConfig,
}

pub struct CreateSessionResult {
    pub session_id: GameSessionId,
}

pub struct CreateSessionUseCase<R> {
    repo: Arc<R>,
}

impl<R> CreateSessionUseCase<R> {
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }
}

impl<R> UseCase<CreateSessionCommand, CreateSessionResult> for CreateSessionUseCase<R>
where
    R: GameSessionRepository,
{
    async fn execute(&self, command: CreateSessionCommand) -> AppResult<CreateSessionResult> {
        let session = GameSession::new(command.owner_id, command.session_mode, command.config);
        self.repo.create(&session).await?;

        Ok(CreateSessionResult {
            session_id: *session.id(),
        })
    }
}
