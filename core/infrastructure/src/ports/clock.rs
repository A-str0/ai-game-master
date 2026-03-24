use chrono::{DateTime, Utc};

pub struct Clock;

#[async_trait::async_trait]
impl application::ports::Clock for Clock {
    async fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}
