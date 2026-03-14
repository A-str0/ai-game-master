use crate::{AppResult, services::PromptDTO};

pub struct AgentResponse(pub String);

#[async_trait::async_trait]
pub trait Agnet: Send + Sync {
    async fn generate(&self, prompt: PromptDTO) -> AppResult<AgentResponse>;
}
