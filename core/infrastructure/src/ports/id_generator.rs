use domain::value_objects::{ContextObjectId, GameSessionId, MessageId};

pub struct IdGenerator;

#[async_trait::async_trait]
impl application::ports::IdGenerator for IdGenerator {
    async fn next_game_session_id(&self) -> GameSessionId {
        todo!()
    }

    async fn next_message_id(&self) -> MessageId {
        todo!()
    }

    async fn next_context_object_id(&self) -> ContextObjectId {
        todo!()
    }
}
