use thiserror::Error;

#[derive(Debug, Clone)]
pub struct PromptContextObject {
    pub title: String,
    pub summary: String,
}

#[derive(Debug, Clone)]
pub struct PromptInput {
    pub system_prompt: String,
    pub world_summary: String,
    pub retrieved_objects: Vec<PromptContextObject>,
    pub player_action: String,
    pub instructions: String,
}

#[derive(Debug, Clone)]
pub enum AgentOrchestratorResponse {
    Text(String),
    CreateContextObject,
}

#[derive(Debug, Error)]
pub enum AgentOrchestratorError {
    #[error("agent backend unavailable")]
    Unavailable,
}

pub type AgentResult<T> = Result<T, AgentOrchestratorError>;

#[async_trait::async_trait]
pub trait AgentOrchestrator: Send + Sync {
    async fn generate(&self, prompt: &PromptInput) -> AgentResult<AgentOrchestratorResponse>;
}
