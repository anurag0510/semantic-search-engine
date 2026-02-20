use serde::Deserialize;
use std::fs;

/// Configuration for the indexing worker
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub kafka_broker: String,
    pub input_topic: String,
    pub consumer_group: String,
    pub qdrant_grpc_url: String,
    pub collection_name: String,
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
            input_topic: "resume_vectorized".to_string(),
            consumer_group: "indexer_group_v1".to_string(),
            qdrant_grpc_url: "http://localhost:6334".to_string(),
            collection_name: "resumes".to_string(),
        }
    }
}
