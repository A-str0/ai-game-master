use crate::{
    DomainResult,
    value_objects::{GameSessionMode, ScoringWeights},
};

/// Configuration embedded into a session and used by application services.
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
    fn validate(retrivial_k: u8, memory_budget: u32) -> DomainResult<()> {
        if retrivial_k <= 0 {
            return Err(crate::DomainError::InvariantViolation(String::from(
                "GameSessionConfig retrivial_k must be greater than 0",
            )));
        }

        if memory_budget <= 0 {
            return Err(crate::DomainError::InvariantViolation(String::from(
                "GameSessionConfig memory_budget must be greater than 0",
            )));
        }

        Ok(())
    }

    /// Creates a validated session configuration.
    pub fn new(
        retrivial_k: u8,
        memory_budget: u32,
        scoring_weights: ScoringWeights,
        session_mode: GameSessionMode,
    ) -> DomainResult<Self> {
        Self::validate(retrivial_k, memory_budget)?;

        Ok(Self {
            retrivial_k,
            memory_budget,
            scoring_weights,
            session_mode,
        })
    }

    /// Restores a session configuration from persisted state.
    pub fn restore(
        retrivial_k: u8,
        memory_budget: u32,
        scoring_weights: ScoringWeights,
        session_mode: GameSessionMode,
    ) -> DomainResult<Self> {
        Self::validate(retrivial_k, memory_budget)?;

        Ok(Self {
            retrivial_k,
            memory_budget,
            scoring_weights,
            session_mode,
        })
    }

    /// Returns how many vector-search candidates should be pulled before reranking.
    pub fn retrivial_k(&self) -> u8 {
        self.retrivial_k
    }

    /// Returns the approximate prompt budget reserved for retrieved memory.
    pub fn memory_budget(&self) -> u32 {
        self.memory_budget
    }

    /// Returns the weights used by domain scoring during retrieval.
    pub fn scoring_weights(&self) -> &ScoringWeights {
        &self.scoring_weights
    }

    /// Returns whether the session is configured for solo or multiplayer play.
    pub fn session_mode(&self) -> &GameSessionMode {
        &self.session_mode
    }
}
