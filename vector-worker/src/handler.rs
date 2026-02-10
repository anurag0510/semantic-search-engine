use crate::config::Config;
use crate::error::WorkerError;
use crate::vectorizer::{generate_embedding, SharedModel};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use shared_types::{DocumentReceivedEvent, DocumentVectorizedEvent};
use std::time::Duration;
use uuid::Uuid;

/// Process a single document event
/// 
/// 1. Generates embedding using the ML model
/// 2. Creates vectorized event
/// 3. Publishes to output Kafka topic
pub async fn process_document(
    event: DocumentReceivedEvent,
    model: SharedModel,
    producer: &FutureProducer,
    config: &Config,
) -> Result<(), WorkerError> {
    let doc_id = event.payload.id;
    let content = event.payload.content;

    tracing::info!(%doc_id, "Processing document");

    // Generate embedding
    let vector = generate_embedding(model, content).await?;
    tracing::info!(%doc_id, dim = vector.len(), "Vector generated successfully");

    // Publish vectorized event
    publish_vectorized_event(producer, config, doc_id, vector).await?;
    tracing::info!(%doc_id, "Published vectorized event");

    Ok(())
}

/// Publish a vectorized document event to Kafka
async fn publish_vectorized_event(
    producer: &FutureProducer,
    config: &Config,
    doc_id: Uuid,
    vector: Vec<f32>,
) -> Result<(), WorkerError> {
    let output_event = DocumentVectorizedEvent { id: doc_id, vector };

    let payload = serde_json::to_vec(&output_event)?;
    let doc_id_str = doc_id.to_string();

    let record = FutureRecord::to(&config.output_topic)
        .key(&doc_id_str)
        .payload(&payload);

    let timeout = Timeout::After(Duration::from_secs(config.kafka_send_timeout_secs));

    producer
        .send(record, timeout)
        .await
        .map_err(|(e, _)| WorkerError::KafkaError(e.to_string()))?;

    Ok(())
}
