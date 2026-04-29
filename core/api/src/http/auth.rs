use std::collections::HashMap;

use anyhow::Result;
use axum::{
    extract::{FromRef, FromRequestParts},
    http::{HeaderMap, request::Parts},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use domain::value_objects::UserId;
use ring::hmac;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::error::ApiError;

const AUTHORIZATION_HEADER: &str = "authorization";

#[derive(Debug, Clone, Copy)]
pub enum JwtAlgorithm {
    Hs256,
}

impl JwtAlgorithm {
    pub fn parse(value: &str) -> Result<Self> {
        match value.trim().to_ascii_uppercase().as_str() {
            "HS256" => Ok(Self::Hs256),
            other => Err(anyhow::anyhow!("unsupported JWT algorithm: {other}")),
        }
    }
}

#[derive(Debug, Clone)]
struct JwtValidationConfig {
    issuer: Option<String>,
    audience: Option<String>,
    leeway_seconds: u64,
    signing_key: hmac::Key,
}

#[derive(Debug, Deserialize)]
struct JwtClaims {
    exp: u64,
    #[serde(default)]
    nbf: Option<u64>,
    #[serde(default)]
    iss: Option<String>,
    #[serde(default)]
    aud: Option<Value>,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[derive(Debug, Deserialize, Serialize)]
struct JwtHeader {
    alg: String,
}

/// Authentication settings used by request extractors.
#[derive(Debug, Clone)]
pub struct AuthenticationConfig {
    validation: JwtValidationConfig,
    user_id_claim: String,
}

impl AuthenticationConfig {
    pub fn new_hs256(
        secret: String,
        issuer: Option<String>,
        audience: Option<String>,
        user_id_claim: String,
        leeway_seconds: u64,
    ) -> Self {
        Self {
            validation: JwtValidationConfig {
                issuer,
                audience,
                leeway_seconds,
                signing_key: hmac::Key::new(hmac::HMAC_SHA256, secret.as_bytes()),
            },
            user_id_claim,
        }
    }

    fn resolve_user_id(&self, headers: &HeaderMap) -> Result<UserId, ApiError> {
        let raw_header = headers
            .get(AUTHORIZATION_HEADER)
            .ok_or_else(|| ApiError::unauthorized("missing bearer token"))?;
        let raw_header = raw_header
            .to_str()
            .map_err(|_| ApiError::unauthorized("invalid authorization header"))?;
        let token = raw_header
            .strip_prefix("Bearer ")
            .or_else(|| raw_header.strip_prefix("bearer "))
            .ok_or_else(|| ApiError::unauthorized("invalid authorization scheme"))?;

        let claims = self.decode_claims(token)?;
        let claim_value = claims
            .get(&self.user_id_claim)
            .ok_or_else(|| ApiError::unauthorized("user id claim is missing"))?;
        let raw_user_id = claim_value
            .as_str()
            .ok_or_else(|| ApiError::unauthorized("user id claim must be a string"))?;
        let user_id = Uuid::parse_str(raw_user_id)
            .map(UserId)
            .map_err(|_| ApiError::unauthorized("user id claim must be a UUID"))?;

        Ok(user_id)
    }

    fn decode_claims(&self, token: &str) -> Result<HashMap<String, Value>, ApiError> {
        let mut segments = token.split('.');
        let header_segment = segments
            .next()
            .ok_or_else(|| ApiError::unauthorized("invalid bearer token"))?;
        let payload_segment = segments
            .next()
            .ok_or_else(|| ApiError::unauthorized("invalid bearer token"))?;
        let signature_segment = segments
            .next()
            .ok_or_else(|| ApiError::unauthorized("invalid bearer token"))?;

        if segments.next().is_some() {
            return Err(ApiError::unauthorized("invalid bearer token"));
        }

        let header = decode_json::<JwtHeader>(header_segment)?;
        if header.alg != "HS256" {
            return Err(ApiError::unauthorized("invalid bearer token"));
        }

        let signature = URL_SAFE_NO_PAD
            .decode(signature_segment)
            .map_err(|_| ApiError::unauthorized("invalid bearer token"))?;
        let signed_data = format!("{header_segment}.{payload_segment}");
        hmac::verify(
            &self.validation.signing_key,
            signed_data.as_bytes(),
            &signature,
        )
        .map_err(|_| ApiError::unauthorized("invalid bearer token"))?;

        let claims = decode_json::<JwtClaims>(payload_segment)?;
        self.validate_registered_claims(&claims)?;

        Ok(claims.extra)
    }

    fn validate_registered_claims(&self, claims: &JwtClaims) -> Result<(), ApiError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| ApiError::service_unavailable("authentication clock unavailable"))?
            .as_secs();
        let leeway = self.validation.leeway_seconds;

        if claims.exp.saturating_add(leeway) < now {
            return Err(ApiError::unauthorized("invalid bearer token"));
        }

        if let Some(nbf) = claims.nbf
            && nbf > now.saturating_add(leeway)
        {
            return Err(ApiError::unauthorized("invalid bearer token"));
        }

        if let Some(expected_issuer) = &self.validation.issuer
            && claims.iss.as_deref() != Some(expected_issuer.as_str())
        {
            return Err(ApiError::unauthorized("invalid bearer token"));
        }

        if let Some(expected_audience) = &self.validation.audience {
            let audience_matches = match &claims.aud {
                Some(Value::String(value)) => value == expected_audience,
                Some(Value::Array(values)) => values
                    .iter()
                    .any(|value| value.as_str() == Some(expected_audience.as_str())),
                _ => false,
            };

            if !audience_matches {
                return Err(ApiError::unauthorized("invalid bearer token"));
            }
        }

        Ok(())
    }
}

