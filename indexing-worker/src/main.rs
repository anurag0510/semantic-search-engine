mod config;
mod error;
mod handler;
mod kafka;
mod qdrant;

use config::Config;
use rdkafka::consumer::Consumer;
use rdkafka::message::Message;
use shared_types::DocumentVectorizedEvent;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let config = Config::from_file("indexing-worker/config.yaml").unwrap_or_else(|_| {
        tracing::warn!("Failed to load config.yaml, using defaults");
        Config::default_config()
    });

    let qdrant_client = qdrant::create_client(&config).await?;
    let consumer = kafka::create_consumer(&config)?;

    consumer.subscribe(&[&config.input_topic])?;
    tracing::info!(
        topic = %config.input_topic,
        broker = %config.kafka_broker,
        "Indexing Worker started. Listening for vectors..."
    );

    loop {
        match consumer.recv().await {
            Err(e) => tracing::warn!(error = %e, "Kafka receive error"),
            Ok(message) => {
                if let Some(payload_bytes) = message.payload() {
                    match serde_json::from_slice::<DocumentVectorizedEvent>(payload_bytes) {
                        Ok(event) => {
                            let doc_id = event.id;
                            if let Err(e) =
                                handler::process_vector_event(event, &qdrant_client, &config).await
                            {
                                tracing::error!(%doc_id, error = %e, "Failed to index document");
                            }
                        }
                        Err(e) => {
                            tracing::error!(error = %e, "Failed to deserialize message payload");
                        }
                    }
                }
            }
        }
    }
}