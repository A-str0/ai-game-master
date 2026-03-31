use crate::{DomainError, DomainResult};

/// ValueObject
#[derive(Debug, Clone)]
pub struct Provenance {
    created_by: String,
    seed: i64,
}

impl Provenance {
    fn validate(created_by: &str) -> DomainResult<()> {
        if created_by.trim().is_empty() {
            return Err(DomainError::InvariantViolation(String::from(
                "Provenance must specify who it was created by",
            )));
        }

        Ok(())
    }

    pub fn new(created_by: &str, seed: i64) -> DomainResult<Self> {
        Self::validate(created_by)?;

        Ok(Self {
            created_by: created_by.to_owned(),
            seed,
        })
    }

    pub fn restore(created_by: &str, seed: i64) -> DomainResult<Self> {
        Self::validate(created_by)?;
        Ok(Self {
            created_by: created_by.to_owned(),
            seed,
        })
    }

    pub fn created_by(&self) -> &str {
        &self.created_by
    }

    pub fn seed(&self) -> i64 {
        self.seed
    }
}
