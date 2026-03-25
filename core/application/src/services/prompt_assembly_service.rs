use domain::aggregates::{GameSession, Message};

use crate::{AppResult, ports::PromptInput};

#[async_trait::async_trait]
pub trait PromptAssemblyService: Send + Sync {
    async fn assemble(
        &self,
        session: &GameSession,
        player_message: &Message,
    ) -> AppResult<PromptInput>;
}
