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

/// Request payload for searching candidates.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MatchQuery {
    /// The job description text to match against.
    pub content: String,
    /// How many top results to return (e.g., 10).
    pub top_k: u64,
}

/// A single matched candidate result.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MatchResult {
    /// The candidate document ID.
    pub id: Uuid,
    /// The similarity score (higher is better for Cosine).
    pub score: f32,
}

/// The response payload containing ranked matches.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MatchResponse {
    pub matches: Vec<MatchResult>,
}