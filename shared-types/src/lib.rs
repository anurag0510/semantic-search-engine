//! Shared types for the semantic search engine.
//!
//! This crate provides common data structures used across
//! the ingestion, vectorization, and indexing pipeline.

mod events;
mod types;

// Re-export public types
pub use events::{DocumentReceivedEvent, DocumentVectorizedEvent};
pub use types::{DenseVector, DocumentPayload};
