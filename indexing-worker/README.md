# Indexing Worker

The Indexing Worker is a Kafka consumer service that receives pre-computed vector embeddings and indexes them into **Qdrant** — a high-performance vector search engine. It is the final stage of the ingestion pipeline, making documents searchable via semantic similarity.

## Overview

The worker performs two core operations:
1. **Consume** — Reads vectorized document events from Kafka
2. **Index** — Upserts embedding vectors into a Qdrant collection via gRPC

## Architecture

```
Kafka (resume_vectorized)     Indexing Worker           Qdrant (resumes collection)
         │                          │                           │
         │  DocumentVectorizedEvent │                           │
         ├─────────────────────────>│                           │
         │                          │  gRPC UpsertPoints        │
         │                          ├──────────────────────────>│
         │                          │                           │
         │                          │         OK / Error        │
         │                          │<──────────────────────────│
         │                          │                           │
```

## Project Structure

```
indexing-worker/
├── Cargo.toml              # Dependencies
├── config.yaml             # Runtime configuration
├── README.md
└── src/
    ├── main.rs             # Entry point, event loop
    ├── config/mod.rs       # Configuration loading
    ├── error/mod.rs        # Custom error types
    ├── handler.rs          # Vector indexing logic
    ├── kafka/mod.rs        # Consumer setup
    └── qdrant/mod.rs       # Qdrant client initialization
```

### Module Responsibilities

| Module | Purpose |
|--------|---------|
| `main` | Initializes components, runs Kafka consumer loop |
| `config` | Loads YAML config with defaults fallback |
| `error` | `IndexerError` enum with error conversions |
| `handler` | Constructs Qdrant points and performs upserts |
| `kafka` | Creates Kafka consumer |
| `qdrant` | Initializes gRPC client, verifies collection exists |

## Configuration

### Config File (`config.yaml`)

```yaml
kafka_broker: "localhost:9092"
input_topic: "resume_vectorized"
consumer_group: "indexer_group_v1"
qdrant_grpc_url: "http://localhost:6334"
collection_name: "resumes"
```

### Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `kafka_broker` | String | `localhost:9092` | Kafka bootstrap server |
| `input_topic` | String | `resume_vectorized` | Topic to consume from |
| `consumer_group` | String | `indexer_group_v1` | Kafka consumer group ID |
| `qdrant_grpc_url` | String | `http://localhost:6334` | Qdrant gRPC endpoint |
| `collection_name` | String | `resumes` | Target Qdrant collection |

> **Note:** Qdrant exposes two ports — `6333` for REST and `6334` for gRPC. This worker uses the **gRPC** endpoint for better performance.

## Qdrant Collection Setup

Before starting the indexing worker, you must create the target collection in Qdrant. This is a one-time setup step.

### Create the Collection

```bash
curl -X PUT 'http://localhost:6333/collections/resumes' \
  -H 'Content-Type: application/json' \
  -d '{
    "vectors": {
      "size": 384,
      "distance": "Cosine"
    },
    "hnsw_config": {
      "m": 16,
      "ef_construct": 100,
      "full_scan_threshold": 10
    }
  }'
```

### What This Does

This API call configures a new Qdrant collection optimized for semantic search:

#### Vector Configuration

| Parameter | Value | Meaning |
|-----------|-------|---------|
| `size` | `384` | Each vector has 384 floating-point dimensions. This matches the output of the **AllMiniLmL6V2** sentence transformer model used by the vector-worker. |
| `distance` | `Cosine` | Similarity between vectors is measured using **Cosine similarity**. This is ideal for text embeddings because it compares the *direction* of vectors rather than their magnitude, making it robust to varying document lengths. |

#### HNSW Index Configuration

Qdrant uses the **HNSW (Hierarchical Navigable Small World)** algorithm for approximate nearest neighbor search. The `hnsw_config` tunes this graph-based index:

| Parameter | Value | Meaning |
|-----------|-------|---------|
| `m` | `16` | The number of edges (connections) each node maintains in the HNSW graph. Higher values improve recall (search accuracy) at the cost of more memory and slower insertions. `16` is a solid default balancing speed and accuracy. |
| `ef_construct` | `100` | The size of the dynamic candidate list used during index construction. Higher values build a more accurate graph but take longer. `100` gives good quality without excessive build time. |
| `full_scan_threshold` | `10` | When a segment contains fewer than this many vectors, Qdrant skips the HNSW index and performs a brute-force scan instead. For tiny segments, linear scan is faster than traversing the graph. |

#### In Summary

This creates a collection that:
- Stores 384-dimensional vectors (one per resume)
- Uses cosine similarity for semantic matching
- Builds a high-quality HNSW graph index (`m=16`, `ef_construct=100`) for fast approximate nearest neighbor queries
- Falls back to exact search for very small data segments

### Verify the Collection

```bash
curl -s 'http://localhost:6333/collections/resumes' | jq .
```

## Event Schema

