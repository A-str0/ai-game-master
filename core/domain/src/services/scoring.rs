use chrono::{DateTime, Utc};

use crate::value_objects::ScoringWeights;

// TODO: move to config
const DEFAULT_RECENCY_HALFLIFE_DAYS: f32 = 7.0;
const DEFAULT_SAME_LOCATION_MULTIPLIER: f32 = 1.2;
const DEFAULT_RELATED_MULTIPLIER: f32 = 1.1;

/// Input features used to score one retrieval candidate.
#[derive(Debug, Clone, Copy)]
pub struct ScoreInput {
    /// Semantic similarity returned by the vector search backend.
    pub semantic_similarity: f32,
    /// Domain-level importance assigned to the object when it was created.
    pub importance_score: f32,
    /// Most recent activity timestamp used to compute recency decay.
    pub last_updated_ts: Option<DateTime<Utc>>,
    /// Whether the object is in the same location as the active scene.
    pub is_same_location: bool,
    /// Whether the object is otherwise related to the current turn.
    pub is_related: bool,
}

/// Configuration knobs that influence scoring behavior.
#[derive(Debug, Clone, Copy)]
pub struct ScoringOptions {
    /// Reference time used to compute recency decay.
    pub now: DateTime<Utc>,
    /// Half-life for the recency signal, expressed in days.
    pub recency_halflife_days: f32,
    /// Multiplier applied when the candidate is in the same location.
    pub same_location_multiplier: f32,
    /// Multiplier applied when the candidate is otherwise related.
    pub related_multiplier: f32,
}

impl Default for ScoringOptions {
    fn default() -> Self {
        Self {
            now: Utc::now(),
            recency_halflife_days: DEFAULT_RECENCY_HALFLIFE_DAYS,
            same_location_multiplier: DEFAULT_SAME_LOCATION_MULTIPLIER,
            related_multiplier: DEFAULT_RELATED_MULTIPLIER,
        }
    }
}

/// A scored retrieval candidate together with its combined ranking score.
#[derive(Debug, Clone, Copy)]
pub struct ScoredInput {
    /// Original features that were evaluated.
    pub input: ScoreInput,
    /// Final weighted score used for ordering.
    pub combined_score: f32,
}

/// Pure service that combines retrieval signals into a single ranking score.
pub struct ScoringService;

impl ScoringService {
    /// Computes the final score for one candidate.
    pub fn score(input: &ScoreInput, weights: &ScoringWeights, options: &ScoringOptions) -> f32 {
        let semantic = Self::normalize_unit(input.semantic_similarity);
        let recency = Self::recency_score(input.last_updated_ts, options);
        let importance = Self::normalize_unit(input.importance_score);
        let proximity = if input.is_same_location { 1.0 } else { 0.0 };

        let mut combined = (weights.semantic() * semantic)
            + (weights.recency() * recency)
            + (weights.importance() * importance)
            + (weights.proximity() * proximity);

        if input.is_same_location {
            combined *= options.same_location_multiplier.max(0.0);
        }

        if input.is_related {
            combined *= options.related_multiplier.max(0.0);
        }

        combined
    }

    /// Scores and sorts candidates in descending order.
    pub fn score_candidates(
        inputs: &[ScoreInput],
        weights: &ScoringWeights,
        options: &ScoringOptions,
    ) -> Vec<ScoredInput> {
        let mut scored = inputs
            .iter()
            .copied()
            .map(|input| ScoredInput {
                combined_score: Self::score(&input, weights, options),
                input,
            })
            .collect::<Vec<_>>();

        scored.sort_by(|left, right| right.combined_score.total_cmp(&left.combined_score));
        scored
    }

    fn normalize_unit(value: f32) -> f32 {
        if value.is_finite() {
            value.clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    fn recency_score(last_updated_ts: Option<DateTime<Utc>>, options: &ScoringOptions) -> f32 {
        let Some(last_updated_ts) = last_updated_ts else {
            return 0.0;
        };

        if options.recency_halflife_days <= 0.0 || !options.recency_halflife_days.is_finite() {
            return 0.0;
        }

        let age_seconds = (options.now - last_updated_ts).num_seconds();
        if age_seconds <= 0 {
            return 1.0;
        }

        let half_life_seconds = options.recency_halflife_days * 86_400.0;
        let decay = 0.5_f32.powf((age_seconds as f32) / half_life_seconds);
        decay.clamp(0.0, 1.0)
    }
}
