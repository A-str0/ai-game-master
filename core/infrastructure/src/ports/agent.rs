use application::{
    AppError, AppResult,
    ports::{AgentPort, AgentPrompt, AgentResponse},
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
        let llm = LLMBuilder::new()
            .api_key("API_KEY") // TODO: change to config
            .model("MODEL") // TODO: change to config
            .build()?;

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
impl AgentPort for Agent {
    async fn generate(&self, prompt: AgentPrompt) -> AppResult<AgentResponse> {
        // TODO: implement error handling

        if let Some(value) = self
            .handle
            .agent
            .run(Task::new(format!("{:?}", prompt)))
            .await
            .ok()
        {
            return Ok(AgentResponse(value));
        }

        Err(AppError::Unavailable)
    }
}
