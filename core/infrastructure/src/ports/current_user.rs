use application::ports::{CurrentUser, CurrentUserError, CurrentUserResult};
use domain::value_objects::UserId;

#[derive(Debug, Clone, Copy, Default)]
pub struct CurrentUserContext {
    user_id: Option<UserId>,
}

impl CurrentUserContext {
    pub fn authenticated(user_id: UserId) -> Self {
        Self {
            user_id: Some(user_id),
        }
    }

    pub fn anonymous() -> Self {
        Self { user_id: None }
    }
}

impl From<UserId> for CurrentUserContext {
    fn from(user_id: UserId) -> Self {
        Self::authenticated(user_id)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RequestCurrentUser {
    context: CurrentUserContext,
}

impl RequestCurrentUser {
    pub fn new(context: CurrentUserContext) -> Self {
        Self { context }
    }
}

#[async_trait::async_trait]
impl CurrentUser for RequestCurrentUser {
    async fn current_user_id(&self) -> CurrentUserResult<UserId> {
        self.context
            .user_id
            .ok_or(CurrentUserError::Unauthenticated)
    }
}
