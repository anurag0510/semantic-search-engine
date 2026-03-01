mod config;
mod error;
mod handler;
mod kafka;
mod vectorizer;

use config::Config;
use rdkafka::consumer::Consumer;
use rdkafka::message::Message;
use shared_types::DocumentReceivedEvent;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Load configuration
    let config = Config::from_file("vector-worker/config.yaml").unwrap_or_else(|_| {
        tracing::warn!("Failed to load config.yaml, using defaults");
        Config::default_config()
    });

    // Initialize ML model
    let model = vectorizer::init_model().await?;

    // Initialize Kafka consumer and producer
    let consumer = kafka::create_consumer(&config)?;
    let producer = kafka::create_producer(&config)?;

    consumer.subscribe(&[&config.input_topic])?;
    tracing::info!(
        topic = %config.input_topic,
        broker = %config.kafka_broker,
        "Worker started. Listening for events..."
    );

    // Main processing loop
    loop {
        match consumer.recv().await {
            Err(e) => {
                tracing::warn!(error = %e, "Kafka receive error");
            }
            Ok(message) => {
                let msg_key = message
                    .key_view::<str>()
                    .map(|r| r.unwrap_or("<invalid-key>"))
                    .unwrap_or("<no-key>");

                tracing::debug!(doc_id = %msg_key, "Received message");

                if let Some(payload_bytes) = message.payload() {
                    match serde_json::from_slice::<DocumentReceivedEvent>(payload_bytes) {
                        Ok(event) => {
                            let doc_id = event.payload.id;
                            if let Err(e) =
                                handler::process_document(event, model.clone(), &producer, &config)
                                    .await
                            {
                                tracing::error!(%doc_id, error = %e, "Failed to process document");
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
