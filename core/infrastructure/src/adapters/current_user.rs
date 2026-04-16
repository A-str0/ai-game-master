use application::ports::{UserPort, UserPortError, UserPortResult};
use domain::value_objects::UserId;

/// Request-scoped authenticated user context.
#[derive(Debug, Clone, Copy, Default)]
pub struct CurrentUserContext {
    user_id: Option<UserId>,
}

impl CurrentUserContext {
    /// Creates a context for an authenticated user.
    pub fn authenticated(user_id: UserId) -> Self {
        Self {
            user_id: Some(user_id),
        }
    }

    /// Creates an anonymous context with no authenticated user.
    pub fn anonymous() -> Self {
        Self { user_id: None }
    }
}

impl From<UserId> for CurrentUserContext {
    fn from(user_id: UserId) -> Self {
        Self::authenticated(user_id)
    }
}

/// [`UserPort`] adapter backed by a request-local [`CurrentUserContext`].
#[derive(Debug, Clone, Copy, Default)]
pub struct RequestCurrentUser {
    context: CurrentUserContext,
}

impl RequestCurrentUser {
    /// Creates a new adapter from the supplied request context.
    pub fn new(context: CurrentUserContext) -> Self {
        Self { context }
    }
}

#[async_trait::async_trait]
impl UserPort for RequestCurrentUser {
    async fn current_user_id(&self) -> UserPortResult<UserId> {
        self.context.user_id.ok_or(UserPortError::Unauthenticated)
    }
}
