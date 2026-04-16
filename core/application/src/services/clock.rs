use chrono::{DateTime, Utc};

/// Time source abstraction used by use cases.
#[async_trait::async_trait]
pub trait Clock: Send + Sync {
    /// Returns the current UTC timestamp.
    async fn now(&self) -> DateTime<Utc>;
}

/// Production clock implementation backed by `Utc::now()`.
pub struct UtcClock;

#[async_trait::async_trait]
impl Clock for UtcClock {
    async fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

impl UtcClock {
    /// Creates a new UTC clock.
    pub fn new() -> Self {
        Self
    }
}
