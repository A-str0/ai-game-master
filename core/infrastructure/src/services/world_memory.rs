use std::sync::Arc;

use application::{
    AppResult,
    ports::{ContextObjectRepository, IdGenerator, ProposedContextObject},
    services::{Embedder, EmbedderQuery, VectorSearcher, VectorUpsertQuery, WorldMemoryManager},
};
use domain::{
    Identifiable,
    aggregates::{ContextObject, GameSession},
    value_objects::Provenance,
};

pub struct DefaultWorldMemoryManager {
    context_object_repo: Arc<dyn ContextObjectRepository>,
    embedder: Arc<dyn Embedder>,
    vector_searcher: Arc<dyn VectorSearcher>,
    clock: Arc<dyn application::ports::Clock>,
    id_generator: Arc<dyn IdGenerator>,
}

impl DefaultWorldMemoryManager {
    pub fn new(
        context_object_repo: Arc<dyn ContextObjectRepository>,
        embedder: Arc<dyn Embedder>,
        vector_searcher: Arc<dyn VectorSearcher>,
        clock: Arc<dyn application::ports::Clock>,
        id_generator: Arc<dyn IdGenerator>,
    ) -> Self {
        Self {
            context_object_repo,
            embedder,
            vector_searcher,
            clock,
            id_generator,
        }
    }
}

#[async_trait::async_trait]
impl WorldMemoryManager for DefaultWorldMemoryManager {
    async fn initialize_session(
        &self,
        session_id: domain::value_objects::GameSessionId,
    ) -> AppResult<()> {
        self.vector_searcher
            .ensure_session_collection(session_id)
            .await
    }

    async fn create_context_object(
        &self,
        session: &GameSession,
        object: ProposedContextObject,
    ) -> AppResult<ContextObject> {
        let created_ts = self.clock.now().await;
        let provenance = Provenance::new(
            "narrator_agent",
            i64::try_from(session.rng_state().seed())
                .map_err(|_| application::AppError::Unavailable)?,
        )?;
        let context_object = ContextObject::new(
            self.id_generator.next_context_object_id().await,
            session.id().clone(),
            object.object_type,
            &object.title,
            &object.short_desc,
            object.long_desc.as_deref(),
            object.attributes,
            None,
            object.importance_score,
            provenance,
            created_ts,
            Some(created_ts),
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
}
