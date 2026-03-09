mod create_session;
mod get_session;
mod send_message;

pub use create_session::*;
pub use get_session::*;
pub use send_message::*;

use crate::AppResult;

#[async_trait::async_trait]
pub trait UseCase<C, O> {
    async fn execute(&self, command: C) -> AppResult<O>;
}
