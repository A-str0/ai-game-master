use application::services::{
    VectorSearchQuery, VectorSearchResponseObject, VectorSearcher, VectorSearcherError,
    VectorSearcherResult, VectorUpsertQuery,
};
use qdrant_client::{
    Payload, Qdrant, QdrantError,
    qdrant::{
        CreateCollectionBuilder, Distance, PointStruct, SearchPointsBuilder, UpsertPointsBuilder,
        VectorParamsBuilder, point_id::PointIdOptions, value::Kind,
    },
};
use uuid::Uuid;

trait QdrantResultExt<T> {
    fn into_vector_searcher(self) -> VectorSearcherResult<T>;
}

impl<T> QdrantResultExt<T> for Result<T, QdrantError> {
    fn into_vector_searcher(self) -> VectorSearcherResult<T> {
        self.map_err(|error| VectorSearcherError::Unavailable {
            details: error.to_string(),
        })
    }
}

pub struct QdSearchService {
    client: Option<Qdrant>,
    init_error: Option<String>,
    embedding_size: u64,
}

impl QdSearchService {
    pub fn new() -> Self {
        let grpc_url = qdrant_grpc_url();
        let embedding_size = std::env::var("EMBEDDING_DIMENSION")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(2048);
        let api_key = std::env::var("QDRANT_API_KEY").ok();
        let client = Qdrant::from_url(&grpc_url)
            .api_key(api_key)
            .skip_compatibility_check()
            .build();

        let (client, init_error) = match client {
            Ok(client) => (Some(client), None),
            Err(error) => (
                None,
                Some(format!(
                    "failed to initialize qdrant client for {grpc_url}: {error}"
                )),
            ),
        };

        Self {
            client,
            init_error,
            embedding_size,
        }
    }

    fn collection_name(&self, session_id: Uuid) -> String {
        format!("session_{}", session_id.simple())
    }

    fn client(&self) -> VectorSearcherResult<&Qdrant> {
        self.client
            .as_ref()
            .ok_or_else(|| VectorSearcherError::Unavailable {
                details: self.init_error.clone().unwrap_or_else(|| {
                    String::from("qdrant client is unavailable for unknown reasons")
                }),
            })
    }

    async fn ensure_collection_internal(&self, session_id: Uuid) -> VectorSearcherResult<()> {
        let collection_name = self.collection_name(session_id);
        let client = self.client()?;

        if client
            .collection_exists(&collection_name)
            .await
            .into_vector_searcher()?
        {
            return Ok(());
        }

        let create_result = client
            .create_collection(
                CreateCollectionBuilder::new(&collection_name).vectors_config(
                    VectorParamsBuilder::new(self.embedding_size, Distance::Cosine),
                ),
            )
            .await;

        match create_result {
            Ok(_) => Ok(()),
            Err(error) => {
                if client
                    .collection_exists(&collection_name)
                    .await
                    .unwrap_or(false)
                {
                    Ok(())
                } else {
                    Err(VectorSearcherError::Unavailable {
                        details: error.to_string(),
                    })
                }
            }
        }
    }
}

#[async_trait::async_trait]
impl VectorSearcher for QdSearchService {
    async fn ensure_session_collection(
        &self,
        session_id: domain::value_objects::GameSessionId,
    ) -> VectorSearcherResult<()> {
        self.ensure_collection_internal(session_id.0).await
    }

    async fn upsert(&self, query: VectorUpsertQuery) -> VectorSearcherResult<()> {
        self.ensure_collection_internal(query.session_id.0).await?;
        let client = self.client()?;
        let collection_name = self.collection_name(query.session_id.0);
        let point = PointStruct::new(
            query.context_object_id.0.to_string(),
            query.embedding,
            Payload::from([(
                "context_object_id",
                query.context_object_id.0.to_string().into(),
            )]),
        );

        client
            .upsert_points(UpsertPointsBuilder::new(&collection_name, vec![point]).wait(true))
            .await
            .into_vector_searcher()?;

        Ok(())
    }

    async fn search(
        &self,
        query: VectorSearchQuery,
    ) -> VectorSearcherResult<Vec<VectorSearchResponseObject>> {
        let collection_name = self.collection_name(query.session_id.0);
        let client = self.client()?;

        if !client
            .collection_exists(&collection_name)
            .await
            .into_vector_searcher()?
        {
            return Ok(Vec::new());
        }

        let response = client
            .search_points(
                SearchPointsBuilder::new(&collection_name, query.embedding, query.k.into())
                    .with_payload(true),
            )
            .await
            .into_vector_searcher()?;

        response
            .result
            .iter()
            .map(|entry| {
                let context_object_id = parse_context_object_id(entry)?;

                Ok(VectorSearchResponseObject {
                    context_object_id,
                    score: entry.score,
                })
            })
            .collect()
    }
}

fn parse_context_object_id(
    point: &qdrant_client::qdrant::ScoredPoint,
) -> VectorSearcherResult<domain::value_objects::ContextObjectId> {
    parse_uuid_payload_value(point.try_get("context_object_id"))
        .or_else(|| {
            point
                .id
                .as_ref()
                .and_then(|id| match id.point_id_options.as_ref() {
                    Some(PointIdOptions::Uuid(raw)) => Uuid::parse_str(raw).ok(),
                    _ => None,
                })
        })
        .map(domain::value_objects::ContextObjectId)
        .ok_or_else(|| VectorSearcherError::InvalidResponse {
            details: format!("qdrant search hit is missing a valid context_object_id: {point:?}"),
        })
}

fn parse_uuid_payload_value(value: Option<&qdrant_client::qdrant::Value>) -> Option<Uuid> {
    match value.and_then(|value| value.kind.as_ref()) {
        Some(Kind::StringValue(raw)) => Uuid::parse_str(raw).ok(),
        _ => None,
    }
}

fn qdrant_grpc_url() -> String {
    if let Ok(explicit_url) = std::env::var("QDRANT_GRPC_URL") {
        return explicit_url.trim_end_matches('/').to_owned();
    }

    let raw_url = std::env::var("QDRANT_URL")
        .unwrap_or_else(|_| String::from("http://127.0.0.1:6333"))
        .trim_end_matches('/')
        .to_owned();

    let Ok(mut parsed_url) = reqwest::Url::parse(&raw_url) else {
        return raw_url;
    };

    match parsed_url.port() {
        Some(6333) => {
            let _ = parsed_url.set_port(Some(6334));
        }
        None => {
            let _ = parsed_url.set_port(Some(6334));
        }
        Some(_) => {}
    }

    parsed_url.to_string().trim_end_matches('/').to_owned()
}
