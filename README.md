# semantic-search-engine

This project enables matching unstructured text based on conceptual meaning rather than keyword overlap. It is designed as an event-driven pipeline optimized for low latency ingestion, scalable vector processing, and fast Approximate Nearest Neighbor (ANN) search.

## Prerequisites

- Podman and Podman Compose (Compatible with Docker and Docker Compose as well)
- Rust toolchain (for building the application)
- macOS/Linux environment (Didn't test on Windows, most probably there should be a workaround if not straightforward)
- On first run, `vector-worker` and `matching-service` will automatically download the `AllMiniLmL6V2` BERT model (~90 MB) via `rust-bert`

## Architecture Overview

The system is a multi-stage event-driven pipeline:

```
Client
  │
  ▼
ingestion-api (:3000)     POST /submit  →  DocumentPayload (JSON)
  │  wraps into DocumentReceivedEvent
  ▼
Kafka topic: resume_received
  │
  ▼
vector-worker             AllMiniLmL6V2 BERT model (384-dim embeddings)
  │  produces DocumentVectorizedEvent { id: Uuid, vector: Vec<f32> }
  ▼
Kafka topic: resume_vectorized
  │
  ▼
indexing-worker           upserts vectors into Qdrant (collection: "resumes")

Client
  │
  ▼
matching-service (:3001)  POST /match  →  MatchQuery { content, top_k }
  │  vectorizes query with same AllMiniLmL6V2 model
  ▼
Qdrant ANN search  →  MatchResponse { matches: [{ id: Uuid, score: f32 }] }
```

**Infrastructure Components:**
- **Kafka** - Event streaming platform for inter-service communication
- **Qdrant** - High-performance vector database for ANN search

## Project Structure

```
semantic-search-engine/
├── shared-types/        # Shared event and type definitions used across all services
├── ingestion-api/       # HTTP API — receives documents and publishes to Kafka
├── vector-worker/       # Kafka consumer/producer — generates BERT embeddings
├── indexing-worker/     # Kafka consumer — upserts vectors into Qdrant
├── matching-service/    # HTTP API — semantic search via Qdrant ANN queries
├── docker-compose.yaml  # Infrastructure services (Kafka, Zookeeper, Qdrant)
└── Cargo.toml           # Cargo workspace configuration
```

## Quick Start

### 1. Start Infrastructure Services

```bash
# Start Kafka, Zookeeper, and Qdrant
podman compose up -d

# Verify services are running
podman ps
```

### 2. Create Kafka Topics

```bash
# Topic for raw document ingestion
podman exec -it kafka-broker kafka-topics --create \
  --topic resume_received \
  --bootstrap-server localhost:9092 \
  --replication-factor 1 \
  --partitions 5

# Topic for vectorized documents
podman exec -it kafka-broker kafka-topics --create \
  --topic resume_vectorized \
  --bootstrap-server localhost:9092 \
  --replication-factor 1 \
  --partitions 5

# Verify topic creation
podman exec -it kafka-broker kafka-topics --list \
  --bootstrap-server localhost:9092
```

### 3. Create Qdrant Collection

The Qdrant collection must be created before starting the indexing-worker or matching-service. Use the REST API or the Qdrant dashboard at `http://localhost:6333/dashboard`:

```bash
curl -X PUT http://localhost:6333/collections/resumes \
  -H 'Content-Type: application/json' \
  -d '{
    "vectors": {
      "size": 384,
      "distance": "Cosine"
    }
  }'
```

### 4. Run the Services

Each service reads its configuration from `config.yaml` in its own directory. Run each in a separate terminal:

```bash
# Ingestion API — listens on :3000
cargo run -p ingestion-api

# Vector worker — consumes from resume_received, produces to resume_vectorized
# Note: downloads AllMiniLmL6V2 model on first run (~90 MB)
cargo run -p vector-worker

# Indexing worker — consumes from resume_vectorized, upserts into Qdrant
cargo run -p indexing-worker

# Matching service — listens on :3001
# Note: downloads AllMiniLmL6V2 model on first run (~90 MB)
cargo run -p matching-service
```

### 5. Use the API

```bash
# Ingest a document
curl -X POST http://localhost:3000/submit \
  -H 'Content-Type: application/json' \
  -d '{"id": "00000000-0000-0000-0000-000000000001", "content": "Experienced Rust engineer with distributed systems background"}'

# Search for semantically similar documents
curl -X POST http://localhost:3001/match \
  -H 'Content-Type: application/json' \
  -d '{"content": "systems programming expert", "top_k": 5}'
```

### Stopping Infrastructure

```bash
# Stop all services
podman compose down

# Stop and remove volumes (clears all data)
podman compose down -v
```

## Service Endpoints

| Service | Endpoint | Description |
|---------|----------|-------------|
| Ingestion API | `http://localhost:3000` | `POST /submit` — ingest a document; `GET /health` |
| Matching Service | `http://localhost:3001` | `POST /match` — semantic search; `GET /health` |
| Kafka Broker | `localhost:9092` | External listener for host-side clients |
| Qdrant HTTP API | `http://localhost:6333` | REST API |
| Qdrant Web UI | `http://localhost:6333/dashboard` | Visual collection browser |
| Qdrant gRPC | `localhost:6334` | gRPC API (used by indexing-worker and matching-service) |

## Current Implementation

### shared-types

Common data structures and event definitions shared across all services:

- `DocumentPayload { id: Uuid, content: String }` — core document entity
- `DenseVector = Vec<f32>` — type alias for embedding vectors
- `DocumentReceivedEvent { payload: DocumentPayload }` — ingestion-api → vector-worker
- `DocumentVectorizedEvent { id: Uuid, vector: DenseVector }` — vector-worker → indexing-worker
- `MatchQuery { content: String, top_k: u64 }` — matching-service request body
- `MatchResult { id: Uuid, score: f32 }` / `MatchResponse { matches: Vec<MatchResult> }` — matching-service response

See [shared-types/README.md](shared-types/README.md) for full details.

### ingestion-api

Axum HTTP server (port 3000). Accepts `POST /submit` with a `DocumentPayload` JSON body, wraps it in a `DocumentReceivedEvent`, and publishes it to the `resume_received` Kafka topic using `rdkafka`. Returns `202 Accepted` on success.

### vector-worker

Kafka consumer/producer. Consumes `DocumentReceivedEvent` messages from `resume_received`, generates 384-dimensional sentence embeddings using the `AllMiniLmL6V2` model via `rust-bert`, and publishes `DocumentVectorizedEvent` messages to `resume_vectorized`. The model is loaded once at startup and shared across events via `Arc<Mutex<SentenceEmbeddingsModel>>`.

### indexing-worker

Kafka consumer (terminal stage — no producer). Consumes `DocumentVectorizedEvent` messages from `resume_vectorized` and upserts each vector into Qdrant via gRPC. Uses UUID strings as point IDs to avoid precision loss. Verifies the Qdrant collection exists at startup and fails fast if it does not.

### matching-service

Axum HTTP server (port 3001). Accepts `POST /match` with a `MatchQuery { content, top_k }` body. Vectorizes the query using the same `AllMiniLmL6V2` model (ensuring vectors are in the same embedding space as indexed documents), runs an ANN search against Qdrant, and returns a `MatchResponse` with ranked results and cosine similarity scores.

## Building

```bash
# Build all crates in the workspace
cargo build

# Build a specific service
cargo build -p ingestion-api

# Run tests
cargo test
```

## Future Improvements

- [ ] Add observability (metrics, distributed tracing)
- [ ] Add comprehensive integration tests
- [ ] Add Docker containers for application services in `docker-compose.yaml`
- [ ] Add Kubernetes manifests for production deployment
- [ ] Store document content as payload in Qdrant alongside vectors (currently only vectors are stored)
- [ ] Add automatic Qdrant collection creation on startup