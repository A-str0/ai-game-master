use std::sync::Arc;

use domain::{
    Identifiable,
    aggregates::{ContextObject, GameSession, Message},
    value_objects::{GameSessionId, MessageId, MessageRole, Provenance},
};

use crate::{
    ports::{
        AgentOrchestrator, AgentOrchestratorResponse, Clock, ContextObjectRepository, Embedder,
        EmbedderQuery, GameSessionRepository, IdGenerator, MessageRepository, PromptAssembler,
        PromptContextObject, ProposedContextObject, RetrivialCandidate, RetrivialService, UserPort,
        UserPortError, VectorSearchQuery, VectorSearcher, VectorUpsertQuery,
    },
    use_cases::{UseCase, UseCaseResult},
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
    context_object_repo: Arc<dyn ContextObjectRepository>,
    current_user: Arc<dyn UserPort>,
    prompt_assembly: Arc<dyn PromptAssembler>,
    retrivial: Arc<dyn RetrivialService>,
    agent: Arc<dyn AgentOrchestrator>,
    embedder: Arc<dyn Embedder>,
    vector_searcher: Arc<dyn VectorSearcher>,
    clock: Arc<dyn Clock>,
    id_generator: Arc<dyn IdGenerator>,
}

impl SendMessageUseCase {
    pub fn new(
        session_repo: Arc<dyn GameSessionRepository>,
        message_repo: Arc<dyn MessageRepository>,
        context_object_repo: Arc<dyn ContextObjectRepository>,
        current_user: Arc<dyn UserPort>,
        prompt_assembly: Arc<dyn PromptAssembler>,
        retrivial: Arc<dyn RetrivialService>,
        agent: Arc<dyn AgentOrchestrator>,
        embedder: Arc<dyn Embedder>,
        vector_searcher: Arc<dyn VectorSearcher>,
        clock: Arc<dyn Clock>,
        id_generator: Arc<dyn IdGenerator>,
    ) -> Self {
        Self {
            session_repo,
            message_repo,
            context_object_repo,
            current_user,
            prompt_assembly,
            retrivial,
            agent,
            embedder,
            vector_searcher,
            clock,
            id_generator,
        }
    }

    async fn create_context_object(
        &self,
        session: &GameSession,
        object: ProposedContextObject,
        created_ts: chrono::DateTime<chrono::Utc>,
    ) -> UseCaseResult<ContextObject> {
        let provenance =
            Provenance::new("narrator_agent", i64::try_from(session.rng_state().seed())?)?;
        let context_object = ContextObject::new(
            self.id_generator.next_context_object_id().await,
            *session.id(),
            object.object_type,
            &object.title,
            &object.short_desc,
            object.long_desc.as_deref(),
            object.attributes,
            None,
            object.importance_score,
            provenance,
            created_ts,
        )?;

        self.context_object_repo.create(&context_object).await?;

        let embedding = self
            .embedder
            .create_embedding(EmbedderQuery {
                text: format!(
                    "{}\n{}\n{}",
                    context_object.title(),
                    context_object.short_desc(),
                    context_object.long_desc().map(String::as_str).unwrap_or("")
                ),
            })
            .await?;

        self.vector_searcher
            .upsert(VectorUpsertQuery {
                session_id: *session.id(),
                context_object_id: *context_object.id(),
                embedding: embedding.vector,
            })
            .await?;

        Ok(context_object)
    }

    async fn retrieve_context_objects(
        &self,
        session: &GameSession,
        player_message: &Message,
    ) -> UseCaseResult<Vec<RetrivialCandidate>> {
        let embedding = self
            .embedder
            .create_embedding(EmbedderQuery {
                text: player_message.text().to_owned(),
            })
            .await?;

        let search_results = self
            .vector_searcher
            .search(VectorSearchQuery {
                session_id: *session.id(),
                embedding: embedding.vector,
                k: session.config().retrivial_k(),
            })
            .await?;

        let mut candidates = Vec::with_capacity(search_results.len());
        for search_result in search_results {
            let context_object = self
                .context_object_repo
                .get_by_id(session.id(), &search_result.context_object_id)
                .await?;

            candidates.push(RetrivialCandidate {
                context_object,
                semantic_similarity: search_result.score,
            });
        }

        Ok(candidates)
    }
}

#[async_trait::async_trait]
impl UseCase<SendMessageCommand, SendMessageResponse> for SendMessageUseCase {
    async fn execute(&self, command: SendMessageCommand) -> UseCaseResult<SendMessageResponse> {
        let current_user_id = self.current_user.current_user_id().await?;
        let session = self.session_repo.get_by_id(&command.session_id).await?;

        if session.owner_id() != &current_user_id {
            return Err(UserPortError::Forbidden.into());
        }

        let message = Message::new(
            self.id_generator.next_message_id().await,
            command.session_id,
            MessageRole::Player,
            &command.text,
            self.clock.now().await,
        )?;

        self.message_repo.create(&message).await?;

        let recent_messages = self
            .message_repo
            .list_recent(&command.session_id, 20)
            .await?;
        let retrivial_candidates = self.retrieve_context_objects(&session, &message).await?;
        let retrivial_now = self.clock.now().await;
        let retrival_objects = self
            .retrivial
            .rerank(&session, retrivial_candidates, retrivial_now)
            .await?;

        let prompt_objects = retrival_objects
            .into_iter()
            .map(|obj| PromptContextObject {
                title: obj.context_object.title().to_owned(),
                summary: obj.context_object.short_desc().to_owned(),
            })
            .collect::<Vec<_>>();

        let prompt = self
            .prompt_assembly
            .assemble(&session, &recent_messages, &message, &prompt_objects)
            .await?;
        let agent_response = self.agent.generate(&prompt).await?;
        let activity_ts = self.clock.now().await;

        match agent_response {
            AgentOrchestratorResponse::Text(msg) => {
                let gm_message = Message::new(
                    self.id_generator.next_message_id().await,
                    command.session_id,
                    MessageRole::Gm,
                    &msg,
                    activity_ts,
                )?;

                self.message_repo.create(&gm_message).await?;
            }
            AgentOrchestratorResponse::CreateContextObject { message, object } => {
                let gm_message = Message::new(
                    self.id_generator.next_message_id().await,
                    command.session_id,
                    MessageRole::Gm,
                    &message,
                    activity_ts,
                )?;

                self.message_repo.create(&gm_message).await?;
                self.create_context_object(&session, object, activity_ts)
                    .await?;
            }
        }

        self.session_repo.update(&session).await?;

        Ok(SendMessageResponse {
            player_message_id: *message.id(),
        })
    }
}
