use domain::{
    aggregates::{GameSession, Message},
    value_objects::MessageId,
};

use crate::AppResult;

#[derive(Debug, Clone)]
pub struct PromptAssemblyResult {
    pub gm_message_id: MessageId,
    pub gm_text: String,
}

#[async_trait::async_trait]
pub trait PromptAssemblyService: Send + Sync {
    async fn assemble(
        &self,
        session: &GameSession,
        player_message: &Message,
    ) -> AppResult<PromptAssemblyResult>;
}
