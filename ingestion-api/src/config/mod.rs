/// Configuration constants for the ingestion API
pub struct Config {
    pub kafka_topic: &'static str,
    pub kafka_broker: &'static str,
    pub server_host: [u8; 4],
    pub server_port: u16,
    pub kafka_timeout_ms: &'static str,
    pub kafka_send_timeout_secs: u64,
}

impl Config {
    pub fn new() -> Self {
        Self {
            kafka_topic: "resume_received",
            kafka_broker: "127.0.0.1:9092",
            server_host: [0, 0, 0, 0],
            server_port: 3000,
            kafka_timeout_ms: "5000",
            kafka_send_timeout_secs: 2,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}