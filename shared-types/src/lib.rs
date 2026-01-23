//! Shared types for the semantic search engine.
//! 
//! This crate provides common data structures used across
//! the ingestion, vectorization, and indexing pipeline.

mod types;
mod events;

// Re-export public types
pub use types::{DocumentPayload, DenseVector};
pub use events::{DocumentReceivedEvent, DocumentVectorizedEvent};