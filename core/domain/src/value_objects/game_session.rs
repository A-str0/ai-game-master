use crate::DomainResult;

const DEFAULT_WEIGHTS_SEMANTIC: f32 = 0.6;
const DEFAULT_WEIGHTS_RECENCY: f32 = 0.2;
const DEFAULT_WEIGHTS_IMPORTANCE: f32 = 0.15;
const DEFAULT_WEIGHTS_PROXIMITY: f32 = 0.05;

/// ValueObject
#[derive(Debug, Clone, Copy)]
pub enum GameSessionMode {
    Solo,
    Multi,
}

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
    pub fn new(semantic: f32, recency: f32, importance: f32, proximity: f32) -> DomainResult<Self> {
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

#[derive(Debug, Clone, Copy)]
pub struct RngState {
    seed: u64,
    counter: u64,
}

impl Default for RngState {
    fn default() -> Self {
        Self {
            seed: 1337,
            counter: 0,
        }
    }
}

impl RngState {
    pub fn new(seed: u64, counter: u64) -> Self {
        Self { seed, counter }
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn counter(&self) -> u64 {
        self.counter
    }
}

/// ValueObject
#[derive(Debug, Clone, Copy)]
pub struct GameSessionConfig {
    retrivial_k: u8,
    memory_budget: u32,
    scoring_weights: ScoringWeights,
    session_mode: GameSessionMode,
}

impl Default for GameSessionConfig {
    fn default() -> Self {
        Self {
            retrivial_k: 10,
            memory_budget: 2000,
            scoring_weights: ScoringWeights::default(),
            session_mode: GameSessionMode::Solo,
        }
    }
}

impl GameSessionConfig {
    pub fn new(
        retrivial_k: u8,
        memory_budget: u32,
        scoring_weights: ScoringWeights,
        session_mode: GameSessionMode,
    ) -> DomainResult<Self> {
        if retrivial_k == 0 {
            return Err(crate::DomainError::InvariantViolation(String::from(
                "GameSessionConfig retrivial_k must be greater than 0",
            )));
        }

        if memory_budget == 0 {
            return Err(crate::DomainError::InvariantViolation(String::from(
                "GameSessionConfig memory_budget must be greater than 0",
            )));
        }

        Ok(Self {
            retrivial_k,
            memory_budget,
            scoring_weights,
            session_mode,
        })
    }

    pub fn restore(
        retrivial_k: u8,
        memory_budget: u32,
        scoring_weights: ScoringWeights,
        session_mode: GameSessionMode,
    ) -> DomainResult<Self> {
        Self::new(retrivial_k, memory_budget, scoring_weights, session_mode)
    }

    pub fn retrivial_k(&self) -> u8 {
        self.retrivial_k
    }

    pub fn memory_budget(&self) -> u32 {
        self.memory_budget
    }

    pub fn scoring_weights(&self) -> &ScoringWeights {
        &self.scoring_weights
    }

    pub fn session_mode(&self) -> &GameSessionMode {
        &self.session_mode
    }
}
