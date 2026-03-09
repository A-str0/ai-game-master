use std::sync::Arc;

use domain::Identifiable;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    AppError, AppResult,
    ports::{GameSessionRepository, MessageRepository, PortError, RepoError, UserAccessPort},
    service::PromptAssemblyService,
    use_cases::UseCase,
};

#[derive(Serialize, Deserialize)]
pub struct SendMessageCommand {
    session_id: Uuid,
    owner_id: Uuid,
    text: String, // TODO: maybe &str???
}

#[derive(Serialize, Deserialize)]
pub struct SendMessageResponse {}

pub struct SendMessageUseCase {
    session_repo: Arc<dyn GameSessionRepository>,
    message_repo: Arc<dyn MessageRepository>,
    user_access: Arc<dyn UserAccessPort>,
    prompt_assembly: Arc<dyn PromptAssemblyService>,
}

impl SendMessageUseCase {
    pub fn new(
        session_repo: Arc<dyn GameSessionRepository>,
        message_repo: Arc<dyn MessageRepository>,
        user_access: Arc<dyn UserAccessPort>,
        prompt_assembly: Arc<dyn PromptAssemblyService>,
    ) -> Self {
        Self {
            session_repo,
            message_repo,
            user_access,
            prompt_assembly,
        }
    }
}

#[async_trait::async_trait]
impl UseCase<SendMessageCommand, SendMessageResponse> for SendMessageUseCase {
    async fn execute(&self, command: SendMessageCommand) -> AppResult<SendMessageResponse> {
        let current_user = self.user_access.get_user().await.map_err(|err| match err {
            PortError::NotFound => AppError::NotFound(command.owner_id.into()),
            PortError::Forbidden => AppError::Forbidden,
            PortError::Unavailable => AppError::Unavailable,
        })?;

        let session = self
            .session_repo
            .get_by_id(&command.session_id.into())
            .await
            .map_err(|err| match err {
                RepoError::NotFound => AppError::NotFound(command.session_id.into()),
                RepoError::Conflict => AppError::Conflict,
                RepoError::Unavailable => AppError::Unavailable,
            })?;

        // TODO: change validation
        if session.owner_id() != current_user.id() {
            return Err(AppError::Forbidden);
        }

        Ok(SendMessageResponse {})
    }
}
