mod create_session;

use crate::AppError;

pub type AppResult<T> = Result<T, AppError>;

#[async_trait::async_trait]
trait UseCase<C, R> {
    async fn execute(&self, command: C) -> AppResult<R>;
}
