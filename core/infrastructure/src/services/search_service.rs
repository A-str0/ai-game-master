use application::{
    AppError, AppResult,
    services::{VectorSearchQuery, VectorSearchResponseObject, VectorSearcher, VectorUpsertQuery},
};
use reqwest::{Client, StatusCode};
use serde_json::{Value, json};
use uuid::Uuid;

pub struct QdSearchService {
    client: Client,
    base_url: String,
    embedding_size: u64,
}

impl QdSearchService {
    pub fn new() -> Self {
        let base_url = std::env::var("QDRANT_URL")
            .unwrap_or_else(|_| String::from("http://127.0.0.1:6333"))
            .trim_end_matches('/')
            .to_owned();
        let embedding_size = std::env::var("EMBEDDING_DIMENSION")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(1024);

        Self {
            client: Client::new(),
            base_url,
            embedding_size,
        }
    }

    fn collection_name(&self, session_id: Uuid) -> String {
        format!("session_{}", session_id.simple())
    }

    async fn ensure_collection_internal(&self, session_id: Uuid) -> AppResult<()> {
        let collection_name = self.collection_name(session_id);
        let url = format!("{}/collections/{}", self.base_url, collection_name);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|_| AppError::Unavailable)?;

        if response.status().is_success() {
            return Ok(());
        }

        if response.status() != StatusCode::NOT_FOUND {
            return Err(AppError::Unavailable);
        }

        let response = self
            .client
            .put(&url)
            .json(&json!({
                "vectors": {
                    "size": self.embedding_size,
                    "distance": "Cosine"
                }
            }))
            .send()
            .await
            .map_err(|_| AppError::Unavailable)?;

        if response.status().is_success() || response.status() == StatusCode::CONFLICT {
            return Ok(());
        }

        Err(AppError::Unavailable)
    }
}

#[async_trait::async_trait]
impl VectorSearcher for QdSearchService {
    async fn ensure_session_collection(
        &self,
        session_id: domain::value_objects::GameSessionId,
    ) -> AppResult<()> {
        self.ensure_collection_internal(session_id.0).await
    }

    async fn upsert(&self, query: VectorUpsertQuery) -> AppResult<()> {
        self.ensure_collection_internal(query.session_id.0).await?;

        let url = format!(
            "{}/collections/{}/points?wait=true",
            self.base_url,
            self.collection_name(query.session_id.0)
        );
        let response = self
            .client
            .put(url)
            .json(&json!({
                "points": [{
                    "id": query.context_object_id.0,
                    "vector": query.embedding,
                    "payload": {
                        "context_object_id": query.context_object_id.0,
                    }
                }]
            }))
            .send()
            .await
            .map_err(|_| AppError::Unavailable)?;

        if response.status().is_success() {
            return Ok(());
        }

        Err(AppError::Unavailable)
    }

    async fn search(&self, query: VectorSearchQuery) -> AppResult<Vec<VectorSearchResponseObject>> {
        let collection_name = self.collection_name(query.session_id.0);
        let url = format!(
            "{}/collections/{}/points/search",
            self.base_url, collection_name
        );
        let response = self
            .client
            .post(url)
            .json(&json!({
                "vector": query.embedding,
                "limit": query.k,
                "with_payload": true
            }))
            .send()
            .await
            .map_err(|_| AppError::Unavailable)?;

        if response.status() == StatusCode::NOT_FOUND {
            return Ok(Vec::new());
        }

        if !response.status().is_success() {
            return Err(AppError::Unavailable);
        }

        let body: Value = response.json().await.map_err(|_| AppError::Unavailable)?;
        let results = body
            .get("result")
            .and_then(Value::as_array)
            .ok_or(AppError::Unavailable)?;

        results
            .iter()
            .map(|entry| {
                let context_object_id = entry
                    .get("payload")
                    .and_then(|payload| payload.get("context_object_id"))
                    .and_then(Value::as_str)
                    .and_then(|value| Uuid::parse_str(value).ok())
                    .map(domain::value_objects::ContextObjectId)
                    .or_else(|| {
                        entry
                            .get("id")
                            .and_then(parse_uuid_value)
                            .map(domain::value_objects::ContextObjectId)
                    })
                    .ok_or(AppError::Unavailable)?;
                let score = entry
                    .get("score")
                    .and_then(Value::as_f64)
                    .map(|value| value as f32)
                    .ok_or(AppError::Unavailable)?;

                Ok(VectorSearchResponseObject {
                    context_object_id,
                    score,
                })
            })
            .collect()
    }
}

fn parse_uuid_value(value: &Value) -> Option<Uuid> {
    match value {
        Value::String(raw) => Uuid::parse_str(raw).ok(),
        _ => None,
    }
}
