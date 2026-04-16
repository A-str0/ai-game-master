use domain::value_objects::{ContextObjectId, GameSessionId, MessageId};
use uuid::Uuid;

/// Source of new identifiers for domain aggregates.
#[async_trait::async_trait]
pub trait IdGenerator: Send + Sync {
    /// Generates a new game session identifier.
    async fn next_game_session_id(&self) -> GameSessionId;
    /// Generates a new message identifier.
    async fn next_message_id(&self) -> MessageId;
    /// Generates a new context object identifier.
    async fn next_context_object_id(&self) -> ContextObjectId;
}

/// UUID-based production implementation of [`IdGenerator`].
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
