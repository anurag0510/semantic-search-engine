# semantic-search-engine

This project enables matching unstructured text based on conceptual meaning rather than keyword overlap. It is designed as an event-driven pipeline optimized for low latency ingestion, scalable vector processing, and fast Approximate Nearest Neighbor (ANN) search.

## 🚧 Project Status

This project is in **early development**. The foundational type system and event architecture are defined, but the core services are not yet implemented.

## Prerequisites

- Podman and Podman Compose (Compatable with Docker and Docker Compose as well)
- Rust toolchain (for building the application)
- macOS/Linux environment (Didn't test on Windows most probably there should be a workaround if not straight forward)

## Architecture Overview

The system is designed as a multi-stage event-driven pipeline:

1. **Ingestion API** - Receives raw text documents and emits `DocumentReceivedEvent` to Kafka
2. **Vectorization Worker** - Consumes document events from Kafka, generates embeddings, and emits `DocumentVectorizedEvent`
3. **Indexing Worker** - Consumes vectorized events and maintains an ANN search index in Qdrant
4. **Query API** - Provides semantic search capabilities over indexed vectors in Qdrant

**Infrastructure Components:**
- **Kafka** - Event streaming platform for inter-service communication
- **Qdrant** - High-performance vector database for ANN search

## Project Structure
```
semantic-search-engine/
├── shared-types/        # Shared type definitions (see shared-types/README.md)
├── docker-compose.yaml  # Docker Compose configuration for infrastructure services
└── Cargo.toml          # Workspace configuration
```

## Quick Start

### Starting Infrastructure Services

Before running the application, start the required infrastructure services:

```bash
# Start all services (Kafka, Zookeeper, Qdrant)
podman compose up -d

# Verify services are running
podman ps

# Create Kafka topic for document ingestion
podman exec -it kafka-broker kafka-topics --create \
  --topic resume_received \
  --bootstrap-server localhost:9092 \
  --replication-factor 1 \
  --partitions 5

# Verify topic creation
podman exec -it kafka-broker kafka-topics --describe \
  --topic resume_received \
  --bootstrap-server localhost:9092
```

### Stopping Infrastructure

```bash
# Stop all services
podman compose down

# Stop and remove volumes (clears all data)
podman compose down -v
```

### Service Endpoints

- **Kafka Broker**: `localhost:9092`
- **Qdrant HTTP API**: `http://localhost:6333`
- **Qdrant Web UI**: `http://localhost:6333/dashboard`
- **Qdrant gRPC**: `localhost:6334`

## Current Implementation

### Shared Types

The `shared-types` crate provides common data structures and event definitions used across all pipeline components. For detailed information about types, events, and usage, see [shared-types/README.md](shared-types/README.md)

## Next Steps (It's a plan items might change in future scope)

- [ ] Implement Ingestion API service with Kafka producer
- [ ] Implement Vectorization Worker with ML model integration (Kafka consumer/producer)
- [ ] Implement Indexing Worker with Qdrant client (Kafka consumer)
- [ ] Implement Query API for semantic search (Qdrant queries)
- [ ] Configure Kafka topics and consumer groups
- [ ] Add observability (metrics, tracing)
- [ ] Add comprehensive testing
- [ ] Add Docker containerization for application services
- [ ] Add Kubernetes manifests for production deployment

## Building

```bash
# 1. Start infrastructure services first
podman compose up -d

# 2. Build the workspace
cargo build

# 3. Run the main application
cargo run

# 4. Run tests
cargo test