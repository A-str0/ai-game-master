use crate::{DomainError, DomainResult};

/// Metadata describing how a context object was produced.
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

    /// Creates new provenance metadata.
    pub fn new(created_by: &str, seed: i64) -> DomainResult<Self> {
        Self::validate(created_by)?;

        Ok(Self {
            created_by: created_by.to_owned(),
            seed,
        })
    }

    /// Restores provenance metadata from persisted state.
    pub fn restore(created_by: &str, seed: i64) -> DomainResult<Self> {
        Self::validate(created_by)?;
        Ok(Self {
            created_by: created_by.to_owned(),
            seed,
        })
    }

    /// Returns the actor or subsystem that produced the object.
    pub fn created_by(&self) -> &str {
        &self.created_by
    }

    /// Returns the RNG seed associated with the originating turn.
    pub fn seed(&self) -> i64 {
        self.seed
    }
}
