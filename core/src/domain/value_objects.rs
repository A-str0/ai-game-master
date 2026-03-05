use anyhow::{Result, bail};

/// ValueObject
#[derive(Debug)]
pub enum ContextObjectType {
    Npc,
    Place,
    Item,
    Event,
    Note,
}

/// ValueObject
#[derive(Debug)]
pub enum GameSessionMode {
    Solo,
    Multi,
}

/// ValueObject
#[derive(Debug)]
pub struct ScoringWeights {
    semantic: f32,
    recency: f32,
    importance: f32,
    proximity: f32,
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            semantic: Default::default(),
            recency: Default::default(),
            importance: Default::default(),
            proximity: Default::default(),
        }
    }
}

/// ValueObject
#[derive(Debug)]
pub struct GameSessionConfig {
    retrivial_k: u8,
    memory_budget: u32,
    scoring_weights: ScoringWeights,
}

impl Default for GameSessionConfig {
    fn default() -> Self {
        Self {
            retrivial_k: 10,
            memory_budget: 2000,
            scoring_weights: ScoringWeights::default(),
        }
    }
}

// TODO: пересмотреть new() и restore()
impl GameSessionConfig {
    pub fn new(
        retrivial_k: u8,
        memory_budget: u32,
        scoring_weights: ScoringWeights,
    ) -> Result<Self> {
        Ok(Self {
            retrivial_k,
            memory_budget,
            scoring_weights,
        })
    }

    pub fn restore(retrivial_k: u8, memory_budget: u32, scoring_weights: ScoringWeights) -> Self {
        Self {
            retrivial_k,
            memory_budget,
            scoring_weights,
        }
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
}

/// ValueObject
#[derive(Debug)]
pub struct Provenance {
    created_by: String,
    seed: i64,
}

impl Provenance {
    pub fn new(created_by: &str, seed: i64) -> Result<Self> {
        if created_by.trim().is_empty() {
            bail!("Provenance must specify who it was created by");
        }

        Ok(Self {
            created_by: String::from(created_by),
            seed,
        })
    }

    pub fn restore(created_by: String, seed: i64) -> Self {
        Self { created_by, seed }
    }

    pub fn created_by(&self) -> &str {
        &self.created_by
    }

    pub fn seed(&self) -> i64 {
        self.seed
    }
}
