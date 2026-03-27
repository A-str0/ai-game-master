use std::sync::Arc;

use application::{
    ports::{ContextObjectRepository, IdGenerator, ProposedContextObject},
    services::{
        Embedder, EmbedderQuery, VectorSearcher, VectorUpsertQuery, WorldMemoryError,
        WorldMemoryManager, WorldMemoryResult,
    },
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
    ) -> WorldMemoryResult<()> {
        self.vector_searcher
            .ensure_session_collection(session_id)
            .await
            .map_err(map_vector_searcher_error)
    }

    async fn create_context_object(
        &self,
        session: &GameSession,
        object: ProposedContextObject,
    ) -> WorldMemoryResult<ContextObject> {
        let created_ts = self.clock.now().await;
        let provenance = Provenance::new(
            "narrator_agent",
            i64::try_from(session.rng_state().seed()).map_err(|_| WorldMemoryError::Internal {
                details: format!(
                    "session rng seed {} does not fit into i64",
                    session.rng_state().seed()
                ),
            })?,
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

        self.context_object_repo
            .create(&context_object)
            .await
            .map_err(map_context_object_repository_error)?;

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
            .await
            .map_err(map_embedder_error)?;

        self.vector_searcher
            .upsert(VectorUpsertQuery {
                session_id: *session.id(),
                context_object_id: *context_object.id(),
                embedding: embedding.vector,
            })
            .await
            .map_err(map_vector_searcher_error)?;

        Ok(context_object)
    }
}

fn map_context_object_repository_error(
    error: application::ports::ContextObjectRepositoryError,
) -> WorldMemoryError {
    match error {
        application::ports::ContextObjectRepositoryError::Conflict { resource, details } => {
            WorldMemoryError::Conflict { resource, details }
        }
        application::ports::ContextObjectRepositoryError::Unavailable { details } => {
            WorldMemoryError::Unavailable {
                details: format!("failed to persist context object: {details}"),
            }
        }
        application::ports::ContextObjectRepositoryError::Internal { details } => {
            WorldMemoryError::Internal {
                details: format!("context object storage returned invalid data: {details}"),
            }
        }
        application::ports::ContextObjectRepositoryError::NotFound { resource, details } => {
            WorldMemoryError::Internal {
                details: format!(
                    "unexpected missing {resource} while writing world memory: {details}"
                ),
            }
        }
    }
}

fn map_vector_searcher_error(
    error: application::services::VectorSearcherError,
) -> WorldMemoryError {
    match error {
        application::services::VectorSearcherError::Unavailable { details } => {
            WorldMemoryError::Unavailable {
                details: format!("vector index operation failed: {details}"),
            }
        }
        application::services::VectorSearcherError::InvalidResponse { details } => {
            WorldMemoryError::Internal {
                details: format!("vector index returned invalid data: {details}"),
            }
        }
    }
}

fn map_embedder_error(error: application::services::EmbedderError) -> WorldMemoryError {
    match error {
        application::services::EmbedderError::Unavailable { details } => {
            WorldMemoryError::Unavailable {
                details: format!("failed to build embedding for new context object: {details}"),
            }
        }
        application::services::EmbedderError::InvalidResponse { details } => {
            WorldMemoryError::Internal {
                details: format!(
                    "embedder returned invalid vector for new context object: {details}"
                ),
            }
        }
    }
}
