mod create_session;
mod get_session;

pub use create_session::{CreateSessionCommand, CreateSessionOutput, CreateSessionUseCase};
pub use get_session::GetSessionUseCase;

use crate::AppResult;

#[async_trait::async_trait]
trait UseCase<C, O> {
    async fn execute(&self, command: C) -> AppResult<O>;
}
