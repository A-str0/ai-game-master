use application::ports::{
    MemoryExtractionResponse, MemoryExtractorError, MemoryExtractorPort, MemoryExtractorRequest,
    MemoryExtractorResult,
};
use autoagents::{
    core::agent::DirectAgentHandle,
    llm::backends::openrouter::OpenRouter,
    prelude::{
        AgentBuilder, AgentOutputT, DirectAgent, Error, LLMBuilder, ReActAgent,
        SlidingWindowMemory, Task,
    },
};
use autoagents_derive::{AgentHooks, agent};

use super::{
    llm_config::load_llm_config, models::MemoryExtractorOutput,
    parsing::extracted_object_into_proposed, prompts::build_extraction_task,
};

#[derive(Clone, Copy, AgentHooks, Default)]
#[agent(
    name = "MemoryExtractor",
    description = "Extracts at most one durable context object from the latest exchange.",
    output = MemoryExtractorOutput
)]
struct MemoryExtractor;

pub struct OpenRouterMemoryExtractorAdapter {
    extractor_handle: DirectAgentHandle<ReActAgent<MemoryExtractor>>,
}

impl OpenRouterMemoryExtractorAdapter {
    pub async fn new() -> Result<Self, Error> {
        let (api_key, model) = load_llm_config();
        let llm = LLMBuilder::<OpenRouter>::new()
            .api_key(api_key)
            .model(model)
            .build()?;

        let extractor = ReActAgent::new(MemoryExtractor);
        let extractor_handle = AgentBuilder::<_, DirectAgent>::new(extractor)
            .llm(llm)
            .memory(Box::new(SlidingWindowMemory::new(10)))
            .build()
            .await?;

        Ok(Self { extractor_handle })
    }
}

#[async_trait::async_trait]
impl MemoryExtractorPort for OpenRouterMemoryExtractorAdapter {
    async fn extract(
        &self,
        request: MemoryExtractorRequest,
    ) -> MemoryExtractorResult<MemoryExtractionResponse> {
        let output = self
            .extractor_handle
            .agent
            .run(Task::new(build_extraction_task(&request)))
            .await
            .map_err(|error| MemoryExtractorError::Unavailable {
                details: format!("memory extraction failed: {error}"),
            })?;

        let object = extracted_object_into_proposed(output.into())?;

        Ok(MemoryExtractionResponse {
            objects: object.into_iter().collect(),
        })
    }
}
