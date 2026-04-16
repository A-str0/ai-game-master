use domain::value_objects::{ContextObjectId, GameSessionId};
use thiserror::Error;

/// Query sent to the vector search backend.
#[derive(Debug, Clone)]
pub struct VectorSearchQuery {
    /// Session-specific collection to search in.
    pub session_id: GameSessionId,
    /// Query embedding.
    pub embedding: Vec<f32>,
    /// Maximum number of nearest neighbors to return.
    pub k: u8,
}

/// One vector-search hit.
#[derive(Debug, Clone)]
pub struct VectorSearchResponseObject {
    /// Identifier of the matched context object.
    pub context_object_id: ContextObjectId,
    /// Similarity score returned by the backend.
    pub score: f32,
}

/// Upsert payload sent to the vector search backend.
#[derive(Debug, Clone)]
pub struct VectorUpsertQuery {
    /// Session-specific collection to update.
    pub session_id: GameSessionId,
    /// Context object represented by the embedding.
    pub context_object_id: ContextObjectId,
    /// Embedding to store for future search.
    pub embedding: Vec<f32>,
}

/// Errors returned by [`VectorSearcher`].
#[derive(Debug, Error)]
pub enum VectorSearcherError {
    /// Backend could not be reached or completed the request.
    #[error("VectorSearcher backend unavailable: {details}")]
    Unavailable {
        /// Backend-specific error details.
        details: String,
    },
    /// Backend responded with malformed or unusable data.
    #[error("VectorSearcher backend returned invalid output: {details}")]
    InvalidResponse {
        /// Backend-specific error details.
        details: String,
    },
}

/// Convenient result alias returned by [`VectorSearcher`].
pub type VectorSearcherResult<T> = Result<T, VectorSearcherError>;

/// Port that stores and searches vectorized context objects per session.
#[async_trait::async_trait]
pub trait VectorSearcher: Send + Sync {
    /// Ensures that search storage exists for the supplied session.
    async fn ensure_session_collection(
        &self,
        session_id: GameSessionId,
    ) -> VectorSearcherResult<()>;
    /// Inserts or updates the embedding for one context object.
    async fn upsert(&self, query: VectorUpsertQuery) -> VectorSearcherResult<()>;
    /// Searches for nearest context objects in the supplied session collection.
    async fn search(
        &self,
        query: VectorSearchQuery,
    ) -> VectorSearcherResult<Vec<VectorSearchResponseObject>>;
}
