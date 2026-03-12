use autoagents::prelude::*;

#[agent(name = "Narrator", description = "TODO")]
#[derive(Clone, Copy, AgentHooks, Default)]
pub struct NarratorAgent;
