use std::sync::Arc;

use thiserror::Error;

use crate::ports::{
    MemoryExtractorError, MemoryExtractorPort, MemoryExtractorRequest, NarratorError, NarratorPort,
    NarratorRequest, ProposedContextObject,
};

/// Combined result of narration and memory extraction.
#[derive(Debug, Clone)]
pub struct AgentOrchestrationResponse {
    /// Final GM message returned to the player.
    pub message: String,
    /// Durable objects proposed for persistence.
    pub objects: Vec<ProposedContextObject>,
}

/// Errors returned by [`AgentOrchestrationService`].
#[derive(Debug, Error)]
pub enum AgentOrchestrationServiceError {
    /// Narration step failed.
    #[error(transparent)]
    Narrator(#[from] NarratorError),
    /// Memory extraction step failed.
    #[error(transparent)]
    MemoryExtractor(#[from] MemoryExtractorError),
    /// Upstream agent returned structurally invalid data.
    #[error("AgentOrchestrationService returned invalid data ({details})")]
    InvalidData {
        /// Validation details explaining why the payload was rejected.
        details: String,
    },
}

/// Convenient result alias returned by [`AgentOrchestrationService`].
pub type AgentOrchestrationServiceResult<T> = Result<T, AgentOrchestrationServiceError>;

/// Application service that coordinates narration and follow-up extraction.
pub struct AgentOrchestrationService {
    narrator: Arc<dyn NarratorPort>,
    memory_extractor: Arc<dyn MemoryExtractorPort>,
}

impl AgentOrchestrationService {
    /// Creates the orchestration service with concrete narrator backends.
    pub fn new(
        narrator: Arc<dyn NarratorPort>,
        memory_extractor: Arc<dyn MemoryExtractorPort>,
    ) -> Self {
        Self {
            narrator,
            memory_extractor,
        }
    }

    /// Generates the final narrated response and any extracted context objects.
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
