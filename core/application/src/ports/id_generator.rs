use domain::value_objects::{ContextObjectId, GameSessionId, MessageId};

#[async_trait::async_trait]
pub trait IdGenerator: Send + Sync {
    async fn next_game_session_id(&self) -> GameSessionId;
    async fn next_message_id(&self) -> MessageId;
    async fn next_context_object_id(&self) -> ContextObjectId;
}
