use std::sync::Arc;

use domain::{
    Identifiable,
    aggregates::{GameSession, GameSessionConfig},
    value_objects::{GameSessionId, RngState},
};

use crate::{
    ports::{Clock, GameSessionRepository, IdGenerator, UserPort, VectorSearcher},
    use_cases::{UseCase, UseCaseResult},
};

pub struct CreateSessionCommand;

pub struct CreateSessionResponse {
    pub session_id: GameSessionId,
    pub seed: u64,
    pub created_ts: chrono::DateTime<chrono::Utc>,
}

pub struct CreateSessionUseCase {
    sessions_repo: Arc<dyn GameSessionRepository>,
    current_user: Arc<dyn UserPort>,
    clock: Arc<dyn Clock>,
    id_generator: Arc<dyn IdGenerator>,
    vector_searcher: Arc<dyn VectorSearcher>,
}

impl CreateSessionUseCase {
    pub fn new(
        sessions_repo: Arc<dyn GameSessionRepository>,
        current_user: Arc<dyn UserPort>,
        clock: Arc<dyn Clock>,
        id_generator: Arc<dyn IdGenerator>,
        vector_searcher: Arc<dyn VectorSearcher>,
    ) -> Self {
        Self {
            sessions_repo,
            current_user,
            clock,
            id_generator,
            vector_searcher,
        }
    }
}

#[async_trait::async_trait]
impl UseCase<CreateSessionCommand, CreateSessionResponse> for CreateSessionUseCase {
    async fn execute(
        &self,
        _command: CreateSessionCommand,
    ) -> UseCaseResult<CreateSessionResponse> {
        let current_user_id = self.current_user.current_user_id().await?;

        let rng_state = RngState::default();
        let seed = rng_state.seed();
        let created_ts = self.clock.now().await;
        let session = GameSession::new(
            self.id_generator.next_game_session_id().await,
            current_user_id,
            GameSessionConfig::default(),
            rng_state,
            created_ts,
        );

        self.sessions_repo.insert(&session).await?;
        self.vector_searcher
            .ensure_session_collection(*session.id())
            .await?;

        Ok(CreateSessionResponse {
            session_id: *session.id(),
            seed,
            created_ts,
        })
    }
}
