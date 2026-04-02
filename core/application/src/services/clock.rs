use chrono::{DateTime, Utc};

#[async_trait::async_trait]
pub trait Clock: Send + Sync {
    async fn now(&self) -> DateTime<Utc>;
}

pub struct UtcClock;

#[async_trait::async_trait]
impl Clock for UtcClock {
    async fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

impl UtcClock {
    pub fn new() -> Self {
        Self
    }
}
