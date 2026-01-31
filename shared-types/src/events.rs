use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::{DenseVector, DocumentPayload};

// ==========================================
// Event Definitions
// ==========================================

/// Event: Emitted by the Ingestion API when valid text is received.
/// Consumer: Vectorization Worker.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DocumentReceivedEvent {
    pub payload: DocumentPayload,
}

/// Event: Emitted by the Vectorization Worker after successful ML inference.
/// Consumer: Indexing Worker.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DocumentVectorizedEvent {
    /// The ID of the original document.
    pub id: Uuid,
    /// The generated resulting vector embedding.
    pub vector: DenseVector,
}
