use std::{fmt::Debug, hash::Hash};
use thiserror::Error;

pub mod agregates;
pub mod services;
pub mod value_objects;

trait Identifiable {
    type Id: Copy + PartialEq + Hash + Debug + 'static;

    fn id(&self) -> &Self::Id;
}

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("validation failed: {0}")]
    Validation(String),
}

pub type DomainResult<T> = Result<T, DomainError>;
