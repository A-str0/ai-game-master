use application::{
    AppResult,
    ports::{Clock, ContextObjectRepository},
    services::{
        Embedder, EmbedderQuery, RetrivialObject, RetrivialService, VectorSearchQuery,
        VectorSearcher,
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
    ) -> AppResult<Vec<RetrivialObject>> {
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

        let scoring_options = ScoringOptions {
            now: self.clock.now().await,
            ..ScoringOptions::default()
        };

        let mut retrivial_objects = Vec::with_capacity(search_results.len());

        for search_result in search_results {
            let context_object = self
                .context_object_repo
                .get_by_id(session.id(), &search_result.context_object_id)
                .await?;

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
