use domain::value_objects::{ContextObjectId, GameSessionId};

use crate::{AppResult, services::Vector};

#[derive(Debug, Clone)]
pub struct VectorSearchQuery {
    pub session_id: GameSessionId,
    pub embedding: Vector,
    pub k: u8,
}

#[derive(Debug, Clone)]
pub struct VectorSearchResponseObject {
    pub context_object_id: ContextObjectId,
    pub score: f32,
}

#[async_trait::async_trait]
pub trait VectorSearcher: Send + Sync {
    async fn search(&self, query: VectorSearchQuery) -> AppResult<Vec<VectorSearchResponseObject>>;
}
