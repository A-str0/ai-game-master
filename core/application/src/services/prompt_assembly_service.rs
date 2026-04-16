use domain::{
    Identifiable,
    aggregates::{GameSession, Message},
};
use thiserror::Error;

use crate::ports::{NarratorContextObject, NarratorMessage, NarratorRequest};

// TODO: move to config
const SYSTEM_PROMPT: &str = "You are the Game Master for a tabletop fantasy role-playing game. Be vivid, coherent and consistent with earlier world details. Use retrieved context objects below when relevant.";
const INSTRUCTIONS: &str = "Use the retrieved objects to answer.";

#[derive(Debug, Error)]
pub enum PromptAssemblerError {
    #[error("PromptAssembler unavailable")]
    Unavailable,
    #[error("PromptAssembler received invalid input")]
    InvalidInput,
}

pub type PromptAssemblerResult<T> = Result<T, PromptAssemblerError>;

#[async_trait::async_trait]
pub trait PromptAssembler: Send + Sync {
    async fn assemble(
        &self,
        session: &GameSession,
        recent_messages: &[Message],
        player_message: &Message,
        retrieved_objects: &[NarratorContextObject],
    ) -> PromptAssemblerResult<NarratorRequest>;
}

pub struct PromptAssemblyService {}

impl PromptAssemblyService {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl PromptAssembler for PromptAssemblyService {
    async fn assemble(
        &self,
        session: &GameSession,
        recent_messages: &[Message],
        player_message: &Message,
        retrieved_objects: &[NarratorContextObject],
    ) -> PromptAssemblerResult<NarratorRequest> {
        Ok(NarratorRequest {
            system_prompt: String::from(SYSTEM_PROMPT),
            world_summary: format!(
                "Session {:?} in {:?} mode.",
                session.id(),
                session.config().session_mode()
            ),
            recent_messages: recent_messages
                .iter()
                .rev()
                .map(|message| NarratorMessage {
                    role: message.role(),
                    text: message.text().to_owned(),
                })
                .collect(),
            retrieved_objects: retrieved_objects.to_vec(),
            player_action: String::from(player_message.text()),
            instructions: String::from(INSTRUCTIONS),
        })
    }
}
