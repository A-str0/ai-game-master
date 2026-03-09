use domain::aggregates::User;

use crate::ports::PortResult;

#[async_trait::async_trait]
pub trait UserAccessPort: Send + Sync {
    async fn get_user(&self) -> PortResult<User>;
}
