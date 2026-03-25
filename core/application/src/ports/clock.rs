use chrono::{DateTime, Utc};

#[async_trait::async_trait]
pub trait ClockPort: Send + Sync {
    async fn now(&self) -> DateTime<Utc>;
}
