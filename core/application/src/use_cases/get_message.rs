use std::sync::Arc;

use domain::{
    Identifiable,
    value_objects::{GameSessionId, MessageId, MessageRole, UserId},
};

use crate::{
    ports::{GameSessionRepository, MessageRepository, UserPort, UserPortError},
    use_cases::{UseCase, UseCaseResult},
};

/// Transport-friendly representation of [`MessageRole`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageRoleDTO {
    /// Message authored by the player.
    Player,
    /// Message authored by the game master.
    Gm,
    /// Message authored by internal system logic.
    System,
}

impl From<MessageRole> for MessageRoleDTO {
    fn from(value: MessageRole) -> Self {
        match value {
            MessageRole::Player => Self::Player,
            MessageRole::Gm => Self::Gm,
            MessageRole::System => Self::System,
        }
    }
}

/// Read model returned for one transcript message.
pub struct MessageView {
    /// Message identifier.
    pub id: MessageId,
    /// Owning session identifier.
    pub session_id: GameSessionId,
    /// Author role.
    pub role: MessageRoleDTO,
    /// Raw stored text.
    pub text: String,
    /// Creation timestamp.
    pub ts: chrono::DateTime<chrono::Utc>,
}

/// Command that fetches one message visible to the current user.
pub struct GetMessageCommand {
    /// Target message identifier.
    pub message_id: MessageId,
}

/// Response returned by [`GetMessageUseCase`].
pub type GetMessageResponse = MessageView;

/// Use case that loads one message and enforces ownership through its session.
pub struct GetMessageUseCase {
    message_repo: Arc<dyn MessageRepository>,
    session_repo: Arc<dyn GameSessionRepository>,
    current_user: Arc<dyn UserPort>,
}

impl GetMessageUseCase {
    /// Creates the use case with its required dependencies.
    pub fn new(
        message_repo: Arc<dyn MessageRepository>,
        session_repo: Arc<dyn GameSessionRepository>,
        current_user: Arc<dyn UserPort>,
    ) -> Self {
        Self {
            message_repo,
            session_repo,
            current_user,
        }
    }
}

fn ensure_owner(owner_id: UserId, current_user_id: UserId) -> UseCaseResult<()> {
    if owner_id != current_user_id {
        return Err(UserPortError::Forbidden.into());
    }

    Ok(())
}

#[async_trait::async_trait]
impl UseCase<GetMessageCommand, GetMessageResponse> for GetMessageUseCase {
    async fn execute(&self, command: GetMessageCommand) -> UseCaseResult<GetMessageResponse> {
        let current_user_id = self.current_user.current_user_id().await?;
        let message = self.message_repo.get_by_id(&command.message_id).await?;
        let session = self.session_repo.get_by_id(message.session_id()).await?;
        ensure_owner(*session.owner_id(), current_user_id)?;

        Ok(MessageView {
            id: *message.id(),
            session_id: *message.session_id(),
            role: MessageRoleDTO::from(message.role()),
            text: message.text().to_owned(),
            ts: message.ts(),
        })
    }
}
