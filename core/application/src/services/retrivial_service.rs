use domain::{
    aggregates::{ContextObject, GameSession},
    services::{ScoreInput, ScoringOptions, ScoringService},
};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct RetrivialCandidate {
    pub context_object: ContextObject,
    pub semantic_similarity: f32,
}

#[derive(Debug, Clone)]
pub struct RetrivialObject {
    pub context_object: ContextObject,
    pub semantic_similarity: f32,
    pub combined_score: f32,
}

#[derive(Debug, Error)]
pub enum RetrivialServiceError {
    #[error("RetrivialService unavailable")]
    Unavailable,
    #[error("RetrivialService returned invalid data: {details}")]
    Internal { details: String },
}

pub type RetrivialServiceResult<T> = Result<T, RetrivialServiceError>;

#[async_trait::async_trait]
pub trait RetrivialServicePort: Send + Sync {
    async fn rerank(
        &self,
        session: &GameSession,
        candidates: Vec<RetrivialCandidate>,
        now: chrono::DateTime<chrono::Utc>,
    ) -> RetrivialServiceResult<Vec<RetrivialObject>>;
}

pub struct RetrivialService;

impl RetrivialService {
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
