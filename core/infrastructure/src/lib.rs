//! Infrastructure adapters for the AI Game Master workspace.
//!
//! This crate provides concrete implementations for application ports:
//! repositories backed by Postgres/Diesel and service adapters backed by
//! OpenRouter, Qdrant, and request-scoped auth state.

#![warn(missing_docs)]

/// Concrete adapters for application ports.
pub mod adapters;
/// Concrete persistence adapters and database wiring.
pub mod repositories;
