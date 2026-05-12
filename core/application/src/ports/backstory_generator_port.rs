use thiserror::Error;

use super::narrator_port::{NarratorContextObject, NarratorMessage, ProposedContextObject};

/// Request sent to the character backstory backend.
#[derive(Debug, Clone)]
pub struct BackstoryGenerationRequest {
    /// Session-level summary and mode context.
    pub world_summary: String,
    /// Recent transcript messages available to the narrator.
    pub recent_messages: Vec<NarratorMessage>,
    /// Retrieved context objects shown to the narrator.
    pub retrieved_objects: Vec<NarratorContextObject>,
    /// Player action that caused the narrator turn.
    pub player_action: String,
    /// Final narrator response for the player.
    pub narrator_message: String,
    /// NPC context object created by the narrator.
    pub context_object: ProposedContextObject,
}

/// Result returned by the character backstory backend.
#[derive(Debug, Clone)]
pub struct BackstoryGenerationResponse {
    /// Backstory text suitable for storing as the object's long description.
    pub backstory: String,
}

/// Errors returned by [`BackstoryGeneratorPort`].
#[derive(Debug, Error)]
pub enum BackstoryGeneratorError {
    /// Backend could not be reached or completed the request.
    #[error("BackstoryGenerator backend unavailable ({details})")]
    Unavailable {
        /// Backend-specific error details.
        details: String,
    },
    /// Request payload is not accepted by the backend.
    #[error("BackstoryGenerator backend received invalid query ({details})")]
    InvalidQuery {
        /// Backend-specific error details.
        details: String,
    },
    /// Backend responded with malformed or unusable data.
    #[error("BackstoryGenerator backend returned invalid output ({details})")]
    InvalidResponse {
        /// Backend-specific error details.
        details: String,
    },
}

/// Convenient result alias returned by [`BackstoryGeneratorPort`].
pub type BackstoryGeneratorResult<T> = Result<T, BackstoryGeneratorError>;

/// Port that enriches NPC context objects with a backstory.
#[async_trait::async_trait]
pub trait BackstoryGeneratorPort: Send + Sync {
    /// Produces a backstory for the supplied NPC object.
    async fn generate_backstory(
        &self,
        request: BackstoryGenerationRequest,
    ) -> BackstoryGeneratorResult<BackstoryGenerationResponse>;
}
