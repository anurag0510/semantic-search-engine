use std::fmt;

/// Custom error types for the indexing worker
#[derive(Debug)]
pub enum IndexerError {
    QdrantConnectionError(String),
    QdrantUpsertError(String),
    KafkaError(String),
    SerializationError(String),
}

impl fmt::Display for IndexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IndexerError::QdrantConnectionError(e) => write!(f, "Qdrant connection error: {}", e),
            IndexerError::QdrantUpsertError(e) => write!(f, "Qdrant upsert error: {}", e),
            IndexerError::KafkaError(e) => write!(f, "Kafka error: {}", e),
            IndexerError::SerializationError(e) => write!(f, "Serialization error: {}", e),
        }
    }
}

impl std::error::Error for IndexerError {}

impl From<rdkafka::error::KafkaError> for IndexerError {
    fn from(err: rdkafka::error::KafkaError) -> Self {
        IndexerError::KafkaError(err.to_string())
    }
}

impl From<serde_json::Error> for IndexerError {
    fn from(err: serde_json::Error) -> Self {
        IndexerError::SerializationError(err.to_string())
    }
}
