use anyhow::{Context, Result};
use domain::value_objects::UserId;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub bind_addr: String,
    pub database_url: String,
    pub default_user_id: UserId,
}

impl ApiConfig {
    pub fn from_env() -> Result<Self> {
        let bind_addr = std::env::var("API_ADDR").unwrap_or_else(|_| String::from("0.0.0.0:3000"));
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            String::from("postgres://postgres:postgres@127.0.0.1:5432/ai_game_master")
        });

        let default_user_id = match std::env::var("API_DEFAULT_USER_ID") {
            Ok(raw) => Uuid::parse_str(&raw)
                .with_context(|| format!("invalid API_DEFAULT_USER_ID value: {raw}"))
                .map(UserId),
            Err(std::env::VarError::NotPresent) => Ok(UserId(Uuid::new_v4())),
            Err(error) => Err(anyhow::anyhow!(error)),
        }?;

        Ok(Self {
            bind_addr,
            database_url,
            default_user_id,
        })
    }
}
