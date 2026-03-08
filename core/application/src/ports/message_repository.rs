use domain::value_objects::{GameSessionId, UserId};

use crate::ports::RepoResult;

#[async_trait::async_trait]
pub trait MessageRepository: Send + Sync {
    async fn create(&self, session_id: GameSessionId, owner_id: UserId) -> RepoResult<()>;
}
