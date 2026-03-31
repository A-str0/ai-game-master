use chrono::{DateTime, Utc};

use crate::value_objects::ScoringWeights;

// TODO: move to config
const DEFAULT_RECENCY_HALFLIFE_DAYS: f32 = 7.0;
const DEFAULT_SAME_LOCATION_MULTIPLIER: f32 = 1.2;
const DEFAULT_RELATED_MULTIPLIER: f32 = 1.1;

/// DTO
#[derive(Debug, Clone, Copy)]
pub struct ScoreInput {
    pub semantic_similarity: f32,
    pub importance_score: f32,
    pub last_updated_ts: Option<DateTime<Utc>>,
    pub is_same_location: bool,
    pub is_related: bool,
}

/// DTO
#[derive(Debug, Clone, Copy)]
pub struct ScoringOptions {
    pub now: DateTime<Utc>,
    pub recency_halflife_days: f32,
    pub same_location_multiplier: f32,
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

#[derive(Debug, Clone, Copy)]
pub struct ScoredInput {
    pub input: ScoreInput,
    pub combined_score: f32,
}

pub struct ScoringService;

impl ScoringService {
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
