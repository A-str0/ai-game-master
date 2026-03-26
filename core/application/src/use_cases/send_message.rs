use std::sync::Arc;

use domain::{
    Identifiable,
    aggregates::Message,
    value_objects::{GameSessionId, MessageId, MessageRole},
};

use crate::{
    AppError, AppResult,
    ports::{
        AgentOrchestrator, AgentOrchestratorResponse, Clock, CurrentUser, GameSessionRepository,
        IdGenerator, MessageRepository, PromptContextObject,
    },
    services::{PromptAssembler, RetrivialService},
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
    current_user: Arc<dyn CurrentUser>,
    prompt_assembly: Arc<dyn PromptAssembler>,
    retrivial: Arc<dyn RetrivialService>,
    agent: Arc<dyn AgentOrchestrator>,
    clock: Arc<dyn Clock>,
    id_generator: Arc<dyn IdGenerator>,
}

impl SendMessageUseCase {
    pub fn new(
        session_repo: Arc<dyn GameSessionRepository>,
        message_repo: Arc<dyn MessageRepository>,
        current_user: Arc<dyn CurrentUser>,
        prompt_assembly: Arc<dyn PromptAssembler>,
        retrivial: Arc<dyn RetrivialService>,
        agent: Arc<dyn AgentOrchestrator>,
        clock: Arc<dyn Clock>,
        id_generator: Arc<dyn IdGenerator>,
    ) -> Self {
        Self {
            session_repo,
            message_repo,
            current_user,
            prompt_assembly,
            retrivial,
            agent,
            clock,
            id_generator,
        }
    }
}

#[async_trait::async_trait]
impl UseCase<SendMessageCommand, SendMessageResponse> for SendMessageUseCase {
    async fn execute(&self, command: SendMessageCommand) -> AppResult<SendMessageResponse> {
        let current_user_id = self.current_user.current_user_id().await?;

        let session = self.session_repo.get_by_id(&command.session_id).await?;

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

        self.message_repo.create(&message).await?;

        let retrival_objects = self.retrivial.find_for_message(&session, &message).await?;
        let prompt_objects = retrival_objects
            .into_iter()
            .map(|obj| PromptContextObject {
                title: String::from(obj.context_object.title()),
                summary: String::from(obj.context_object.short_desc()),
            })
            .collect::<Vec<_>>();

        let prompt = self
            .prompt_assembly
            .assemble(&session, &message, &prompt_objects)
            .await?;
        let agent_response = self.agent.generate(&prompt).await?;

        match agent_response {
            AgentOrchestratorResponse::Text(msg) => {
                let gm_message = Message::new(
                    self.id_generator.next_message_id().await,
                    command.session_id,
                    MessageRole::Gm,
                    &msg,
                    self.clock.now().await,
                )
                .map_err(AppError::from)?;

                self.message_repo.create(&gm_message).await?;
            }
            AgentOrchestratorResponse::CreateContextObject => {}
        }

        Ok(SendMessageResponse {
            player_message_id: *message.id(),
        })
    }
}
