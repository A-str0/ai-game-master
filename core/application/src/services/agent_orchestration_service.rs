use std::sync::Arc;

use thiserror::Error;

use crate::ports::{
    MemoryExtractorError, MemoryExtractorPort, MemoryExtractorRequest, NarratorError, NarratorPort,
    NarratorRequest, ProposedContextObject,
};

#[derive(Debug, Clone)]
pub struct AgentOrchestrationResponse {
    pub message: String,
    pub objects: Vec<ProposedContextObject>,
}

#[derive(Debug, Error)]
pub enum AgentOrchestrationServiceError {
    #[error(transparent)]
    Narrator(#[from] NarratorError),
    #[error(transparent)]
    MemoryExtractor(#[from] MemoryExtractorError),
    #[error("AgentOrchestrationService returned invalid data ({details})")]
    InvalidData { details: String },
}

pub type AgentOrchestrationServiceResult<T> = Result<T, AgentOrchestrationServiceError>;

pub struct AgentOrchestrationService {
    narrator: Arc<dyn NarratorPort>,
    memory_extractor: Arc<dyn MemoryExtractorPort>,
}

impl AgentOrchestrationService {
    pub fn new(
        narrator: Arc<dyn NarratorPort>,
        memory_extractor: Arc<dyn MemoryExtractorPort>,
    ) -> Self {
        Self {
            narrator,
            memory_extractor,
        }
    }

    pub async fn generate(
        &self,
        request: NarratorRequest,
    ) -> AgentOrchestrationServiceResult<AgentOrchestrationResponse> {
        let narration = self.narrator.narrate(request.clone()).await?;

        if narration.message.trim().is_empty() {
            return Err(AgentOrchestrationServiceError::InvalidData {
                details: String::from("narrator returned an empty final message"),
            });
        }

        if let Some(object) = narration.proposed_context_object {
            return Ok(AgentOrchestrationResponse {
                message: narration.message,
                objects: vec![object],
            });
        }

        let extraction = self
            .memory_extractor
            .extract(MemoryExtractorRequest {
                recent_messages: request.recent_messages,
                retrieved_objects: request.retrieved_objects,
                player_action: request.player_action,
                narrator_message: narration.message.clone(),
            })
            .await?;

        Ok(AgentOrchestrationResponse {
            message: narration.message,
            objects: extraction.objects,
        })
    }
}
