use crate::{
    DomainResult,
    value_objects::{GameSessionMode, ScoringWeights},
};

/// Aggregate Root
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
