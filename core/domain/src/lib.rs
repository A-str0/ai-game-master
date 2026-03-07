use std::{fmt::Debug, hash::Hash};
use thiserror::Error;

pub mod aggregates;
pub mod services;
pub mod value_objects;

pub trait Identifiable {
    type Id: Copy + PartialEq + Hash + Debug + 'static;

    fn id(&self) -> &Self::Id;
}

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("domain invariant violated: {0}")]
    InvariantViolation(String),
}

pub type DomainResult<T> = Result<T, DomainError>;
