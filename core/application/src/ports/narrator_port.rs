use std::collections::HashMap;

use domain::value_objects::{AttributeValue, ContextObjectType, MessageRole};
use thiserror::Error;

/// Compact representation of a retrieved context object sent to the narrator.
#[derive(Debug, Clone)]
pub struct NarratorContextObject {
    /// Display title of the object.
    pub title: String,
    /// Short summary that fits into prompt context.
    pub summary: String,
}

/// One message supplied to the narrator prompt.
#[derive(Debug, Clone)]
pub struct NarratorMessage {
    /// Role that authored the message.
    pub role: MessageRole,
    /// Raw message text.
    pub text: String,
}

/// Full prompt payload supplied to the narrator backend.
#[derive(Debug, Clone)]
pub struct NarratorRequest {
    /// System prompt that establishes the narrator's role.
    pub system_prompt: String,
    /// Session-level summary and mode context.
    pub world_summary: String,
    /// Recent transcript messages.
    pub recent_messages: Vec<NarratorMessage>,
    /// Retrieved context objects selected for the current turn.
    pub retrieved_objects: Vec<NarratorContextObject>,
    /// Player action that must be answered.
    pub player_action: String,
    /// Additional instructions appended by the prompt assembly service.
    pub instructions: String,
}

/// Candidate durable object proposed by the narrator or memory extractor.
#[derive(Debug, Clone)]
pub struct ProposedContextObject {
    /// Type assigned to the extracted object.
    pub object_type: ContextObjectType,
    /// Human-readable object title.
    pub title: String,
    /// Compact summary used for retrieval and prompting.
    pub short_desc: String,
    /// Longer description when the agent supplied one.
    pub long_desc: Option<String>,
    /// Structured attributes inferred from the exchange.
    pub attributes: HashMap<String, AttributeValue>,
    /// Relative importance score used during retrieval.
    pub importance_score: f32,
}

/// Result returned by the narrator backend.
#[derive(Debug, Clone)]
pub struct NarratorResponse {
    /// Final narrated GM message.
    pub message: String,
    /// Optional durable object produced directly by the narrator.
    pub proposed_context_object: Option<ProposedContextObject>,
}

/// Errors returned by [`NarratorPort`].
#[derive(Debug, Error)]
pub enum NarratorError {
    /// Backend could not be reached or completed the request.
    #[error("Narrator backend unavailable ({details})")]
    Unavailable {
        /// Backend-specific error details.
        details: String,
    },
    /// Request payload is not accepted by the backend.
    #[error("Narrator backend received invalid query ({details})")]
    InvalidQuery {
        /// Backend-specific error details.
        details: String,
    },
    /// Backend responded with malformed or unusable data.
    #[error("Narrator backend returned invalid output ({details})")]
    InvalidResponse {
        /// Backend-specific error details.
        details: String,
    },
}

/// Convenient result alias returned by [`NarratorPort`].
pub type NarratorResult<T> = Result<T, NarratorError>;

/// Port that generates the GM response for a turn.
#[async_trait::async_trait]
pub trait NarratorPort: Send + Sync {
    /// Produces the narrator output for the supplied turn context.
    async fn narrate(&self, request: NarratorRequest) -> NarratorResult<NarratorResponse>;
}
