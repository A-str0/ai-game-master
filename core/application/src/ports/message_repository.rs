use domain::{aggregates::Message, value_objects::GameSessionId};

use crate::ports::RepoResult;

#[async_trait::async_trait]
pub trait MessageRepositoryPort: Send + Sync {
    async fn create(&self, message: &Message) -> RepoResult<()>;
    async fn list_recent(
        &self,
        session_id: &GameSessionId,
        limit: usize,
    ) -> RepoResult<Vec<Message>>;
}
