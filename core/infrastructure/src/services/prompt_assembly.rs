use application::{
    AppResult,
    ports::{PromptContextObject, PromptInput, PromptMessage},
    services::PromptAssembler,
};
use domain::{
    Identifiable,
    aggregates::{GameSession, Message},
};

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
        recent_messages: &[Message],
        player_message: &Message,
        retrieved_objects: &[PromptContextObject],
    ) -> AppResult<PromptInput> {
        Ok(PromptInput {
            system_prompt: String::from(SYSTEM_PROMPT),
            world_summary: format!(
                "Session {:?} in {:?} mode.",
                session.id(),
                session.config().session_mode()
            ),
            recent_messages: recent_messages
                .iter()
                .rev()
                .map(|message| PromptMessage {
                    role: format!("{:?}", message.role()).to_lowercase(),
                    text: message.text().to_owned(),
                })
                .collect(),
            retrieved_objects: retrieved_objects.to_vec(),
            player_action: String::from(player_message.text()),
            instructions: String::from(INSTRUCTIONS),
        })
    }
}
