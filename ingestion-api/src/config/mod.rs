use serde::Deserialize;
use std::fs;

/// Configuration for the ingestion API
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub kafka_topic: String,
    pub kafka_broker: String,
    pub server_host: String,
    pub server_port: u16,
    pub kafka_timeout_ms: String,
    pub kafka_send_timeout_secs: u64,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config: Config = yaml_serde::from_str(&contents)?;
        Ok(config)
    }

    // Keep default for fallback
    pub fn default_config() -> Self {
        Self {
            kafka_topic: "resume_received".to_string(),
            kafka_broker: "127.0.0.1:9092".to_string(),
            server_host: "0.0.0.0".to_string(),
            server_port: 3000,
            kafka_timeout_ms: "5000".to_string(),
            kafka_send_timeout_secs: 2,
        }
    }
}
