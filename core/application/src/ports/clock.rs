use chrono::{DateTime, Utc};

#[async_trait::async_trait]
pub trait Clock: Send + Sync {
    async fn now(&self) -> DateTime<Utc>;
}
