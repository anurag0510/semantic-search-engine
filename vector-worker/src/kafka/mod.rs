use crate::config::Config;
use crate::error::WorkerError;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::StreamConsumer;
use rdkafka::producer::FutureProducer;

/// Create a Kafka consumer configured for the vector worker
pub fn create_consumer(config: &Config) -> Result<StreamConsumer, WorkerError> {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", &config.kafka_broker)
        .set("group.id", &config.consumer_group)
        .set("enable.auto.commit", "true")
        .set("auto.offset.reset", "earliest")
        .create()?;

    Ok(consumer)
}

/// Create a Kafka producer for publishing vectorized events
pub fn create_producer(config: &Config) -> Result<FutureProducer, WorkerError> {
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", &config.kafka_broker)
        .set("message.timeout.ms", &config.kafka_timeout_ms)
        .create()?;

    Ok(producer)
}
