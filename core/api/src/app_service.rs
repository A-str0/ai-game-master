use std::sync::Arc;

use application::{
    ports::{
        ContextObjectRepository, Embedder, GameSessionRepository, MessageRepository,
        UnitOfWorkFactory, UserPort, VectorSearcher,
    },
    services::{
        AgentOrchestrationService, Clock, IdGenerator, PromptAssembler, RetrivialServicePort,
    },
    use_cases::{
        CreateSessionCommand, CreateSessionResponse, CreateSessionUseCase, GetMessageCommand,
        GetMessageResponse, GetMessageUseCase, GetMessagesCommand, GetMessagesResponse,
        GetMessagesUseCase, GetSessionCommand, GetSessionResponse, GetSessionUseCase,
        SendMessageCommand, SendMessageResponse, SendMessageUseCase, UseCase, UseCaseError,
    },
};
use async_trait::async_trait;
use domain::value_objects::{GameSessionId, UserId};
use infrastructure::adapters::{CurrentUserContext, RequestCurrentUser};

/// Application-facing service used by HTTP handlers.
#[async_trait]
pub trait ApiApplicationService: Send + Sync {
    /// Creates a new session for the supplied user.
    async fn create_session(&self, user_id: UserId) -> Result<CreateSessionResponse, UseCaseError>;

    /// Returns one session visible to the supplied user.
    async fn get_session(
        &self,
        user_id: UserId,
        session_id: GameSessionId,
    ) -> Result<GetSessionResponse, UseCaseError>;

    /// Returns one message visible to the supplied user.
    async fn get_message(
        &self,
        user_id: UserId,
        message_id: domain::value_objects::MessageId,
    ) -> Result<GetMessageResponse, UseCaseError>;

    /// Returns messages for one visible session.
    async fn get_messages(
        &self,
        user_id: UserId,
        command: GetMessagesCommand,
    ) -> Result<GetMessagesResponse, UseCaseError>;

    /// Processes a player message on behalf of the supplied user.
    async fn send_message(
        &self,
        user_id: UserId,
        command: SendMessageCommand,
    ) -> Result<SendMessageResponse, UseCaseError>;
}

/// Default API application service that constructs use cases per request.
#[derive(Clone)]
pub struct LiveApiApplicationService {
    sessions_repo: Arc<dyn GameSessionRepository>,
    messages_repo: Arc<dyn MessageRepository>,
    context_object_repo: Arc<dyn ContextObjectRepository>,
    unit_of_work: Arc<dyn UnitOfWorkFactory>,
    retrivial: Arc<dyn RetrivialServicePort>,
    agent_orchestration: Arc<AgentOrchestrationService>,
    embedder: Arc<dyn Embedder>,
    vector_searcher: Arc<dyn VectorSearcher>,
    clock: Arc<dyn Clock>,
    id_generator: Arc<dyn IdGenerator>,
    prompt_assembly: Arc<dyn PromptAssembler>,
}

impl LiveApiApplicationService {
    #[allow(clippy::too_many_arguments)]
    /// Creates the live application service from concrete repositories and services.
    pub fn new(
        sessions_repo: Arc<dyn GameSessionRepository>,
        messages_repo: Arc<dyn MessageRepository>,
        context_object_repo: Arc<dyn ContextObjectRepository>,
        unit_of_work: Arc<dyn UnitOfWorkFactory>,
        retrivial: Arc<dyn RetrivialServicePort>,
        agent_orchestration: Arc<AgentOrchestrationService>,
        embedder: Arc<dyn Embedder>,
        vector_searcher: Arc<dyn VectorSearcher>,
        clock: Arc<dyn Clock>,
        id_generator: Arc<dyn IdGenerator>,
        prompt_assembly: Arc<dyn PromptAssembler>,
    ) -> Self {
        Self {
            sessions_repo,
            messages_repo,
            context_object_repo,
            unit_of_work,
            retrivial,
            agent_orchestration,
            embedder,
            vector_searcher,
            clock,
            id_generator,
            prompt_assembly,
        }
    }

    fn current_user_port(&self, user_id: UserId) -> Arc<dyn UserPort> {
        Arc::new(RequestCurrentUser::new(CurrentUserContext::authenticated(
            user_id,
        )))
    }
}

#[async_trait]
impl ApiApplicationService for LiveApiApplicationService {
    async fn create_session(&self, user_id: UserId) -> Result<CreateSessionResponse, UseCaseError> {
        let use_case = CreateSessionUseCase::new(
            Arc::clone(&self.unit_of_work),
            self.current_user_port(user_id),
            Arc::clone(&self.clock),
            Arc::clone(&self.id_generator),
            Arc::clone(&self.vector_searcher),
        );

        use_case.execute(CreateSessionCommand).await
    }

    async fn get_session(
        &self,
        user_id: UserId,
        session_id: GameSessionId,
    ) -> Result<GetSessionResponse, UseCaseError> {
        let use_case = GetSessionUseCase::new(
            Arc::clone(&self.sessions_repo),
            self.current_user_port(user_id),
        );

        use_case.execute(GetSessionCommand { session_id }).await
    }

    async fn send_message(
        &self,
        user_id: UserId,
        command: SendMessageCommand,
    ) -> Result<SendMessageResponse, UseCaseError> {
        let use_case = SendMessageUseCase::new(
            Arc::clone(&self.sessions_repo),
            Arc::clone(&self.messages_repo),
            Arc::clone(&self.context_object_repo),
            Arc::clone(&self.unit_of_work),
            self.current_user_port(user_id),
            Arc::clone(&self.prompt_assembly),
            Arc::clone(&self.retrivial),
            Arc::clone(&self.agent_orchestration),
            Arc::clone(&self.embedder),
            Arc::clone(&self.vector_searcher),
            Arc::clone(&self.clock),
            Arc::clone(&self.id_generator),
        );

        use_case.execute(command).await
    }

    async fn get_message(
        &self,
        user_id: UserId,
        message_id: domain::value_objects::MessageId,
    ) -> Result<GetMessageResponse, UseCaseError> {
        let use_case = GetMessageUseCase::new(
            Arc::clone(&self.messages_repo),
            Arc::clone(&self.sessions_repo),
            self.current_user_port(user_id),
        );

        use_case.execute(GetMessageCommand { message_id }).await
    }

    async fn get_messages(
        &self,
        user_id: UserId,
        command: GetMessagesCommand,
    ) -> Result<GetMessagesResponse, UseCaseError> {
        let use_case = GetMessagesUseCase::new(
            Arc::clone(&self.messages_repo),
            Arc::clone(&self.sessions_repo),
            self.current_user_port(user_id),
        );

        use_case.execute(command).await
    }
}
