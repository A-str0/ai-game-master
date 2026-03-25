use chrono::{DateTime, Utc};

pub struct Clock;

#[async_trait::async_trait]
impl application::ports::ClockPort for Clock {
    async fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

impl Clock {
    pub fn new() -> Self {
        Self
    }
}
