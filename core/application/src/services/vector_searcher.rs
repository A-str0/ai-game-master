use domain::value_objects::{ContextObjectId, GameSessionId};
use thiserror::Error;

use crate::services::Vector;

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

#[derive(Debug, Error)]
pub enum VectorSearcherError {
    #[error("VectorSearcher backend unavailable: {details}")]
    Unavailable { details: String },
    #[error("VectorSearcher backend returned invalid output: {details}")]
    InvalidResponse { details: String },
}

pub type VectorSearcherResult<T> = Result<T, VectorSearcherError>;

#[async_trait::async_trait]
pub trait VectorSearcher: Send + Sync {
    async fn ensure_session_collection(
        &self,
        session_id: GameSessionId,
    ) -> VectorSearcherResult<()>;
    async fn upsert(&self, query: VectorUpsertQuery) -> VectorSearcherResult<()>;
    async fn search(
        &self,
        query: VectorSearchQuery,
    ) -> VectorSearcherResult<Vec<VectorSearchResponseObject>>;
}
