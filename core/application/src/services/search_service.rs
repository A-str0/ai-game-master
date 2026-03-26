use domain::value_objects::GameSessionId;

use crate::{AppResult, services::VectorSearchResponseObject};

#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub session_id: GameSessionId,
    pub text: String,
    pub k: u8,
}

#[async_trait::async_trait]
pub trait SearchService: Send + Sync {
    async fn search(&self, query: SearchQuery) -> AppResult<Vec<VectorSearchResponseObject>>;
}
