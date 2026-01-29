use crate::config::Config;
use rdkafka::producer::FutureProducer;

/// Shared application state containing dependencies
#[derive(Clone)]
pub struct AppState {
    pub producer: FutureProducer,
    pub config: Config,
}

impl AppState {
    pub fn new(producer: FutureProducer, config: Config) -> Self {
        Self { producer, config }
    }
}
