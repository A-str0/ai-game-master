use crate::{
    application::{AppError, ports::RepoResult},
    domain::{aggregates::GameSession, value_objects::GameSessionId},
};

pub trait GameSessionRepository {
    async fn create(&self, session: &GameSession) -> RepoResult<()>;
    async fn get(&self, id: &GameSessionId) -> RepoResult<GameSession>;
}
