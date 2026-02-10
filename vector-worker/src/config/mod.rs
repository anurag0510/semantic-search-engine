use serde::Deserialize;
use std::fs;

/// Configuration for the vector worker
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub kafka_broker: String,
    pub input_topic: String,
    pub output_topic: String,
    pub consumer_group: String,
    pub kafka_timeout_ms: String,
    pub kafka_send_timeout_secs: u64,
}

impl Config {
    /// Load configuration from a YAML file
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config: Config = yaml_serde::from_str(&contents)?;
        Ok(config)
    }

    /// Default configuration for development
    pub fn default_config() -> Self {
        Self {
            kafka_broker: "localhost:9092".to_string(),
            input_topic: "resume_received".to_string(),
            output_topic: "resume_vectorized".to_string(),
            consumer_group: "vectorizer_group_v1".to_string(),
            kafka_timeout_ms: "5000".to_string(),
            kafka_send_timeout_secs: 5,
        }
    }
}
