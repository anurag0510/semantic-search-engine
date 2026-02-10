# Vector Worker

The Vector Worker is a Kafka consumer service that generates vector embeddings from text documents using a sentence transformer model. It consumes `DocumentReceivedEvent` messages and produces `DocumentVectorizedEvent` messages for downstream indexing.

## Overview

The worker performs three core operations:
1. **Consume** - Reads document events from Kafka input topic
2. **Vectorize** - Generates embeddings using AllMiniLmL6V2 model (384 dimensions)
3. **Produce** - Publishes vectorized events to Kafka output topic

## Architecture

```
Kafka (resume_received)     Vector Worker           Kafka (resume_vectorized)
         │                       │                           │
         │  DocumentReceivedEvent│                           │
         ├──────────────────────>│                           │
         │                       │  ML Inference             │
         │                       │  (AllMiniLmL6V2)          │
         │                       │                           │
         │                       │  DocumentVectorizedEvent  │
         │                       ├──────────────────────────>│
         │                       │                           │
```

## Project Structure

```
vector-worker/
├── Cargo.toml              # Dependencies
├── config.yaml             # Runtime configuration
├── README.md
└── src/
    ├── main.rs             # Entry point, event loop
    ├── config/mod.rs       # Configuration loading
    ├── error/mod.rs        # Custom error types
    ├── handler.rs          # Document processing logic
    ├── kafka/mod.rs        # Consumer/producer setup
    └── vectorizer/mod.rs   # ML model initialization & inference
```

### Module Responsibilities

| Module | Purpose |
|--------|---------|
| `main` | Initializes components, runs Kafka consumer loop |
| `config` | Loads YAML config with defaults fallback |
| `error` | `WorkerError` enum with error conversions |
| `handler` | Orchestrates vectorization and publishing |
| `kafka` | Creates Kafka consumer and producer |
| `vectorizer` | Manages ML model lifecycle and inference |

## Configuration

### Config File (`config.yaml`)

```yaml
kafka_broker: "localhost:9092"
input_topic: "resume_received"
output_topic: "resume_vectorized"
consumer_group: "vectorizer_group_v1"
kafka_timeout_ms: "5000"
kafka_send_timeout_secs: 5
```

### Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `kafka_broker` | String | `localhost:9092` | Kafka bootstrap server |
| `input_topic` | String | `resume_received` | Topic to consume from |
| `output_topic` | String | `resume_vectorized` | Topic to produce to |
| `consumer_group` | String | `vectorizer_group_v1` | Kafka consumer group ID |
| `kafka_timeout_ms` | String | `5000` | Kafka operation timeout |
| `kafka_send_timeout_secs` | u64 | `5` | Producer send timeout |

## Event Schemas

### Input: `DocumentReceivedEvent`

```json
{
  "payload": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "content": "Document text to vectorize..."
  }
}
```

### Output: `DocumentVectorizedEvent`

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "vector": [0.123, -0.456, 0.789, ...]
}
```

The output vector has 384 dimensions (AllMiniLmL6V2 model).

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `tokio` | 1.x | Async runtime |
| `rdkafka` | 0.39 | Kafka client |
| `rust-bert` | 0.23 | Sentence embeddings model |
| `serde` / `serde_json` | 1.0 | Serialization |
| `tracing` | 0.1 | Structured logging |
| `shared-types` | local | Common event definitions |

## Development

### Prerequisites

1. **Rust Toolchain** (1.70+)
2. **LibTorch** - Required by rust-bert (auto-downloaded on first run)
3. **CMake** - Required for building rdkafka
4. **Running Kafka** - See infrastructure setup below

### Build

```bash
# Build
cargo build -p vector-worker

# Build release
cargo build -p vector-worker --release
```

### Run

```bash
# From workspace root
cargo run -p vector-worker

# With debug logging
RUST_LOG=debug cargo run -p vector-worker

