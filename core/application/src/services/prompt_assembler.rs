use domain::aggregates::{GameSession, Message};

use crate::{
    AppResult,
    ports::{PromptContextObject, PromptInput},
};

#[async_trait::async_trait]
pub trait PromptAssembler: Send + Sync {
    async fn assemble(
        &self,
        session: &GameSession,
        recent_messages: &[Message],
        player_message: &Message,
        retrieved_objects: &[PromptContextObject],
    ) -> AppResult<PromptInput>;
}
