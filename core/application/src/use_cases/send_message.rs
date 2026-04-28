use std::sync::Arc;

use domain::{
    Identifiable,
    aggregates::{ContextObject, GameSession, Message},
    value_objects::{GameSessionId, MessageId, MessageRole, Provenance},
};

use crate::{
    ports::{
        ContextObjectRepository, Embedder, EmbedderQuery, GameSessionRepository, MessageRepository,
        NarratorContextObject, ProposedContextObject, UnitOfWorkFactory, UserPort, UserPortError,
        VectorSearchQuery, VectorSearcher, VectorUpsertQuery,
    },
    services::{
        AgentOrchestrationService, Clock, IdGenerator, PromptAssembler, RetrivialCandidate,
        RetrivialServicePort,
    },
    use_cases::{UseCase, UseCaseResult},
};

/// Command that submits a new player message to a session.
pub struct SendMessageCommand {
    /// Target session identifier.
    pub session_id: GameSessionId,
    /// Raw player message text.
    pub text: String,
}

/// Result returned after the turn has been accepted.
pub struct SendMessageResponse {
    /// Identifier of the persisted player message.
    pub player_message_id: MessageId,
}

/// Main turn-processing use case.
///
/// The pipeline persists the player message, retrieves supporting context,
/// calls narration/extraction agents, stores the GM response, and persists any
/// newly extracted context objects.
pub struct SendMessageUseCase {
    session_repo: Arc<dyn GameSessionRepository>,
    message_repo: Arc<dyn MessageRepository>,
    context_object_repo: Arc<dyn ContextObjectRepository>,
    unit_of_work: Arc<dyn UnitOfWorkFactory>,
    current_user: Arc<dyn UserPort>,
    prompt_assembly: Arc<dyn PromptAssembler>,
    retrivial: Arc<dyn RetrivialServicePort>,
    agent_orchestration: Arc<AgentOrchestrationService>,
    embedder: Arc<dyn Embedder>,
    vector_searcher: Arc<dyn VectorSearcher>,
    clock: Arc<dyn Clock>,
    id_generator: Arc<dyn IdGenerator>,
}

impl SendMessageUseCase {
    /// Creates the use case with its required dependencies.
    pub fn new(
        session_repo: Arc<dyn GameSessionRepository>,
        message_repo: Arc<dyn MessageRepository>,
        context_object_repo: Arc<dyn ContextObjectRepository>,
        unit_of_work: Arc<dyn UnitOfWorkFactory>,
        current_user: Arc<dyn UserPort>,
        prompt_assembly: Arc<dyn PromptAssembler>,
        retrivial: Arc<dyn RetrivialServicePort>,
        agent_orchestration: Arc<AgentOrchestrationService>,
        embedder: Arc<dyn Embedder>,
        vector_searcher: Arc<dyn VectorSearcher>,
        clock: Arc<dyn Clock>,
        id_generator: Arc<dyn IdGenerator>,
    ) -> Self {
        Self {
            session_repo,
            message_repo,
            context_object_repo,
            unit_of_work,
            current_user,
            prompt_assembly,
            retrivial,
            agent_orchestration,
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
    ) -> UseCaseResult<(ContextObject, Vec<f32>)> {
        let provenance = Provenance::new("narrator_agent", session.rng_state().seed())?;
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

        Ok((context_object, embedding.vector))
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
        let mut session = self.session_repo.get_by_id(&command.session_id).await?;

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

        let retrivial_candidates = self.retrieve_context_objects(&session, &message).await?;
        let retrivial_now = self.clock.now().await;
        let retrivial_objects = self
            .retrivial
            .rerank(&session, retrivial_candidates, retrivial_now)
            .await?;

        let prompt_objects = retrivial_objects
            .into_iter()
            .map(|obj| NarratorContextObject {
                title: obj.context_object.title().to_owned(),
                summary: obj.context_object.short_desc().to_owned(),
            })
            .collect::<Vec<_>>();

        let recent_messages = self
            .message_repo
            .list_recent(&command.session_id, 20)
            .await?;

        let prompt = self
            .prompt_assembly
            .assemble(&session, &recent_messages, &message, &prompt_objects)
            .await?;
        let orchestration_response = self.agent_orchestration.generate(prompt).await?;
        let activity_ts = self.clock.now().await;

        let gm_message = Message::new(
            self.id_generator.next_message_id().await,
            command.session_id,
            MessageRole::Gm,
            &orchestration_response.message,
            activity_ts,
        )?;

        let mut context_objects = Vec::with_capacity(orchestration_response.objects.len());
        for object in orchestration_response.objects {
            let (context_object, embedding) = self
                .create_context_object(&session, object, activity_ts)
                .await?;
            self.vector_searcher
                .upsert(VectorUpsertQuery {
                    session_id: *session.id(),
                    context_object_id: *context_object.id(),
                    embedding,
                })
                .await?;
            context_objects.push(context_object);
        }

        session.mark_activity(activity_ts);
        let mut unit_of_work = self.unit_of_work.begin().await?;
        unit_of_work.insert_message(&message).await?;
        unit_of_work.insert_message(&gm_message).await?;
        for context_object in &context_objects {
            unit_of_work.insert_context_object(context_object).await?;
        }
        unit_of_work.update_session(&session).await?;
        unit_of_work.commit().await?;

        Ok(SendMessageResponse {
            player_message_id: *message.id(),
        })
    }
}
