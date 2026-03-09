use crate::config::Config;
use qdrant_client::Qdrant;
use rust_bert::pipelines::sentence_embeddings::SentenceEmbeddingsModel;
use std::sync::Mutex;

/// Shared application state holding heavy clients
pub struct AppState {
    /// Mutex needed for thread-safe access to the C++ LibTorch backend
    pub model: Mutex<SentenceEmbeddingsModel>,
    pub qdrant: Qdrant,
    pub config: Config,
}

impl AppState {
    pub fn new(model: SentenceEmbeddingsModel, qdrant: Qdrant, config: Config) -> Self {
        Self {
            model: Mutex::new(model),
            qdrant,
            config,
        }
    }
}
