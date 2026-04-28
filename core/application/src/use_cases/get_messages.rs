use std::sync::Arc;

use domain::{Identifiable, value_objects::GameSessionId};

use crate::{
    ports::{GameSessionRepository, MessageRepository, UserPort, UserPortError},
    use_cases::{MessageView, UseCase, UseCaseResult},
};

/// Command that fetches transcript messages for one session.
pub struct GetMessagesCommand {
    /// Session whose transcript should be listed.
    pub session_id: GameSessionId,
    /// Maximum number of messages to return.
    pub limit: usize,
}

/// Response returned by [`GetMessagesUseCase`].
pub struct GetMessagesResponse {
    /// Messages ordered from newest to oldest.
    pub messages: Vec<MessageView>,
}

/// Use case that loads messages for one session and enforces ownership.
pub struct GetMessagesUseCase {
    message_repo: Arc<dyn MessageRepository>,
    session_repo: Arc<dyn GameSessionRepository>,
    current_user: Arc<dyn UserPort>,
}

impl GetMessagesUseCase {
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

#[async_trait::async_trait]
impl UseCase<GetMessagesCommand, GetMessagesResponse> for GetMessagesUseCase {
    async fn execute(&self, command: GetMessagesCommand) -> UseCaseResult<GetMessagesResponse> {
        let current_user_id = self.current_user.current_user_id().await?;
        let session = self.session_repo.get_by_id(&command.session_id).await?;

        if session.owner_id() != &current_user_id {
            return Err(UserPortError::Forbidden.into());
        }

        let messages = self
            .message_repo
            .list_by_session(&command.session_id, command.limit)
            .await?
            .into_iter()
            .map(|message| MessageView {
                id: *message.id(),
                session_id: *message.session_id(),
                role: message.role().into(),
                text: message.text().to_owned(),
                ts: message.ts(),
            })
            .collect();

        Ok(GetMessagesResponse { messages })
    }
}
