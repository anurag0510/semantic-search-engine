use crate::{config::Config, error::ApiError};
use rdkafka::{
    config::ClientConfig,
    producer::{FutureProducer, FutureRecord},
    util::Timeout,
};
use shared_types::{DocumentPayload, DocumentReceivedEvent};
use std::time::Duration;

/// Initialize and return a configured Kafka producer
pub fn create_producer(config: &Config) -> Result<FutureProducer, rdkafka::error::KafkaError> {
    ClientConfig::new()
        .set("bootstrap.servers", config.kafka_broker.to_string())
        .set("message.timeout.ms", config.kafka_timeout_ms.to_string())
        .create()
}

/// Publish a document event to Kafka
pub async fn publish_document_event(
    producer: &FutureProducer,
    config: &Config,
    payload: DocumentPayload,
) -> Result<(), ApiError> {
    let doc_id = payload.id.to_string();

    tracing::info!(%doc_id, "Publishing document to Kafka");

    // Wrap payload in event structure
    let event = DocumentReceivedEvent { payload };

    // Serialize event to JSON bytes
    let payload_bytes =
        serde_json::to_vec(&event).map_err(|e| ApiError::SerializationError(e.to_string()))?;

    // Construct Kafka record with document ID as key for partition ordering
    let record = FutureRecord::to(config.kafka_topic.as_str())
        .key(&doc_id)
        .payload(&payload_bytes);

    // Send to Kafka with timeout
    let timeout = Timeout::After(Duration::from_secs(config.kafka_send_timeout_secs));
    producer.send(record, timeout).await.map_err(|(e, _)| {
        tracing::error!(%doc_id, error = %e, "Failed to publish to Kafka");
        ApiError::KafkaPublishError(e.to_string())
    })?;

    tracing::info!(%doc_id, "Successfully published to Kafka");
    Ok(())
}
