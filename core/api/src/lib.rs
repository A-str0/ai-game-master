//! HTTP API crate for the AI Game Master workspace.
//!
//! The crate bootstraps concrete infrastructure adapters, exposes the Axum
//! router, and provides an application-facing service boundary for handlers.

#![warn(missing_docs)]

mod app_service;
mod bootstrap;
mod config;
mod http;

/// Runnable API server wrapper returned by the bootstrap process.
pub use bootstrap::{ApiServer, bootstrap_api_server};
/// Environment-driven API configuration.
pub use config::ApiConfig;
