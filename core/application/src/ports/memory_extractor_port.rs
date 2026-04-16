use thiserror::Error;

use super::narrator_port::{NarratorContextObject, NarratorMessage, ProposedContextObject};

#[derive(Debug, Clone)]
pub struct MemoryExtractorRequest {
    pub recent_messages: Vec<NarratorMessage>,
    pub retrieved_objects: Vec<NarratorContextObject>,
    pub player_action: String,
    pub narrator_message: String,
}

#[derive(Debug, Clone, Default)]
pub struct MemoryExtractionResponse {
    pub objects: Vec<ProposedContextObject>,
}

#[derive(Debug, Error)]
pub enum MemoryExtractorError {
    #[error("MemoryExtractor backend unavailable ({details})")]
    Unavailable { details: String },
    #[error("MemoryExtractor backend received invalid query ({details})")]
    InvalidQuery { details: String },
    #[error("MemoryExtractor backend returned invalid output ({details})")]
    InvalidResponse { details: String },
}

pub type MemoryExtractorResult<T> = Result<T, MemoryExtractorError>;

#[async_trait::async_trait]
pub trait MemoryExtractorPort: Send + Sync {
    async fn extract(
        &self,
        request: MemoryExtractorRequest,
    ) -> MemoryExtractorResult<MemoryExtractionResponse>;
}
