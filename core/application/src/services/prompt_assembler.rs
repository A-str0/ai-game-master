use domain::aggregates::{GameSession, Message};
use thiserror::Error;

use crate::ports::{PromptContextObject, PromptInput};

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
        retrieved_objects: &[PromptContextObject],
    ) -> PromptAssemblerResult<PromptInput>;
}
