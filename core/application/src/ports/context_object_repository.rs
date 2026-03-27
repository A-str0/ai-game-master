use domain::{
    aggregates::ContextObject,
    value_objects::{ContextObjectId, GameSessionId},
};

use crate::ports::RepoResult;

#[async_trait::async_trait]
pub trait ContextObjectRepository: Send + Sync {
    async fn create(&self, context_object: &ContextObject) -> RepoResult<()>;
    async fn get_by_id(
        &self,
        session_id: &GameSessionId,
        id: &ContextObjectId,
    ) -> RepoResult<ContextObject>;
    async fn update(&self, context_object: &ContextObject) -> RepoResult<()>;
}
