use std::sync::Arc;

use domain::{
    Identifiable,
    aggregates::{GameSession, GameSessionConfig},
    value_objects::{GameSessionId, RngState},
};

use crate::{
    ports::{UnitOfWorkFactory, UserPort, VectorSearcher},
    services::{Clock, IdGenerator},
    use_cases::{UseCase, UseCaseResult},
};

/// Command that requests creation of a new session for the current user.
pub struct CreateSessionCommand;

/// Result returned after a session has been created and provisioned.
pub struct CreateSessionResponse {
    /// Newly created session identifier.
    pub session_id: GameSessionId,
    /// RNG seed assigned to the session.
    pub seed: i64,
    /// Session creation timestamp.
    pub created_ts: chrono::DateTime<chrono::Utc>,
}

/// Use case that provisions a fresh game session for the current user.
pub struct CreateSessionUseCase {
    unit_of_work: Arc<dyn UnitOfWorkFactory>,
    current_user: Arc<dyn UserPort>,
    clock: Arc<dyn Clock>,
    id_generator: Arc<dyn IdGenerator>,
    vector_searcher: Arc<dyn VectorSearcher>,
}

impl CreateSessionUseCase {
    /// Creates the use case with its required dependencies.
    pub fn new(
        unit_of_work: Arc<dyn UnitOfWorkFactory>,
        current_user: Arc<dyn UserPort>,
        clock: Arc<dyn Clock>,
        id_generator: Arc<dyn IdGenerator>,
        vector_searcher: Arc<dyn VectorSearcher>,
    ) -> Self {
        Self {
            unit_of_work,
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

        let mut unit_of_work = self.unit_of_work.begin().await?;
        unit_of_work.insert_session(&session).await?;
        unit_of_work.commit().await?;
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