# Module-specific logging
RUST_LOG=info,vector_worker::vectorizer=debug cargo run -p vector-worker
```

### Infrastructure Setup

```bash
# Start Kafka
podman compose up -d

# Create input topic
podman exec -it kafka-broker kafka-topics --create \
  --topic resume_received \
  --bootstrap-server localhost:9092 \
  --partitions 5

# Create output topic
podman exec -it kafka-broker kafka-topics --create \
  --topic resume_vectorized \
  --bootstrap-server localhost:9092 \
  --partitions 5
```

## Testing

### Manual Testing

1. **Start the worker:**
   ```bash
   cargo run -p vector-worker
   ```

2. **Produce a test message:**
   ```bash
   echo '{"payload":{"id":"123e4567-e89b-12d3-a456-426614174000","content":"Senior Rust developer"}}' | \
   podman exec -i kafka-broker kafka-console-producer \
     --topic resume_received \
     --bootstrap-server localhost:9092
   ```

3. **Verify output:**
   ```bash
   podman exec -it kafka-broker kafka-console-consumer \
     --topic resume_vectorized \
     --from-beginning \
     --bootstrap-server localhost:9092 \
     --max-messages 1
   ```

### End-to-End Test

```bash
# 1. Start infrastructure
podman compose up -d

# 2. Create topics
podman exec -it kafka-broker kafka-topics --create --topic resume_received --bootstrap-server localhost:9092 --partitions 5
podman exec -it kafka-broker kafka-topics --create --topic resume_vectorized --bootstrap-server localhost:9092 --partitions 5

# 3. Start ingestion API (terminal 1)
cargo run -p ingestion-api

# 4. Start vector worker (terminal 2)
cargo run -p vector-worker

# 5. Submit document (terminal 3)
curl -X POST http://localhost:3000/submit \
  -H "Content-Type: application/json" \
  -d '{"id":"550e8400-e29b-41d4-a716-446655440000","content":"Experienced ML engineer"}'

# 6. Check vectorized output
podman exec -it kafka-broker kafka-console-consumer \
  --topic resume_vectorized \
  --from-beginning \
  --bootstrap-server localhost:9092
```

## Error Handling

| Error Type | Cause | Resolution |
|------------|-------|------------|
| `ModelInitError` | Failed to load ML model | Check LibTorch installation, network for model download |
| `InferenceError` | ML inference failed | Check input text, model mutex state |
| `KafkaError` | Kafka connection/publish failed | Verify broker is running, check topic exists |
| `SerializationError` | JSON serialization failed | Check event payload structure |

## ML Model

The worker uses **AllMiniLmL6V2** from the sentence-transformers family:

- **Dimensions**: 384
- **Model Size**: ~90MB
- **Inference Speed**: Fast (optimized for CPU)
- **Quality**: Good for semantic similarity tasks

On first run, the model is downloaded automatically (~90MB). Subsequent runs load from cache.

### Model Cache Location

```
~/.cache/huggingface/
```

## Performance Notes

- Model is wrapped in `Arc<Mutex<>>` for thread-safe access
- Inference runs in `spawn_blocking` to avoid blocking async runtime
- Single model instance is reused across all messages
- Consider horizontal scaling via multiple consumer instances in the same consumer group

## Logging

```
INFO  vector_worker: Initializing ML model (this may take time on first run)...
INFO  vector_worker: ML model loaded successfully
INFO  vector_worker: Worker started. Listening for events... topic=resume_received broker=localhost:9092
DEBUG vector_worker: Received message doc_id="550e8400..."
INFO  vector_worker::handler: Processing document doc_id="550e8400..."
INFO  vector_worker::handler: Vector generated successfully doc_id="550e8400..." dim=384
INFO  vector_worker::handler: Published vectorized event doc_id="550e8400..."
```

## Related Documentation

- [Ingestion API](../ingestion-api/README.md) - Upstream service
- [Shared Types](../shared-types/README.md) - Event definitions
- [Project Root](../README.md) - Architecture overview
