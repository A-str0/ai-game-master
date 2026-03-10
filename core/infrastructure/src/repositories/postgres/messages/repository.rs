use application::ports::{MessageRepository, RepoResult};
use domain::{aggregates::Message, value_objects::GameSessionId};

pub struct PgMessageRepository {}

#[async_trait::async_trait]
impl MessageRepository for PgMessageRepository {
    async fn create(&self, message: &Message) -> RepoResult<()> {
        todo!()
    }

    async fn list_recent(
        &self,
        session_id: &GameSessionId,
        limit: usize,
    ) -> RepoResult<Vec<Message>> {
        todo!()
    }
}
