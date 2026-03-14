use domain::aggregates::{GameSession, Message};

use crate::AppResult;

/// DTO
#[derive(Debug, Clone)]
pub struct ContextObjectDTO {}

/// DTO
#[derive(Debug, Clone)]
pub struct PromptDTO {
    system_prompt: String,
    world_summary: String,
    retrived_objects: Vec<ContextObjectDTO>,
    player_action: String,
    instructions: String,
}

#[async_trait::async_trait]
pub trait PromptAssemblyService: Send + Sync {
    async fn assemble(
        &self,
        session: &GameSession,
        player_message: &Message,
    ) -> AppResult<PromptDTO>;
}
