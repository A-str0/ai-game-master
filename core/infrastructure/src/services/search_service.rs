use application::{
    AppResult,
    services::{VectorSearchQuery, VectorSearchResponseObject, VectorSearcher},
};

pub struct QdSearchService;

#[async_trait::async_trait]
impl VectorSearcher for QdSearchService {
    async fn search(&self, query: VectorSearchQuery) -> AppResult<Vec<VectorSearchResponseObject>> {
        todo!()
    }
}
