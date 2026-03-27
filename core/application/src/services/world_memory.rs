use domain::{
    aggregates::{ContextObject, GameSession},
    value_objects::GameSessionId,
};

use crate::{AppResult, ports::ProposedContextObject};

#[async_trait::async_trait]
pub trait WorldMemoryManager: Send + Sync {
    async fn initialize_session(&self, session_id: GameSessionId) -> AppResult<()>;

    async fn create_context_object(
        &self,
        session: &GameSession,
        object: ProposedContextObject,
    ) -> AppResult<ContextObject>;
}
