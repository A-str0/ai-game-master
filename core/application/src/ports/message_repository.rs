use domain::aggregates::Message;

use crate::ports::RepoResult;

#[async_trait::async_trait]
pub trait MessageRepository: Send + Sync {
    async fn create(&self, message: &Message) -> RepoResult<()>;
}
