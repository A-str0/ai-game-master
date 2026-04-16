//! Core domain model for the AI Game Master workspace.
//!
//! This crate contains the stable business concepts shared by the rest of the
//! system:
//! - aggregates that model sessions, messages, and durable world state
//! - value objects that define the domain vocabulary and invariants
//! - pure domain services used to score retrieved memories

#![warn(missing_docs)]

use std::{fmt::Debug, hash::Hash};
use thiserror::Error;

/// Aggregate roots and closely related domain entities.
pub mod aggregates;
/// Pure domain services that operate on validated value objects.
pub mod services;
/// Small validated types used by aggregates and services.
pub mod value_objects;

/// Common interface for aggregates and entities that expose a stable identifier.
pub trait Identifiable {
    /// Identifier type for the implementing domain object.
    type Id: Copy + PartialEq + Hash + Debug + 'static;

    /// Returns the stable identifier of the current object.
    fn id(&self) -> &Self::Id;
}

/// Error returned when domain invariants are violated.
#[derive(Debug, Error)]
pub enum DomainError {
    /// The provided state breaks one of the domain rules.
    #[error("domain invariant violated: {0}")]
    InvariantViolation(String),
}

/// Convenient result alias used across the domain layer.
pub type DomainResult<T> = Result<T, DomainError>;
