use std::sync::Arc;

use domain::{Identifiable, value_objects::GameSessionId};

use crate::{
    AppError, AppResult,
    ports::{CurrentUser, GameSessionRepository, ProposedContextObject},
    services::WorldMemoryManager,
    use_cases::UseCase,
};

pub struct CreateContextObjectCommand {
    pub session_id: GameSessionId,
    pub object: ProposedContextObject,
}

pub struct CreateContextObjectResponse {
    pub context_object_id: domain::value_objects::ContextObjectId,
}

pub struct CreateContextObjectUseCase {
    session_repo: Arc<dyn GameSessionRepository>,
    current_user: Arc<dyn CurrentUser>,
    world_memory: Arc<dyn WorldMemoryManager>,
}

impl CreateContextObjectUseCase {
    pub fn new(
        session_repo: Arc<dyn GameSessionRepository>,
        current_user: Arc<dyn CurrentUser>,
        world_memory: Arc<dyn WorldMemoryManager>,
    ) -> Self {
        Self {
            session_repo,
            current_user,
            world_memory,
        }
    }
}

#[async_trait::async_trait]
impl UseCase<CreateContextObjectCommand, CreateContextObjectResponse>
    for CreateContextObjectUseCase
{
    async fn execute(
        &self,
        command: CreateContextObjectCommand,
    ) -> AppResult<CreateContextObjectResponse> {
        let current_user_id = self.current_user.current_user_id().await?;
        let session = self.session_repo.get_by_id(&command.session_id).await?;

        if session.owner_id() != &current_user_id {
            return Err(AppError::Forbidden);
        }

        let context_object = self
            .world_memory
            .create_context_object(&session, command.object)
            .await?;

        Ok(CreateContextObjectResponse {
            context_object_id: *context_object.id(),
        })
    }
}
