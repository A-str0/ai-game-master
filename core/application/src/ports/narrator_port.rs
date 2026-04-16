use std::collections::HashMap;

use domain::value_objects::{AttributeValue, ContextObjectType, MessageRole};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct NarratorContextObject {
    pub title: String,
    pub summary: String,
}

#[derive(Debug, Clone)]
pub struct NarratorMessage {
    pub role: MessageRole,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct NarratorRequest {
    pub system_prompt: String,
    pub world_summary: String,
    pub recent_messages: Vec<NarratorMessage>,
    pub retrieved_objects: Vec<NarratorContextObject>,
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
pub struct NarratorResponse {
    pub message: String,
    pub proposed_context_object: Option<ProposedContextObject>,
}

#[derive(Debug, Error)]
pub enum NarratorError {
    #[error("Narrator backend unavailable ({details})")]
    Unavailable { details: String },
    #[error("Narrator backend received invalid query ({details})")]
    InvalidQuery { details: String },
    #[error("Narrator backend returned invalid output ({details})")]
    InvalidResponse { details: String },
}

pub type NarratorResult<T> = Result<T, NarratorError>;

#[async_trait::async_trait]
pub trait NarratorPort: Send + Sync {
    async fn narrate(&self, request: NarratorRequest) -> NarratorResult<NarratorResponse>;
}