fn decode_json<T>(segment: &str) -> Result<T, ApiError>
where
    T: for<'de> Deserialize<'de>,
{
    let raw = URL_SAFE_NO_PAD
        .decode(segment)
        .map_err(|_| ApiError::unauthorized("invalid bearer token"))?;

    serde_json::from_slice(&raw).map_err(|_| ApiError::unauthorized("invalid bearer token"))
}

/// Extracted request user resolved from a validated bearer token.
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
    use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
    use ring::hmac;
    use serde::Serialize;
    use uuid::Uuid;

    use super::{AuthenticationConfig, JwtHeader};

    #[derive(Debug, Serialize)]
    struct TestClaims {
        sub: String,
        exp: u64,
        iss: String,
        aud: String,
    }

    fn test_config() -> AuthenticationConfig {
        AuthenticationConfig::new_hs256(
            String::from("test-secret"),
            Some(String::from("iam-service")),
            Some(String::from("ai-game-master")),
            String::from("sub"),
            0,
        )
    }

    fn bearer_for(user_id: Uuid) -> String {
        let claims = TestClaims {
            sub: user_id.to_string(),
            exp: 4_102_444_800,
            iss: String::from("iam-service"),
            aud: String::from("ai-game-master"),
        };
        let header = JwtHeader {
            alg: String::from("HS256"),
        };

        format!("Bearer {}", encode_token(&header, &claims, "test-secret"))
    }

    #[test]
    fn resolves_user_from_valid_bearer_token() {
        let user_id = Uuid::new_v4();
        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            HeaderValue::from_str(&bearer_for(user_id)).unwrap(),
        );

        let resolved = test_config().resolve_user_id(&headers).unwrap();

        assert_eq!(resolved.0, user_id);
    }

    #[test]
    fn rejects_missing_authorization_header() {
        let error = test_config()
            .resolve_user_id(&HeaderMap::new())
            .unwrap_err();

        assert_eq!(error.message(), "missing bearer token");
    }

    #[test]
    fn rejects_invalid_user_id_claim() {
        let claims = TestClaims {
            sub: String::from("not-a-uuid"),
            exp: 4_102_444_800,
            iss: String::from("iam-service"),
            aud: String::from("ai-game-master"),
        };
        let header = JwtHeader {
            alg: String::from("HS256"),
        };
        let token = encode_token(&header, &claims, "test-secret");

        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
        );

        let error = test_config().resolve_user_id(&headers).unwrap_err();

        assert_eq!(error.message(), "user id claim must be a UUID");
    }

    fn encode_token(header: &JwtHeader, claims: &TestClaims, secret: &str) -> String {
        let header = URL_SAFE_NO_PAD.encode(serde_json::to_vec(header).unwrap());
        let claims = URL_SAFE_NO_PAD.encode(serde_json::to_vec(claims).unwrap());
        let signed_data = format!("{header}.{claims}");
        let key = hmac::Key::new(hmac::HMAC_SHA256, secret.as_bytes());
        let signature = hmac::sign(&key, signed_data.as_bytes());
        let signature = URL_SAFE_NO_PAD.encode(signature.as_ref());

        format!("{signed_data}.{signature}")
    }
}
