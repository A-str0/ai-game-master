use std::sync::Arc;

use domain::{
    Identifiable,
    aggregates::{GameSession, GameSessionConfig},
    value_objects::{GameSessionId, RngState},
};

use crate::{
    AppError,
    ports::{Clock, CurrentUser, GameSessionRepository, IdGenerator},
    use_cases::{AppResult, UseCase},
};

pub struct CreateSessionCommand;

pub struct CreateSessionResponse {
    pub session_id: GameSessionId,
    pub seed: i64,
    pub created_ts: chrono::DateTime<chrono::Utc>,
}

pub struct CreateSessionUseCase {
    sessions_repo: Arc<dyn GameSessionRepository>,
    current_user: Arc<dyn CurrentUser>,
    clock: Arc<dyn Clock>,
    id_generator: Arc<dyn IdGenerator>,
}

impl CreateSessionUseCase {
    pub fn new(
        sessions_repo: Arc<dyn GameSessionRepository>,
        current_user: Arc<dyn CurrentUser>,
        clock: Arc<dyn Clock>,
        id_generator: Arc<dyn IdGenerator>,
    ) -> Self {
        Self {
            sessions_repo,
            current_user,
            clock,
            id_generator,
        }
    }
}

#[async_trait::async_trait]
impl UseCase<CreateSessionCommand, CreateSessionResponse> for CreateSessionUseCase {
    async fn execute(&self, _command: CreateSessionCommand) -> AppResult<CreateSessionResponse> {
        let current_user_id = self.current_user.current_user_id().await?;

        let rng_state = RngState::default();
        let seed = i64::try_from(rng_state.seed()).map_err(|_| AppError::Unavailable)?;
        let created_ts = self.clock.now().await;
        let session = GameSession::new(
            self.id_generator.next_game_session_id().await,
            current_user_id,
            GameSessionConfig::default(),
            rng_state,
            created_ts,
        );
        self.sessions_repo.create(&session).await?;

        Ok(CreateSessionResponse {
            session_id: *session.id(),
            seed,
            created_ts,
        })
    }
}
