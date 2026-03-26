use application::ports::{ContextObjectRepository, RepoResult};
use domain::{aggregates::ContextObject, value_objects::ContextObjectId};

pub struct QdContextObjectRepository {}

#[async_trait::async_trait]
impl ContextObjectRepository for QdContextObjectRepository {
    async fn create(&self, context_object: &ContextObject) -> RepoResult<()> {
        todo!()
    }

    async fn get_by_id(&self, id: &ContextObjectId) -> RepoResult<ContextObject> {
        todo!()
    }

    async fn update(&self, context_object: &ContextObject) -> RepoResult<()> {
        todo!()
    }
}
