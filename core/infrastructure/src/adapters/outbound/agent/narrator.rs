use application::ports::{
    NarratorError, NarratorPort, NarratorRequest, NarratorResponse, NarratorResult,
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
    llm_config::load_llm_config, models::NarratorOutput, prompts::build_narrator_task,
    tools::CreateContextObjectTool,
};

#[derive(Clone, Copy, AgentHooks, Default)]
#[agent(
    name = "Narrator",
    description = "Narrates scenes and uses tools to create durable context objects when needed.",
    tools = [CreateContextObjectTool],
    output = NarratorOutput
)]
struct Narrator;

pub struct OpenRouterNarratorAdapter {
    narrator_handle: DirectAgentHandle<ReActAgent<Narrator>>,
}

impl OpenRouterNarratorAdapter {
    pub async fn new() -> Result<Self, Error> {
        let (api_key, model) = load_llm_config();
        let llm = LLMBuilder::<OpenRouter>::new()
            .api_key(api_key)
            .model(model)
            .build()?;

        let narrator = ReActAgent::new(Narrator);
        let narrator_handle = AgentBuilder::<_, DirectAgent>::new(narrator)
            .llm(llm)
            .memory(Box::new(SlidingWindowMemory::new(10)))
            .build()
            .await?;

        Ok(Self { narrator_handle })
    }
}

#[async_trait::async_trait]
impl NarratorPort for OpenRouterNarratorAdapter {
    async fn narrate(&self, request: NarratorRequest) -> NarratorResult<NarratorResponse> {
        let output = self
            .narrator_handle
            .agent
            .run(Task::new(build_narrator_task(&request)))
            .await
            .map_err(|error| NarratorError::Unavailable {
                details: format!("agent execution failed: {error}"),
            })?;

        if let Some(details) = output.tool_call_error {
            return Err(NarratorError::InvalidResponse { details });
        }

        if !output.tool_call_attempted {
            eprintln!(
                "narrator returned text without create_context_object tool call; player_action={:?}",
                request.player_action
            );
        }

        if output.message.trim().is_empty() {
            return Err(NarratorError::InvalidResponse {
                details: String::from("agent returned an empty final message"),
            });
        }

        Ok(NarratorResponse {
            message: output.message,
            proposed_context_object: output.context_object,
        })
    }
}
