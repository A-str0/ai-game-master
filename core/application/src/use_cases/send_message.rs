use std::sync::Arc;

use domain::{
    Identifiable,
    aggregates::Message,
    value_objects::{GameSessionId, MessageId, MessageRole, UserId},
};

use crate::{
    AppError, AppResult,
    ports::{
        Clock, GameSessionRepository, IdGenerator, MessageRepository, PortError, RepoError,
        UserAccessPort,
    },
    service::PromptAssemblyService,
    use_cases::UseCase,
};

pub struct SendMessageCommand {
    pub session_id: GameSessionId,
    pub owner_id: UserId,
    pub text: String,
}

pub struct SendMessageResponse {
    pub player_message_id: MessageId,
    pub text: String,
}

pub struct SendMessageUseCase {
    session_repo: Arc<dyn GameSessionRepository>,
    message_repo: Arc<dyn MessageRepository>,
    user_access: Arc<dyn UserAccessPort>,
    prompt_assembly: Arc<dyn PromptAssemblyService>,
    clock: Arc<dyn Clock>,
    id_generator: Arc<dyn IdGenerator>,
}

impl SendMessageUseCase {
    pub fn new(
        session_repo: Arc<dyn GameSessionRepository>,
        message_repo: Arc<dyn MessageRepository>,
        user_access: Arc<dyn UserAccessPort>,
        prompt_assembly: Arc<dyn PromptAssemblyService>,
        clock: Arc<dyn Clock>,
        id_generator: Arc<dyn IdGenerator>,
    ) -> Self {
        Self {
            session_repo,
            message_repo,
            user_access,
            prompt_assembly,
            clock,
            id_generator,
        }
    }
}

#[async_trait::async_trait]
impl UseCase<SendMessageCommand, SendMessageResponse> for SendMessageUseCase {
    async fn execute(&self, command: SendMessageCommand) -> AppResult<SendMessageResponse> {
        let current_user = self.user_access.get_user().await.map_err(|err| match err {
            PortError::NotFound => AppError::NotFound(command.owner_id.0),
            PortError::Forbidden => AppError::Forbidden,
            PortError::Unavailable => AppError::Unavailable,
        })?;

        if current_user.id() != &command.owner_id {
            return Err(AppError::Forbidden);
        }

        let session = self
            .session_repo
            .get_by_id(&command.session_id)
            .await
            .map_err(|err| match err {
                RepoError::NotFound => AppError::NotFound(command.session_id.0),
                RepoError::Conflict => AppError::Conflict,
                RepoError::Unavailable => AppError::Unavailable,
            })?;

        if session.owner_id() != current_user.id() {
            return Err(AppError::Forbidden);
        }

        let message = self
            .build_player_message(&command)
            .await?;

        self.message_repo.create(&message).await.map_err(|err| match err {
            RepoError::NotFound => AppError::NotFound(command.session_id.0),
            RepoError::Conflict => AppError::Conflict,
            RepoError::Unavailable => AppError::Unavailable,
        })?;

        let llm_response = self
            .prompt_assembly
            .assemble(&command.session_id, &message)
            .await?;

        Ok(SendMessageResponse {
            player_message_id: *message.id(),
            text: llm_response,
        })
    }
}

impl SendMessageUseCase {
    async fn build_player_message(&self, command: &SendMessageCommand) -> AppResult<Message> {
        Message::new(
            self.id_generator.next_message_id().await,
            command.session_id,
            MessageRole::Player,
            &command.text,
            self.clock.now().await,
            None,
        )
        .map_err(AppError::from)
    }
}
