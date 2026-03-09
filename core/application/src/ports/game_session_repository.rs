use domain::{aggregates::GameSession, value_objects::GameSessionId};

use crate::ports::RepoResult;

#[async_trait::async_trait]
pub trait GameSessionRepository: Send + Sync {
    async fn create(&self, session: &GameSession) -> RepoResult<()>;
    async fn get_by_id(&self, id: &GameSessionId) -> RepoResult<GameSession>;
}
