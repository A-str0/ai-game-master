use application::ports::IdGenerator;
use domain::value_objects::{ContextObjectId, GameSessionId, MessageId};
use uuid::Uuid;

pub struct UuidGenerator;

#[async_trait::async_trait]
impl IdGenerator for UuidGenerator {
    async fn next_game_session_id(&self) -> GameSessionId {
        GameSessionId(Uuid::new_v4())
    }

    async fn next_message_id(&self) -> MessageId {
        MessageId(Uuid::new_v4())
    }

    async fn next_context_object_id(&self) -> ContextObjectId {
        ContextObjectId(Uuid::new_v4())
    }
}
