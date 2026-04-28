use std::sync::Arc;

use axum::{
    Router,
    extract::FromRef,
    routing::{get, post},
};
use domain::value_objects::UserId;

use crate::app_service::ApiApplicationService;

use super::{
    auth::AuthenticationConfig,
    handlers::{
        create_session_handle, get_message_handle, get_messages_handle, get_session_handle,
        send_message_handle,
    },
};

/// Shared Axum state used by HTTP handlers and extractors.
#[derive(Clone)]
pub struct HttpApiState {
    application: Arc<dyn ApiApplicationService>,
    authentication: AuthenticationConfig,
}

impl HttpApiState {
    fn new(application: Arc<dyn ApiApplicationService>, default_user_id: UserId) -> Self {
        Self {
            application,
            authentication: AuthenticationConfig::new(default_user_id),
        }
    }
}

impl FromRef<HttpApiState> for Arc<dyn ApiApplicationService> {
    fn from_ref(state: &HttpApiState) -> Self {
        Arc::clone(&state.application)
    }
}

impl FromRef<HttpApiState> for AuthenticationConfig {
    fn from_ref(state: &HttpApiState) -> Self {
        state.authentication
    }
}

/// Builds the HTTP router for the public API surface.
///
/// Registered routes:
/// - `POST /api/sessions`
/// - `GET /api/sessions/{session_id}`
/// - `GET /api/messages`
/// - `GET /api/messages/{message_id}`
/// - `POST /api/messages`
pub fn build_router(
    application: Arc<dyn ApiApplicationService>,
    default_user_id: UserId,
) -> Router {
    Router::new()
        .route("/api/sessions", post(create_session_handle))
        .route("/api/sessions/{session_id}", get(get_session_handle))
        .route(
            "/api/messages",
            get(get_messages_handle).post(send_message_handle),
        )
        .route("/api/messages/{message_id}", get(get_message_handle))
        .with_state(HttpApiState::new(application, default_user_id))
}
