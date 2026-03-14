use std::sync::Arc;

use domain::{
    Identifiable,
    aggregates::Message,
    value_objects::{GameSessionId, MessageId, MessageRole},
};

use crate::{
    AppError, AppResult,
    ports::{
        AgentPort, Clock, CurrentUserError, CurrentUserPort, GameSessionRepository, IdGenerator,
        MessageRepository, RepoError,
    },
    services::PromptAssemblyService,
    use_cases::UseCase,
};

pub struct SendMessageCommand {
    pub session_id: GameSessionId,
    pub text: String,
}

pub struct SendMessageResponse {
    pub player_message_id: MessageId,
}

pub struct SendMessageUseCase {
    session_repo: Arc<dyn GameSessionRepository>,
    message_repo: Arc<dyn MessageRepository>,
    current_user: Arc<dyn CurrentUserPort>,
    prompt_assembly: Arc<dyn PromptAssemblyService>,
    agent: Arc<dyn AgentPort>,
    clock: Arc<dyn Clock>,
    id_generator: Arc<dyn IdGenerator>,
}

impl SendMessageUseCase {
    pub fn new(
        session_repo: Arc<dyn GameSessionRepository>,
        message_repo: Arc<dyn MessageRepository>,
        current_user: Arc<dyn CurrentUserPort>,
        prompt_assembly: Arc<dyn PromptAssemblyService>,
        agent: Arc<dyn AgentPort>,
        clock: Arc<dyn Clock>,
        id_generator: Arc<dyn IdGenerator>,
    ) -> Self {
        Self {
            session_repo,
            message_repo,
            current_user,
            prompt_assembly,
            agent,
            clock,
            id_generator,
        }
    }
}

#[async_trait::async_trait]
impl UseCase<SendMessageCommand, SendMessageResponse> for SendMessageUseCase {
    async fn execute(&self, command: SendMessageCommand) -> AppResult<SendMessageResponse> {
        let current_user_id =
            self.current_user
                .current_user_id()
                .await
                .map_err(|err| match err {
                    CurrentUserError::Unauthenticated => AppError::Unauthenticated,
                    CurrentUserError::Forbidden => AppError::Forbidden,
                    CurrentUserError::Unavailable => AppError::Unavailable,
                })?;

        let session = self
            .session_repo
            .get_by_id(&command.session_id)
            .await
            .map_err(|err| match err {
                RepoError::NotFound => AppError::NotFound(command.session_id.0),
                RepoError::Conflict => AppError::Conflict,
                RepoError::Unavailable => AppError::Unavailable,
            })?;

        if session.owner_id() != &current_user_id {
            return Err(AppError::Forbidden);
        }

        let message = Message::new(
            self.id_generator.next_message_id().await,
            command.session_id,
            MessageRole::Player,
            &command.text,
            self.clock.now().await,
        )
        .map_err(AppError::from)?;

        self.message_repo
            .create(&message)
            .await
            .map_err(|err| match err {
                RepoError::NotFound => AppError::NotFound(command.session_id.0),
                RepoError::Conflict => AppError::Conflict,
                RepoError::Unavailable => AppError::Unavailable,
            })?;

        let prompt = self.prompt_assembly.assemble(&session, &message).await?;

        let agent_response = self.agent.generate(prompt).await?;

        let gm_message = Message::new(
            self.id_generator.next_message_id().await,
            command.session_id,
            MessageRole::Gm,
            &agent_response.0,
            self.clock.now().await,
        )
        .map_err(AppError::from)?;

        self.message_repo
            .create(&gm_message)
            .await
            .map_err(|err| match err {
                RepoError::NotFound => AppError::NotFound(command.session_id.0),
                RepoError::Conflict => AppError::Conflict,
                RepoError::Unavailable => AppError::Unavailable,
            })?;

        Ok(SendMessageResponse {
            player_message_id: *message.id(),
        })
    }
}
