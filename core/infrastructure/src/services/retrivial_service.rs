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

        todo!()
    }
}
