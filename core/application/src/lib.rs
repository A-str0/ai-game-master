//! Application layer for the AI Game Master workspace.
//!
//! This crate coordinates domain objects through use cases, keeps external
//! systems behind ports, and implements stateless orchestration services.

#![warn(missing_docs)]

/// Traits and DTOs that define secondary ports.
pub mod ports;
/// Application services used by use cases.
pub mod services;
/// Primary entry points that execute user-facing behavior.
pub mod use_cases;
