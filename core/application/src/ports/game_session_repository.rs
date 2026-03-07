use domain::{aggregates::GameSession, value_objects::GameSessionId};

use crate::ports::RepoResult;

pub trait GameSessionRepository {
    async fn create(&self, session: &GameSession) -> RepoResult<()>;
    async fn get(&self, id: &GameSessionId) -> RepoResult<GameSession>;
}
