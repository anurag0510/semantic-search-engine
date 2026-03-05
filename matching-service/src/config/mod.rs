use serde::Deserialize;
use std::fs;

/// Configuration for the matching service
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub qdrant_grpc_url: String,
    pub collection_name: String,
    pub server_host: String,
    pub server_port: u16,
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
            qdrant_grpc_url: "http://localhost:6334".to_string(),
            collection_name: "resumes".to_string(),
            server_host: "0.0.0.0".to_string(),
            server_port: 3001,
        }
    }
}
