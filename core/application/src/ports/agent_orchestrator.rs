use std::collections::HashMap;

use domain::value_objects::{AttributeValue, ContextObjectType};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct PromptContextObject {
    pub title: String,
    pub summary: String,
}

#[derive(Debug, Clone)]
pub struct PromptMessage {
    pub role: String,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct PromptInput {
    pub system_prompt: String,
    pub world_summary: String,
    pub recent_messages: Vec<PromptMessage>,
    pub retrieved_objects: Vec<PromptContextObject>,
    pub player_action: String,
    pub instructions: String,
}

#[derive(Debug, Clone)]
pub struct ProposedContextObject {
    pub object_type: ContextObjectType,
    pub title: String,
    pub short_desc: String,
    pub long_desc: Option<String>,
    pub attributes: HashMap<String, AttributeValue>,
    pub importance_score: f32,
}

#[derive(Debug, Clone)]
pub enum AgentOrchestratorResponse {
    Text(String),
    CreateContextObjects {
        message: String,
        objects: Vec<ProposedContextObject>,
    },
}

#[derive(Debug, Error)]
pub enum AgentOrchestratorError {
    #[error("AgentOrchestrator backend unavaliable ({details})")]
    Unavailable { details: String },
    #[error("AgentOrchestrator backend received invalid query ({details})")]
    InvalidQuery { details: String },
    #[error("AgentOrchestrator backend returned invalid output ({details})")]
    InvalidResponse { details: String },
}

pub type AgentResult<T> = Result<T, AgentOrchestratorError>;

#[async_trait::async_trait]
pub trait AgentOrchestrator: Send + Sync {
    async fn generate(&self, prompt: &PromptInput) -> AgentResult<AgentOrchestratorResponse>;
}
