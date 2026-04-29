//! HTTP transport layer built on top of Axum.

mod auth;
mod dto;
mod error;
mod handlers;
mod router;

pub(crate) use auth::{AuthenticationConfig, JwtAlgorithm};
/// Builds the root API router with all routes and shared state.
pub use router::build_router;
