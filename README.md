# semantic-search-engine

This project enables matching unstructured text based on conceptual meaning rather than keyword overlap. It is designed as an event-driven pipeline optimized for low latency ingestion, scalable vector processing, and fast Approximate Nearest Neighbor (ANN) search.

## 🚧 Project Status

This project is in **early development**. The foundational type system and event architecture are defined, but the core services are not yet implemented.

## Architecture Overview

The system is designed as a multi-stage event-driven pipeline:

1. **Ingestion API** - Receives raw text documents and emits `DocumentReceivedEvent`
2. **Vectorization Worker** - Consumes document events, generates embeddings, and emits `DocumentVectorizedEvent`
3. **Indexing Worker** - Consumes vectorized events and maintains an ANN search index
4. **Query API** - Provides semantic search capabilities over indexed vectors

## Project Structure
```
semantic-search-engine/
├── shared-types/        # Shared type definitions (see shared-types/README.md)
└── Cargo.toml          # Workspace configuration
```

## Current Implementation

### Shared Types

The `shared-types` crate provides common data structures and event definitions used across all pipeline components. For detailed information about types, events, and usage, see [shared-types/README.md](shared-types/README.md)

## Next Steps

- [ ] Implement Ingestion API service
- [ ] Implement Vectorization Worker with ML model integration
- [ ] Implement Indexing Worker with ANN search (e.g., HNSW, FAISS)
- [ ] Implement Query API for semantic search
- [ ] Add message broker integration (Kafka, Redis Streams, etc.)
- [ ] Add observability (metrics, tracing)
- [ ] Add comprehensive testing

## Building

```bash
# Build the workspace
cargo build

# Run the main application
cargo run