### Input: `DocumentVectorizedEvent`

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "vector": [0.123, -0.456, 0.789, ...]
}
```

The vector has **384 dimensions** (AllMiniLmL6V2 model output).

### Point ID Mapping

The document UUID is converted to a Qdrant point ID using:

```rust
let point_id = doc_id.as_u128() as u64;
```

This takes the lower 64 bits of the UUID. For most practical purposes, this provides sufficient uniqueness.

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `tokio` | 1.x | Async runtime |
| `rdkafka` | 0.39 | Kafka client |
| `qdrant-client` | 1.7 | Qdrant gRPC client |
| `serde` / `serde_json` | 1.0 | Serialization |
| `yaml_serde` | 0.10 | YAML config parsing |
| `tracing` | 0.1 | Structured logging |
| `shared-types` | local | Common event definitions |

## Development

### Prerequisites

1. **Rust Toolchain** (1.70+)
2. **CMake** — Required for building rdkafka
3. **Running Kafka** — See infrastructure setup below
4. **Running Qdrant** — With the `resumes` collection created

### Build

```bash
# Build
cargo build -p indexing-worker

# Build release
cargo build -p indexing-worker --release
```

### Run

```bash
# From workspace root
cargo run -p indexing-worker

# With debug logging
RUST_LOG=debug cargo run -p indexing-worker

# Module-specific logging
RUST_LOG=info,indexing_worker::handler=debug cargo run -p indexing-worker
```

### Infrastructure Setup

```bash
# Start Kafka + Qdrant
podman compose up -d

# Create the Qdrant collection (one-time)
curl -X PUT 'http://localhost:6333/collections/resumes' \
  -H 'Content-Type: application/json' \
  -d '{
    "vectors": { "size": 384, "distance": "Cosine" },
    "hnsw_config": { "m": 16, "ef_construct": 100, "full_scan_threshold": 10 }
  }'

# Create Kafka topic (if not already created)
podman exec -it kafka-broker kafka-topics --create \
  --topic resume_vectorized \
  --bootstrap-server localhost:9092 \
  --partitions 5
```

## Testing

### Manual Testing

1. **Start the worker:**
   ```bash
   cargo run -p indexing-worker
   ```

2. **Produce a test message to Kafka:**
   ```bash
   echo '{"id":"123e4567-e89b-12d3-a456-426614174000","vector":[0.1,0.2,0.3]}' | \
   podman exec -i kafka-broker kafka-console-producer \
     --topic resume_vectorized \
     --bootstrap-server localhost:9092
   ```

   > Replace the vector with a real 384-dimensional array for production testing.

3. **Verify the point was indexed in Qdrant:**
   ```bash
   curl -s 'http://localhost:6333/collections/resumes/points/scroll' \
     -H 'Content-Type: application/json' \
     -d '{"limit": 10, "with_vector": true}' | jq .
   ```

### End-to-End Test

```bash
# 1. Start infrastructure
podman compose up -d

# 2. Create collection + topics
# (see Infrastructure Setup above)

# 3. Start ingestion API (terminal 1)
cargo run -p ingestion-api

# 4. Start vector worker (terminal 2)
cargo run -p vector-worker

# 5. Start indexing worker (terminal 3)
cargo run -p indexing-worker

# 6. Submit a document (terminal 4)
curl -X POST http://localhost:3000/submit \
  -H "Content-Type: application/json" \
  -d '{"id":"550e8400-e29b-41d4-a716-446655440000","content":"Experienced ML engineer with 5 years in NLP"}'

# 7. Verify in Qdrant
curl -s 'http://localhost:6333/collections/resumes/points/scroll' \
  -H 'Content-Type: application/json' \
  -d '{"limit": 10, "with_vector": false}' | jq .
```

## Error Handling

| Error Type | Cause | Resolution |
|------------|-------|------------|
| `QdrantConnectionError` | Cannot connect to Qdrant or collection not found | Verify Qdrant is running, ensure the collection was created |
| `QdrantUpsertError` | Upsert operation failed | Check vector dimensions match collection config (384), verify Qdrant health |
| `KafkaError` | Kafka connection/consume failed | Verify broker is running, check topic exists |
| `SerializationError` | JSON deserialization failed | Check event payload structure matches `DocumentVectorizedEvent` |

## Logging

```
INFO  indexing_worker: Connected to Qdrant successfully collection=resumes url=http://localhost:6334
INFO  indexing_worker: Indexing Worker started. Listening for vectors... topic=resume_vectorized broker=localhost:9092
INFO  indexing_worker::handler: Processing vector for indexing doc_id="550e8400..."
INFO  indexing_worker::handler: Successfully indexed in Qdrant doc_id="550e8400..."
```

## Performance Notes

- Uses **gRPC** (port 6334) for lower latency compared to REST
- Upserts are performed with `wait(true)` ensuring durability before acknowledging
- Single-point upserts per message; batching can be added for higher throughput
- Horizontal scaling is possible via multiple consumer instances in the same consumer group

## Related Documentation

- [Vector Worker](../vector-worker/README.md) — Upstream service (produces vectors)
- [Ingestion API](../ingestion-api/README.md) — Entry point of the pipeline
- [Shared Types](../shared-types/README.md) — Event definitions
- [Project Root](../README.md) — Architecture overview
