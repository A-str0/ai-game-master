use domain::{aggregates::Message, value_objects::GameSessionId};

use crate::AppResult;

#[async_trait::async_trait]
pub trait PromptAssemblyService: Send + Sync {
    async fn assemble(&self, session_id: &GameSessionId, player_message: &Message) -> AppResult<String>;
}
