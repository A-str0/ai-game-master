use std::sync::Arc;

use domain::value_objects::ContextObjectType;
use thiserror::Error;

use crate::ports::{
    BackstoryGenerationRequest, BackstoryGeneratorError, BackstoryGeneratorPort,
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
    /// NPC backstory generation failed.
    #[error(transparent)]
    BackstoryGenerator(#[from] BackstoryGeneratorError),
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
    backstory_generator: Arc<dyn BackstoryGeneratorPort>,
}

impl AgentOrchestrationService {
    /// Creates the orchestration service with concrete narrator backends.
    pub fn new(
        narrator: Arc<dyn NarratorPort>,
        memory_extractor: Arc<dyn MemoryExtractorPort>,
        backstory_generator: Arc<dyn BackstoryGeneratorPort>,
    ) -> Self {
        Self {
            narrator,
            memory_extractor,
            backstory_generator,
        }
    }

    async fn enrich_narrator_context_object(
        &self,
        request: &NarratorRequest,
        narrator_message: &str,
        mut object: ProposedContextObject,
    ) -> AgentOrchestrationServiceResult<ProposedContextObject> {
        if object.object_type != ContextObjectType::Npc {
            return Ok(object);
        }

        let response = self
            .backstory_generator
            .generate_backstory(BackstoryGenerationRequest {
                world_summary: request.world_summary.clone(),
                recent_messages: request.recent_messages.clone(),
                retrieved_objects: request.retrieved_objects.clone(),
                player_action: request.player_action.clone(),
                narrator_message: narrator_message.to_owned(),
                context_object: object.clone(),
            })
            .await?;

        object.long_desc = Some(response.backstory);
        Ok(object)
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
            let object = self
                .enrich_narrator_context_object(&request, &narration.message, object)
                .await?;

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
