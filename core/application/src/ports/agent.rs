use thiserror::Error;

#[derive(Debug, Clone)]
pub struct PromptContextObject {
    pub title: String,
    pub summary: String,
}

#[derive(Debug, Clone)]
pub struct AgentPrompt {
    pub system_prompt: String,
    pub world_summary: String,
    pub retrieved_objects: Vec<PromptContextObject>,
    pub player_action: String,
    pub instructions: String,
}

pub struct AgentResponse(pub String);

#[derive(Debug, Error)]
pub enum AgentError {
    #[error("agent backend unavailable")]
    Unavailable,
}

pub type AgentResult<T> = Result<T, AgentError>;

#[async_trait::async_trait]
pub trait AgentPort: Send + Sync {
    async fn generate(&self, prompt: AgentPrompt) -> AgentResult<AgentResponse>;
}
