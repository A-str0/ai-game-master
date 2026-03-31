use crate::DomainResult;

// TODO: move to config
const DEFAULT_WEIGHTS_SEMANTIC: f32 = 0.6;
const DEFAULT_WEIGHTS_RECENCY: f32 = 0.2;
const DEFAULT_WEIGHTS_IMPORTANCE: f32 = 0.15;
const DEFAULT_WEIGHTS_PROXIMITY: f32 = 0.05;

/// ValueObject
#[derive(Debug, Clone, Copy)]
pub struct ScoringWeights {
    semantic: f32,
    recency: f32,
    importance: f32,
    proximity: f32,
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            semantic: DEFAULT_WEIGHTS_SEMANTIC,
            recency: DEFAULT_WEIGHTS_RECENCY,
            importance: DEFAULT_WEIGHTS_IMPORTANCE,
            proximity: DEFAULT_WEIGHTS_PROXIMITY,
        }
    }
}

impl ScoringWeights {
    fn validate(semantic: f32, recency: f32, importance: f32, proximity: f32) -> DomainResult<()> {
        for (name, value) in [
            ("semantic", semantic),
            ("recency", recency),
            ("importance", importance),
            ("proximity", proximity),
        ] {
            if !value.is_finite() || !(0.0..=1.0).contains(&value) {
                return Err(crate::DomainError::InvariantViolation(format!(
                    "ScoringWeights {name} must be between 0.0 and 1.0"
                )));
            }
        }

        let total = semantic + recency + importance + proximity;
        if (total - 1.0).abs() > 0.000_1 {
            return Err(crate::DomainError::InvariantViolation(String::from(
                "ScoringWeights must sum to 1.0",
            )));
        }

        Ok(())
    }

    pub fn new(semantic: f32, recency: f32, importance: f32, proximity: f32) -> DomainResult<Self> {
        Self::validate(semantic, recency, importance, proximity)?;

        Ok(Self {
            semantic,
            recency,
            importance,
            proximity,
        })
    }

    pub fn semantic(&self) -> f32 {
        self.semantic
    }

    pub fn recency(&self) -> f32 {
        self.recency
    }

    pub fn importance(&self) -> f32 {
        self.importance
    }

    pub fn proximity(&self) -> f32 {
        self.proximity
    }
}
