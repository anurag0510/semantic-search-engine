use crate::config::Config;
use crate::error::IndexerError;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::StreamConsumer;

/// Create a Kafka consumer configured for the indexing worker
pub fn create_consumer(config: &Config) -> Result<StreamConsumer, IndexerError> {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", &config.kafka_broker)
        .set("group.id", &config.consumer_group)
        .set("enable.auto.commit", "true")
        .set("auto.offset.reset", "earliest")
        .create()?;

    Ok(consumer)
}
