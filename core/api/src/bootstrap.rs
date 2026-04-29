use std::sync::Arc;

use anyhow::{Context, Result};
use application::{
    ports::{
        ContextObjectRepository, Embedder, GameSessionRepository, MemoryExtractorPort,
        MessageRepository, NarratorPort, UnitOfWorkFactory, VectorSearcher,
    },
    services::{
        AgentOrchestrationService, Clock, IdGenerator, PromptAssembler, PromptAssemblyService,
        RetrivialService, RetrivialServicePort, UtcClock, UuidGenerator,
    },
};
use axum::Router;
use infrastructure::{
    adapters::{
        EmbeddingAdapter, OpenRouterMemoryExtractorAdapter, OpenRouterNarratorAdapter,
        QdSearchService,
    },
    repositories::connecion::PgDatabase,
};

use crate::{
    ApiConfig,
    app_service::{ApiApplicationService, LiveApiApplicationService},
    http::build_router,
};

/// Runnable Axum server with a bound router configuration.
pub struct ApiServer {
    bind_addr: String,
    router: Router,
}

impl ApiServer {
    /// Binds the configured TCP listener and starts serving HTTP traffic.
    pub async fn serve(self) -> Result<()> {
        let listener = tokio::net::TcpListener::bind(&self.bind_addr)
            .await
            .with_context(|| format!("failed to bind API listener on {}", self.bind_addr))?;

        axum::serve(listener, self.router)
            .await
            .context("api server failed")?;

        Ok(())
    }
}

/// Wires the full API stack from environment/configuration-driven dependencies.
pub async fn bootstrap_api_server(config: ApiConfig) -> Result<ApiServer> {
    let database = PgDatabase::new(&config.database_url).with_context(|| {
        format!(
            "failed to initialize postgres database from DATABASE_URL: {}",
            config.database_url
        )
    })?;

    let clock: Arc<dyn Clock> = Arc::new(UtcClock::new());
    let sessions_repo: Arc<dyn GameSessionRepository> = Arc::new(database.game_sessions());
    let messages_repo: Arc<dyn MessageRepository> = Arc::new(database.messages());
    let context_object_repo: Arc<dyn ContextObjectRepository> =
        Arc::new(database.context_objects());
    let unit_of_work: Arc<dyn UnitOfWorkFactory> = Arc::new(database.unit_of_work());
    let embedder: Arc<dyn Embedder> = Arc::new(EmbeddingAdapter::new().await?);
    let vector_searcher: Arc<dyn VectorSearcher> = Arc::new(QdSearchService::new());
    let retrivial: Arc<dyn RetrivialServicePort> = Arc::new(RetrivialService::new());
    let narrator: Arc<dyn NarratorPort> = Arc::new(OpenRouterNarratorAdapter::new().await?);
    let memory_extractor: Arc<dyn MemoryExtractorPort> =
        Arc::new(OpenRouterMemoryExtractorAdapter::new().await?);
    let agent_orchestration = Arc::new(AgentOrchestrationService::new(narrator, memory_extractor));
    let id_generator: Arc<dyn IdGenerator> = Arc::new(UuidGenerator);
    let prompt_assembly: Arc<dyn PromptAssembler> = Arc::new(PromptAssemblyService::new());

    let application: Arc<dyn ApiApplicationService> = Arc::new(LiveApiApplicationService::new(
        sessions_repo,
        messages_repo,
        context_object_repo,
        unit_of_work,
        retrivial,
        agent_orchestration,
        embedder,
        vector_searcher,
        clock,
        id_generator,
        prompt_assembly,
    ));

    Ok(ApiServer {
        bind_addr: config.bind_addr.clone(),
        router: build_router(application, config.authentication),
    })
}
