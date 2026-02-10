use crate::error::WorkerError;
use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsBuilder, SentenceEmbeddingsModel, SentenceEmbeddingsModelType,
};
use std::sync::{Arc, Mutex};

/// Type alias for thread-safe model reference
pub type SharedModel = Arc<Mutex<SentenceEmbeddingsModel>>;

/// Initialize the sentence embeddings model
/// 
/// Loads the AllMiniLmL6V2 model which provides a good balance of:
/// - Fast inference speed
/// - Small memory footprint  
/// - Good quality embeddings (384 dimensions)
pub async fn init_model() -> Result<SharedModel, WorkerError> {
    tracing::info!("Initializing ML model (this may take time on first run)...");

    let model = tokio::task::spawn_blocking(|| {
        SentenceEmbeddingsBuilder::remote(SentenceEmbeddingsModelType::AllMiniLmL6V2)
            .create_model()
    })
    .await
    .map_err(|e| WorkerError::ModelInitError(e.to_string()))?
    .map_err(|e| WorkerError::ModelInitError(e.to_string()))?;

    tracing::info!("ML model loaded successfully");

    Ok(Arc::new(Mutex::new(model)))
}

/// Generate embeddings for the given text
/// 
/// Runs inference in a blocking task to avoid blocking the async runtime.
/// Returns a vector of f32 values representing the text embedding.
pub async fn generate_embedding(
    model: SharedModel,
    text: String,
) -> Result<Vec<f32>, WorkerError> {
    let vector_result = tokio::task::spawn_blocking(move || {
        let model_guard = model.lock().expect("Model mutex poisoned");
        model_guard.encode(&[text])
    })
    .await
    .map_err(|e| WorkerError::InferenceError(e.to_string()))?
    .map_err(|e| WorkerError::InferenceError(e.to_string()))?;

    vector_result
        .into_iter()
        .next()
        .ok_or_else(|| WorkerError::InferenceError("No vector returned from model".to_string()))
}
