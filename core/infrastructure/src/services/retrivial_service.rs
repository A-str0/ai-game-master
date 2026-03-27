use application::{
    ports::{Clock, ContextObjectRepository},
    services::{
        Embedder, EmbedderQuery, RetrivialObject, RetrivialService, RetrivialServiceError,
        RetrivialServiceResult, VectorSearchQuery, VectorSearcher,
    },
};
use domain::{
    Identifiable,
    aggregates::{GameSession, Message},
    services::{ScoreInput, ScoringOptions, ScoringService},
};
use std::sync::Arc;

pub struct QdRetrivialService {
    embedder: Arc<dyn Embedder>,
    vector_searcher: Arc<dyn VectorSearcher>,
    context_object_repo: Arc<dyn ContextObjectRepository>,
    clock: Arc<dyn Clock>,
}

impl QdRetrivialService {
    pub fn new(
        embedder: Arc<dyn Embedder>,
        vector_searcher: Arc<dyn VectorSearcher>,
        context_object_repo: Arc<dyn ContextObjectRepository>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            embedder,
            vector_searcher,
            context_object_repo,
            clock,
        }
    }
}

#[async_trait::async_trait]
impl RetrivialService for QdRetrivialService {
    async fn find_for_message(
        &self,
        session: &GameSession,
        player_message: &Message,
    ) -> RetrivialServiceResult<Vec<RetrivialObject>> {
        let embedding = self
            .embedder
            .create_embedding(EmbedderQuery {
                text: player_message.text().to_owned(),
            })
            .await
            .map_err(map_embedder_error)?;

        let search_results = self
            .vector_searcher
            .search(VectorSearchQuery {
                session_id: *session.id(),
                embedding: embedding.vector,
                k: session.config().retrivial_k(),
            })
            .await
            .map_err(map_vector_searcher_error)?;

        let scoring_options = ScoringOptions {
            now: self.clock.now().await,
            ..ScoringOptions::default()
        };

        let mut retrivial_objects = Vec::with_capacity(search_results.len());

        for search_result in search_results {
            let context_object = self
                .context_object_repo
                .get_by_id(session.id(), &search_result.context_object_id)
                .await
                .map_err(map_context_object_repository_error)?;

            let score_input = ScoreInput {
                semantic_similarity: search_result.score,
                importance_score: context_object.importance_score(),
                last_updated_ts: context_object
                    .updated_ts()
                    .or(Some(context_object.created_ts())),
                is_same_location: false,
                is_related: false,
            };

            let combined_score = ScoringService::score(
                &score_input,
                session.config().scoring_weights(),
                &scoring_options,
            );

            retrivial_objects.push(RetrivialObject {
                context_object,
                semantic_similarity: search_result.score,
                combined_score,
            });
        }

        retrivial_objects
            .sort_by(|left, right| right.combined_score.total_cmp(&left.combined_score));

        Ok(retrivial_objects)
    }
}

fn map_embedder_error(error: application::services::EmbedderError) -> RetrivialServiceError {
    match error {
        application::services::EmbedderError::Unavailable { details } => {
            RetrivialServiceError::Unavailable {
                details: format!("failed to embed player message: {details}"),
            }
        }
        application::services::EmbedderError::InvalidResponse { details } => {
            RetrivialServiceError::Internal {
                details: format!("embedder returned invalid data for player message: {details}"),
            }
        }
    }
}

fn map_vector_searcher_error(
    error: application::services::VectorSearcherError,
) -> RetrivialServiceError {
    match error {
        application::services::VectorSearcherError::Unavailable { details } => {
            RetrivialServiceError::Unavailable {
                details: format!("vector search failed during retrieval: {details}"),
            }
        }
        application::services::VectorSearcherError::InvalidResponse { details } => {
            RetrivialServiceError::Internal {
                details: format!("vector search returned invalid data during retrieval: {details}"),
            }
        }
    }
}

fn map_context_object_repository_error(
    error: application::ports::ContextObjectRepositoryError,
) -> RetrivialServiceError {
    match error {
        application::ports::ContextObjectRepositoryError::Unavailable { details } => {
            RetrivialServiceError::Unavailable {
                details: format!("failed to load retrieved context object from storage: {details}"),
            }
        }
        application::ports::ContextObjectRepositoryError::NotFound { resource, details } => {
            RetrivialServiceError::Internal {
                details: format!(
                    "retrieval returned {resource} id that does not exist in storage: {details}"
                ),
            }
        }
        application::ports::ContextObjectRepositoryError::Conflict { resource, details } => {
            RetrivialServiceError::Internal {
                details: format!(
                    "unexpected conflict while loading retrieved {resource}: {details}"
                ),
            }
        }
        application::ports::ContextObjectRepositoryError::Internal { details } => {
            RetrivialServiceError::Internal {
                details: format!("context object storage returned invalid data: {details}"),
            }
        }
    }
}
