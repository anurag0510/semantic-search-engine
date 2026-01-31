use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type alias for a dense vector embedding.
/// Standard transformer models (like BERT-base) typically output 768 dimensions.
pub type DenseVector = Vec<f32>;

/// The core entity representing a resume or job description.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DocumentPayload {
    /// Unique ID generated at ingestion.
    pub id: Uuid,
    /// The raw text content to be embedded.
    pub content: String,
    // Future extensibility: Add metadata fields here (e.g., source, timestamp).
}
