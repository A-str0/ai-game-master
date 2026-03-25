use std::sync::Arc;

use application::{
    AppResult,
    ports::{MessageRepository, PromptInput},
    services::PromptAssembler,
};
use domain::aggregates::{GameSession, Message};

// TODO: move to config
const SYSTEM_PROMPT: &str = "You are the Game Master for a tabletop fantasy role-playing game. Be vivid, coherent and consistent with earlier world details. Use retrieved context objects below when relevant.";
const INSTRUCTIONS: &str = "Use the retrieved objects to answer.";

pub struct PromptAssembly {}

impl PromptAssembly {
    pub fn new() -> Self {
        Self {}
    }
}

#[allow(unused_variables)] // TODO: remove
#[async_trait::async_trait]
impl PromptAssembler for PromptAssembly {
    async fn assemble(
        &self,
        session: &GameSession,
        player_message: &Message,
    ) -> AppResult<PromptInput> {
        Ok(PromptInput {
            system_prompt: String::from(SYSTEM_PROMPT),
            world_summary: String::from("TODO"), // TODO
            retrieved_objects: Vec::new(),       // TODO
            player_action: String::from(player_message.text()), // TODO
            instructions: String::from(INSTRUCTIONS),
        })
    }
}
