use anyhow::{Result, bail};

pub enum ContextObjectType {
    Npc,
    Place,
    Item,
    Event,
    Note,
}

pub enum GameSessionMode {
    Solo,
    Multi,
}

/// ValueObject
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
