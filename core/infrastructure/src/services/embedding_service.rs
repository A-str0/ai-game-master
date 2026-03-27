use std::sync::Arc;

use application::services::{
    Embedder, EmbedderError, EmbedderQuery, EmbedderResponse, EmbedderResult,
};
use autoagents::{
    llm::{backends::openai::OpenAI, embedding::EmbeddingProvider},
    prelude::*,
};
use autoagents_derive::{AgentHooks, agent};

#[derive(Default, Clone, Copy, AgentHooks)]
#[agent(name = "Embedding Agent", description = "==PLACEHOLDER==")]
pub struct EmbeddingAgent;

pub struct OpenRouterEmbeddingService {
    handle: Arc<dyn EmbeddingProvider>,
}

impl OpenRouterEmbeddingService {
    pub async fn new() -> Result<Self, Error> {
        let api_key = std::env::var("LLM_API_KEY").unwrap_or_else(|_| String::from("API_KEY")); // TODO: change enviromental variable
        let model = std::env::var("EMBEDDING_MODEL")
            .unwrap_or_else(|_| String::from("nvidia/llama-nemotron-embed-vl-1b-v2:free"));

        let provider: Arc<OpenAI> = LLMBuilder::<OpenAI>::new()
            .api_key(api_key)
            .model(model)
            .build()?;

        let handle = provider;

        Ok(Self { handle })
    }
}

#[async_trait::async_trait]
impl Embedder for OpenRouterEmbeddingService {
    async fn create_embedding(&self, query: EmbedderQuery) -> EmbedderResult<EmbedderResponse> {
        let vectors = self.handle.embed(vec![query.text]).await.map_err(|error| {
            EmbedderError::Unavailable {
                details: format!("openrouter embedding request failed: {error}"),
            }
        })?;
        let vector = vectors
            .into_iter()
            .next()
            .ok_or(EmbedderError::InvalidResponse {
                details: String::from("embedding backend returned an empty vector list"),
            })?;

        Ok(EmbedderResponse { vector })
    }
}
