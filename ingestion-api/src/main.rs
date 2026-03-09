mod config;
mod error;
mod handlers;
mod kafka;
mod routes;
mod state;

use config::Config;
use state::AppState;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Load configuration
    let config = Config::from_file("config.yaml").unwrap_or_else(|_| Config::default_config());

    let server_host: IpAddr = config.server_host.parse()?;
    let server_port = config.server_port;

    // Initialize Kafka producer
    let producer = kafka::create_producer(&config)?;
    tracing::info!("Kafka Producer initialized successfully");

    // Create application state with config included
    let state = Arc::new(AppState::new(producer, config));

    // Build router with state and config
    let app = routes::create_router(state);

    // Start server
    let addr = SocketAddr::from((server_host, server_port));
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
