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

#[derive(Debug, Clone)]
pub struct VectorUpsertQuery {
    pub session_id: GameSessionId,
    pub context_object_id: ContextObjectId,
    pub embedding: Vector,
}

#[async_trait::async_trait]
pub trait VectorSearcher: Send + Sync {
    async fn ensure_session_collection(&self, session_id: GameSessionId) -> AppResult<()>;
    async fn upsert(&self, query: VectorUpsertQuery) -> AppResult<()>;
    async fn search(&self, query: VectorSearchQuery) -> AppResult<Vec<VectorSearchResponseObject>>;
}
