use thiserror::Error;

use super::narrator_port::{NarratorContextObject, NarratorMessage, ProposedContextObject};

/// Request sent to the memory extraction backend.
#[derive(Debug, Clone)]
pub struct MemoryExtractorRequest {
    /// Recent chat history relevant to the current turn.
    pub recent_messages: Vec<NarratorMessage>,
    /// Retrieved context objects shown to the narrator.
    pub retrieved_objects: Vec<NarratorContextObject>,
    /// The latest player action.
    pub player_action: String,
    /// The narrator response that should be mined for durable memory.
    pub narrator_message: String,
}

/// Result returned by the memory extraction backend.
#[derive(Debug, Clone, Default)]
pub struct MemoryExtractionResponse {
    /// Context objects proposed for persistence.
    pub objects: Vec<ProposedContextObject>,
}

/// Errors returned by [`MemoryExtractorPort`].
#[derive(Debug, Error)]
pub enum MemoryExtractorError {
    /// Backend could not be reached or completed the request.
    #[error("MemoryExtractor backend unavailable ({details})")]
    Unavailable {
        /// Backend-specific error details.
        details: String,
    },
    /// Request payload is not accepted by the backend.
    #[error("MemoryExtractor backend received invalid query ({details})")]
    InvalidQuery {
        /// Backend-specific error details.
        details: String,
    },
    /// Backend responded with malformed or unusable data.
    #[error("MemoryExtractor backend returned invalid output ({details})")]
    InvalidResponse {
        /// Backend-specific error details.
        details: String,
    },
}

/// Convenient result alias returned by [`MemoryExtractorPort`].
pub type MemoryExtractorResult<T> = Result<T, MemoryExtractorError>;

/// Port that extracts durable world state from the latest exchange.
#[async_trait::async_trait]
pub trait MemoryExtractorPort: Send + Sync {
    /// Extracts context objects that should be stored after a narrated turn.
    async fn extract(
        &self,
        request: MemoryExtractorRequest,
    ) -> MemoryExtractorResult<MemoryExtractionResponse>;
}
