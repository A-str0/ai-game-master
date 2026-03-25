use application::ports::{
    AgentOrchestrator, AgentOrchestratorError, AgentOrchestratorResponse, AgentResult, PromptInput,
};
use autoagents::{core::agent::DirectAgentHandle, prelude::*};
use autoagents_derive::{AgentHooks, agent};

#[derive(Clone, Copy, AgentHooks, Default)]
#[agent(name = "TEST", description = "TEST")]
pub struct Narrator;

pub struct Agent {
    handle: DirectAgentHandle<ReActAgent<Narrator>>, // TODO: change to switchable backend
}

impl Agent {
    pub async fn new() -> Result<Self, Error> {
        let api_key = std::env::var("LLM_API_KEY").unwrap_or_else(|_| String::from("API_KEY"));
        let model = std::env::var("LLM_MODEL")
            .unwrap_or_else(|_| String::from("nvidia/nemotron-3-super-120b-a12b:free"));

        let llm = LLMBuilder::new().api_key(api_key).model(model).build()?;

        let agent = ReActAgent::new(Narrator);
        let handle = AgentBuilder::<_, DirectAgent>::new(agent)
            .llm(llm)
            .memory(Box::new(SlidingWindowMemory::new(10)))
            .build()
            .await?;

        Ok(Self { handle })
    }
}

#[async_trait::async_trait]
impl AgentOrchestrator for Agent {
    async fn generate(&self, prompt: PromptInput) -> AgentResult<AgentOrchestratorResponse> {
        // TODO: implement error handling

        if let Some(value) = self
            .handle
            .agent
            .run(Task::new(format!("{:?}", prompt)))
            .await
            .ok()
        {
            return Ok(AgentOrchestratorResponse(value));
        }

        Err(AgentOrchestratorError::Unavailable)
    }
}
