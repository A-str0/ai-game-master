use domain::{
    aggregates::{ContextObject, GameSession},
    services::{ScoreInput, ScoringOptions, ScoringService},
};
use thiserror::Error;

/// Candidate object returned by vector search before reranking.
#[derive(Debug, Clone)]
pub struct RetrivialCandidate {
    /// Domain object loaded from persistent storage.
    pub context_object: ContextObject,
    /// Raw semantic similarity returned by vector search.
    pub semantic_similarity: f32,
}

/// Candidate object after domain-specific reranking.
#[derive(Debug, Clone)]
pub struct RetrivialObject {
    /// Domain object loaded from persistent storage.
    pub context_object: ContextObject,
    /// Raw semantic similarity returned by vector search.
    pub semantic_similarity: f32,
    /// Final combined score produced by domain scoring.
    pub combined_score: f32,
}

/// Errors returned by [`RetrivialServicePort`].
#[derive(Debug, Error)]
pub enum RetrivialServiceError {
    /// Reranking service is unavailable.
    #[error("RetrivialService unavailable")]
    Unavailable,
    /// Service returned invalid or inconsistent data.
    #[error("RetrivialService returned invalid data: {details}")]
    Internal {
        /// Validation details explaining why the payload was rejected.
        details: String,
    },
}

/// Convenient result alias returned by [`RetrivialServicePort`].
pub type RetrivialServiceResult<T> = Result<T, RetrivialServiceError>;

/// Service that reranks retrieved context objects using domain scoring.
#[async_trait::async_trait]
pub trait RetrivialServicePort: Send + Sync {
    /// Reranks vector-search candidates for the supplied session.
    async fn rerank(
        &self,
        session: &GameSession,
        candidates: Vec<RetrivialCandidate>,
        now: chrono::DateTime<chrono::Utc>,
    ) -> RetrivialServiceResult<Vec<RetrivialObject>>;
}

/// Default reranking service built on top of [`ScoringService`].
pub struct RetrivialService;

impl RetrivialService {
    /// Creates a new reranking service.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl RetrivialServicePort for RetrivialService {
    async fn rerank(
        &self,
        session: &GameSession,
        candidates: Vec<RetrivialCandidate>,
        now: chrono::DateTime<chrono::Utc>,
    ) -> RetrivialServiceResult<Vec<RetrivialObject>> {
        let scoring_options = ScoringOptions {
            now,
            ..ScoringOptions::default()
        };

        let mut retrivial_objects = candidates
            .into_iter()
            .map(|candidate| {
                let score_input = ScoreInput {
                    semantic_similarity: candidate.semantic_similarity,
                    importance_score: candidate.context_object.importance_score(),
                    last_updated_ts: candidate
                        .context_object
                        .updated_ts()
                        .or(Some(candidate.context_object.created_ts())),
                    is_same_location: false,
                    is_related: false,
                };

                let combined_score = ScoringService::score(
                    &score_input,
                    session.config().scoring_weights(),
                    &scoring_options,
                );

                RetrivialObject {
                    context_object: candidate.context_object,
                    semantic_similarity: score_input.semantic_similarity,
                    combined_score,
                }
            })
            .collect::<Vec<_>>();

        retrivial_objects
            .sort_by(|left, right| right.combined_score.total_cmp(&left.combined_score));

        Ok(retrivial_objects)
    }
}
