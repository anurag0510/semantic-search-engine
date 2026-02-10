use std::fmt;

/// Custom error types for the vector worker
#[derive(Debug)]
pub enum WorkerError {
    ModelInitError(String),
    InferenceError(String),
    KafkaError(String),
    SerializationError(String),
}

impl fmt::Display for WorkerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkerError::ModelInitError(e) => write!(f, "Model initialization error: {}", e),
            WorkerError::InferenceError(e) => write!(f, "Inference error: {}", e),
            WorkerError::KafkaError(e) => write!(f, "Kafka error: {}", e),
            WorkerError::SerializationError(e) => write!(f, "Serialization error: {}", e),
        }
    }
}

impl std::error::Error for WorkerError {}

impl From<rdkafka::error::KafkaError> for WorkerError {
    fn from(err: rdkafka::error::KafkaError) -> Self {
        WorkerError::KafkaError(err.to_string())
    }
}

impl From<serde_json::Error> for WorkerError {
    fn from(err: serde_json::Error) -> Self {
        WorkerError::SerializationError(err.to_string())
    }
}
