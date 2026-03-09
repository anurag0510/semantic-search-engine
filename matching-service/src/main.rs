mod config;
mod error;
mod handlers;
mod qdrant;
mod routes;
mod state;

use config::Config;
use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsBuilder, SentenceEmbeddingsModelType,
};
use state::AppState;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("Initializing Matching Service...");

    // Load configuration
    let config = Config::from_file("matching-service/config.yaml").unwrap_or_else(|_| {
        tracing::warn!("Failed to load config.yaml, using defaults");
        Config::default_config()
    });

    let server_host: IpAddr = config.server_host.parse()?;
    let server_port = config.server_port;

    // 1. Load ML Model (blocking operation at startup)
    tracing::info!("Loading BERT model (this may take a moment)...");
    let model = tokio::task::spawn_blocking(|| {
        SentenceEmbeddingsBuilder::remote(SentenceEmbeddingsModelType::AllMiniLmL6V2)
            .create_model()
            .expect("Failed to load ML model")
    })
    .await?;
    tracing::info!("BERT model loaded.");

    // 2. Initialize Qdrant client
    let qdrant_client = qdrant::create_client(&config).await?;
    tracing::info!("Qdrant client initialized.");

    // 3. Build shared state
    let state = Arc::new(AppState::new(model, qdrant_client, config));

    // 4. Build router and start server
    let app = routes::create_router(state);
    let addr = SocketAddr::from((server_host, server_port));
    tracing::info!("Matching Service listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}