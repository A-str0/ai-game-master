use application::ports::{GameSessionRepository, RepoResult};
use domain::{aggregates::GameSession, value_objects::GameSessionId};

pub struct PgGameSessionRepository {}

#[async_trait::async_trait]
impl GameSessionRepository for PgGameSessionRepository {
    async fn create(&self, session: &GameSession) -> RepoResult<()> {
        todo!()
    }

    async fn get_by_id(&self, id: &GameSessionId) -> RepoResult<GameSession> {
        todo!()
    }

    async fn update(&self, session: &GameSession) -> RepoResult<()> {
        todo!()
    }
}
