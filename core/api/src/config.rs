use anyhow::{Context, Result};

use crate::http::{AuthenticationConfig, JwtAlgorithm};

/// Runtime configuration required to start the HTTP API.
#[derive(Debug, Clone)]
pub struct ApiConfig {
    /// Socket address the API should bind to.
    pub bind_addr: String,
    /// Postgres connection string used by the repositories.
    pub database_url: String,
    /// JWT authentication configuration used by request extractors.
    pub authentication: AuthenticationConfig,
}

impl ApiConfig {
    /// Loads configuration from environment variables.
    ///
    /// Supported variables:
    /// - `API_ADDR`
    /// - `DATABASE_URL`
    /// - `JWT_ALGORITHM`
    /// - `JWT_SECRET`
    /// - `JWT_ISSUER`
    /// - `JWT_AUDIENCE`
    /// - `JWT_USER_ID_CLAIM`
    /// - `JWT_LEEWAY_SECONDS`
    pub fn from_env() -> Result<Self> {
        let bind_addr = std::env::var("API_ADDR").unwrap_or_else(|_| String::from("0.0.0.0:3000"));
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            String::from("postgres://postgres:postgres@127.0.0.1:5432/ai_game_master")
        });

        let algorithm = match std::env::var("JWT_ALGORITHM") {
            Ok(raw) => JwtAlgorithm::parse(&raw)
                .with_context(|| format!("invalid JWT_ALGORITHM value: {raw}"))?,
            Err(std::env::VarError::NotPresent) => JwtAlgorithm::Hs256,
            Err(error) => return Err(anyhow::anyhow!(error)),
        };

        let issuer = match std::env::var("JWT_ISSUER") {
            Ok(raw) if !raw.trim().is_empty() => Some(raw),
            Ok(_) | Err(std::env::VarError::NotPresent) => None,
            Err(error) => return Err(anyhow::anyhow!(error)),
        };

        let audience = match std::env::var("JWT_AUDIENCE") {
            Ok(raw) if !raw.trim().is_empty() => Some(raw),
            Ok(_) | Err(std::env::VarError::NotPresent) => None,
            Err(error) => return Err(anyhow::anyhow!(error)),
        };

        let user_id_claim =
            std::env::var("JWT_USER_ID_CLAIM").unwrap_or_else(|_| String::from("sub"));
        let leeway_seconds = match std::env::var("JWT_LEEWAY_SECONDS") {
            Ok(raw) => raw
                .parse::<u64>()
                .with_context(|| format!("invalid JWT_LEEWAY_SECONDS value: {raw}"))?,
            Err(std::env::VarError::NotPresent) => 0,
            Err(error) => return Err(anyhow::anyhow!(error)),
        };

        let authentication = match algorithm {
            JwtAlgorithm::Hs256 => {
                let secret = std::env::var("JWT_SECRET")
                    .context("JWT_SECRET is required when JWT_ALGORITHM=HS256")?;

                AuthenticationConfig::new_hs256(
                    secret,
                    issuer,
                    audience,
                    user_id_claim,
                    leeway_seconds,
                )
            }
        };

        Ok(Self {
            bind_addr,
            database_url,
            authentication,
        })
    }
}
