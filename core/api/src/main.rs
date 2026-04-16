#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = api::ApiConfig::from_env()?;
    let server = api::bootstrap_api_server(config).await?;

    server.serve().await
}
