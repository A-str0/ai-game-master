use domain::{aggregates::ContextObject, value_objects::ContextObjectId};

use crate::ports::RepoResult;

#[async_trait::async_trait]
pub trait ContextObjectRepository: Send + Sync {
    async fn create(&self, object: &ContextObject) -> RepoResult<()>;
    async fn upsert(&self, object: &ContextObject) -> RepoResult<()>;
    async fn get_by_id(&self, id: &ContextObjectId) -> RepoResult<ContextObject>;
}
