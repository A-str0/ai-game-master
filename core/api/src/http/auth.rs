use axum::{
    extract::{FromRef, FromRequestParts},
    http::{HeaderMap, request::Parts},
};
use domain::value_objects::UserId;
use uuid::Uuid;

use super::error::ApiError;

const USER_ID_HEADER: &str = "x-user-id";

/// Authentication settings used by request extractors.
#[derive(Debug, Clone, Copy)]
pub struct AuthenticationConfig {
    default_user_id: UserId,
}

impl AuthenticationConfig {
    /// Creates a new authentication config with a fallback user ID.
    pub fn new(default_user_id: UserId) -> Self {
        Self { default_user_id }
    }

    fn resolve_user_id(&self, headers: &HeaderMap) -> Result<UserId, ApiError> {
        let Some(raw_user_id) = headers.get(USER_ID_HEADER) else {
            return Ok(self.default_user_id);
        };

        let raw_user_id = raw_user_id
            .to_str()
            .map_err(|_| ApiError::bad_request("invalid x-user-id header"))?;
        let user_id = Uuid::parse_str(raw_user_id)
            .map(UserId)
            .map_err(|_| ApiError::bad_request("invalid x-user-id header"))?;

        Ok(user_id)
    }
}

/// Extracted request user resolved from `x-user-id` or the configured default.
#[derive(Debug, Clone, Copy)]
pub struct RequestUser {
    /// User identifier associated with the request.
    pub user_id: UserId,
}

impl<S> FromRequestParts<S> for RequestUser
where
    S: Send + Sync,
    AuthenticationConfig: FromRef<S>,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let config = AuthenticationConfig::from_ref(state);
        let user_id = config.resolve_user_id(&parts.headers)?;

        Ok(Self { user_id })
    }
}

#[cfg(test)]
mod tests {
    use axum::http::{HeaderMap, HeaderValue};
    use uuid::Uuid;

    use super::AuthenticationConfig;
    use domain::value_objects::UserId;

    #[test]
    fn uses_default_user_when_header_is_missing() {
        let default_user_id = UserId(Uuid::new_v4());
        let config = AuthenticationConfig::new(default_user_id);

        let user_id = config.resolve_user_id(&HeaderMap::new()).unwrap();

        assert_eq!(user_id, default_user_id);
    }

    #[test]
    fn rejects_invalid_user_id_header() {
        let mut headers = HeaderMap::new();
        headers.insert("x-user-id", HeaderValue::from_static("not-a-uuid"));

        let config = AuthenticationConfig::new(UserId(Uuid::new_v4()));
        let error = config.resolve_user_id(&headers).unwrap_err();

        assert_eq!(error.message(), "invalid x-user-id header");
    }
}
