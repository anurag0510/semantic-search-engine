# Ingestion API

The Ingestion API is the entry point for the semantic search engine pipeline. It exposes HTTP endpoints for receiving document submissions and publishes them as events to Kafka for downstream processing by the vectorization worker.

## 📋 Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Project Structure](#project-structure)
- [API Endpoints](#api-endpoints)
- [Configuration](#configuration)
- [Dependencies](#dependencies)
- [Development](#development)
- [Error Handling](#error-handling)
- [Logging](#logging)
- [Testing](#testing)

## Overview

The Ingestion API is a Rust-based HTTP service built with [Axum](https://github.com/tokio-rs/axum) that:

1. **Accepts document submissions** via RESTful endpoints
2. **Validates incoming payloads** using type-safe deserialization
3. **Publishes events to Kafka** using the `rdkafka` library
4. **Returns immediately** with HTTP 202 Accepted (asynchronous handoff pattern)

This service follows a fire-and-forget pattern where document processing happens asynchronously through the event pipeline, ensuring low latency for API clients.

## Architecture

### Component Flow

```
┌─────────────┐      HTTP POST       ┌──────────────────┐
│   Client    │ ───────────────────> │  Ingestion API   │
└─────────────┘                      └──────────────────┘
                                              │
                                              │ Publishes
                                              │ DocumentReceivedEvent
                                              ▼
                                      ┌──────────────────┐
                                      │   Kafka Topic    │
                                      │ resume_received  │
                                      └──────────────────┘
                                              │
                                              │ Consumed by
                                              ▼
                                      ┌──────────────────┐
                                      │ Vectorization    │
                                      │    Worker        │
                                      └──────────────────┘
```

### Key Design Decisions

- **Async Runtime**: Uses Tokio for non-blocking I/O operations
- **Event-Driven**: Decouples ingestion from processing using Kafka as message broker
- **Type Safety**: Leverages Rust's type system and shared-types crate for compile-time guarantees
- **Idempotent Keys**: Uses document UUID as Kafka partition key for ordering guarantees
- **Graceful Failures**: Returns specific HTTP status codes based on failure type

## Project Structure

```
ingestion-api/
├── Cargo.toml           # Package manifest with dependencies
├── config.yaml          # Runtime configuration (Kafka, server settings)
├── README.md           # This file
└── src/
    ├── main.rs         # Application entry point & server initialization
    ├── config/
    │   └── mod.rs      # Configuration loading and defaults
    ├── error/
    │   └── mod.rs      # Custom error types and HTTP error mapping
    ├── handlers/
    │   └── mod.rs      # HTTP request handlers (health, submit)
    ├── kafka/
    │   └── mod.rs      # Kafka producer creation and event publishing
    ├── routes/
    │   └── mod.rs      # Axum router configuration
    └── state/
        └── mod.rs      # Shared application state (producer, config)
```

### Module Responsibilities

| Module | Purpose |
|--------|---------|
| `main.rs` | Initializes logging, loads config, creates Kafka producer, starts HTTP server |
| `config` | Loads YAML configuration with fallback to defaults |
| `error` | Defines `ApiError` enum and implements Axum's `IntoResponse` for HTTP error mapping |
| `handlers` | HTTP endpoint implementations (`health_check`, `submit_resume`) |
| `kafka` | Kafka producer initialization and event publishing logic |
| `routes` | Defines HTTP routes and attaches handlers |
| `state` | `AppState` struct holding shared dependencies (producer, config) |

## API Endpoints

### `GET /health`

Health check endpoint for monitoring and readiness probes.

**Response:**
- `200 OK` - Service is healthy

**Example:**
```bash
curl http://localhost:3000/health
```

### `POST /submit`

Submit a document for processing through the semantic search pipeline.

**Request Body:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "content": "Experienced Software Engineer with 5 years in Rust and distributed systems..."
}
```

**Response:**
- `202 Accepted` - Document successfully published to Kafka
- `500 Internal Server Error` - Serialization failed
- `502 Bad Gateway` - Kafka publish failed

**Field Descriptions:**
- `id` (UUID): Unique identifier for the document, used as Kafka partition key
- `content` (String): Raw text content to be vectorized

**Example:**
```bash
curl -X POST http://localhost:3000/submit \
  -H "Content-Type: application/json" \
  -d '{
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "content": "Senior DevOps engineer with Kubernetes expertise"
  }'
```

**Important Notes:**
- The endpoint returns immediately after publishing to Kafka (asynchronous processing)
- HTTP 202 indicates successful handoff, NOT completion of processing
- Subsequent processing errors won't be reflected in this response

## Configuration

Configuration is loaded from `config.yaml` in the following order:
1. Attempt to load from `config.yaml` file
2. Fall back to hardcoded defaults if file is missing or invalid

### Configuration File (`config.yaml`)

```yaml
# Kafka Configuration
kafka_topic: "resume_received"         # Topic name for document events
kafka_broker: "127.0.0.1:9092"        # Kafka broker address
kafka_timeout_ms: "5000"              # Kafka operation timeout
kafka_send_timeout_secs: 2            # Timeout for send operations

# Server Configuration
server_host: "0.0.0.0"                # Listen address (0.0.0.0 = all interfaces)
server_port: 3000                      # HTTP server port
```

### Configuration Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `kafka_topic` | String | `resume_received` | Kafka topic for publishing events |
| `kafka_broker` | String | `127.0.0.1:9092` | Kafka broker connection string |
| `kafka_timeout_ms` | String | `5000` | Kafka client timeout in milliseconds |
| `kafka_send_timeout_secs` | u64 | `2` | Max seconds to wait for send acknowledgment |
| `server_host` | String | `0.0.0.0` | HTTP server bind address |
| `server_port` | u16 | `3000` | HTTP server port |

### Environment-Specific Configuration

For different environments, create separate config files:

```bash
# Development
cp config.yaml config.dev.yaml

# Production (adjust broker URLs, timeouts)
cp config.yaml config.prod.yaml
```

Then specify the config file when running:
```rust
let config = Config::from_file("config.prod.yaml")?;
```

## Dependencies

### Core Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `axum` | 0.8 | Modern, ergonomic web framework |
| `tokio` | 1.49 | Async runtime with full features |
| `rdkafka` | 0.39 | Apache Kafka client library |
| `serde` | 1.0 | Serialization/deserialization framework |
| `serde_json` | 1.0 | JSON serialization support |
| `yaml_serde` | 0.10 | YAML configuration parsing |
| `tower-http` | 0.6 | HTTP middleware (tracing, CORS) |
| `tracing` | 0.1 | Structured logging framework |
| `tracing-subscriber` | 0.3 | Log output formatting |
| `shared-types` | (local) | Shared type definitions and events |

### Kafka Feature Flags

The `rdkafka` dependency uses these features:
- `tokio`: Async integration with Tokio runtime
- `cmake-build`: Build librdkafka from source using CMake

## Development

### Prerequisites

1. **Rust Toolchain** (1.70+)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Running Kafka Instance**
   ```bash
   # From workspace root
   podman compose up -d
   
   # Create topic
   podman exec -it kafka-broker kafka-topics --create \
     --topic resume_received \
     --bootstrap-server localhost:9092 \
     --replication-factor 1 \
     --partitions 5
   ```

### Building

```bash
# Build in debug mode
cargo build

# Build with optimizations
cargo build --release
```

### Running

```bash
# Run from workspace root
cargo run -p ingestion-api

# Run with custom config
# (Edit main.rs to accept config path as CLI arg)
cargo run -p ingestion-api
```

### Development Workflow

```bash
# Watch mode (requires cargo-watch)
cargo install cargo-watch
cargo watch -x 'run -p ingestion-api'

# Format code
cargo fmt

# Lint code
cargo clippy --all-targets --all-features

# Check compilation without building
cargo check -p ingestion-api
```

### Debugging Kafka Events

Monitor Kafka topic to verify events are being published:

```bash
# Console consumer
podman exec -it kafka-broker kafka-console-consumer \
  --topic resume_received \
  --from-beginning \
  --bootstrap-server localhost:9092 \
  --property print.key=true \
  --property key.separator=":"

# Verify topic has messages
podman exec -it kafka-broker kafka-run-class \
  kafka.tools.GetOffsetShell \
  --broker-list localhost:9092 \
  --topic resume_received
```

## Error Handling

The API uses a custom `ApiError` enum that maps to appropriate HTTP status codes:

### Error Types

```rust
pub enum ApiError {
    SerializationError(String),  // 500 Internal Server Error
    KafkaPublishError(String),   // 502 Bad Gateway
}
```

### Error Response Format

All errors return JSON with an `error` field:

```json
{
  "error": "Kafka publish error: Failed to send message to broker"
}
```

### HTTP Status Codes

| Status Code | Meaning | Cause |
|-------------|---------|-------|
| `200 OK` | Health check successful | `/health` endpoint |
| `202 Accepted` | Document accepted for processing | Successful Kafka publish |
| `500 Internal Server Error` | JSON serialization failed | Invalid payload structure |
| `502 Bad Gateway` | Kafka unavailable | Broker unreachable or timeout |

### Error Propagation

The codebase uses Rust's `Result` type with the `?` operator for clean error propagation:

```rust
// In handlers/mod.rs
pub async fn submit_resume(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<DocumentPayload>,
) -> Result<StatusCode, ApiError> {
    // Errors automatically converted via ? operator
    kafka::publish_document_event(&state.producer, &state.config, payload).await?;
    Ok(StatusCode::ACCEPTED)
}
```

## Logging

The service uses the `tracing` crate for structured logging.

### Log Levels

- `DEBUG`: Initialization details, connection info
- `INFO`: Document submissions, successful Kafka publishes
- `ERROR`: Kafka publish failures

### Log Output Examples

```
2026-01-31T12:00:00.123456Z  INFO ingestion_api: Kafka Producer initialized successfully
2026-01-31T12:00:00.234567Z  INFO ingestion_api: Listening on 0.0.0.0:3000
2026-01-31T12:01:15.345678Z  INFO ingestion_api: Received document submission doc_id="550e8400-e29b-41d4-a716-446655440000"
2026-01-31T12:01:15.456789Z  INFO ingestion_api::kafka: Publishing document to Kafka doc_id="550e8400-e29b-41d4-a716-446655440000"
2026-01-31T12:01:15.567890Z  INFO ingestion_api::kafka: Successfully published to Kafka doc_id="550e8400-e29b-41d4-a716-446655440000"
```

### Adjusting Log Levels

Modify in [main.rs](src/main.rs):

```rust
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::INFO)  // Change to INFO, WARN, ERROR
    .init();
```

Or use environment variable (requires `env-filter` feature):

```bash
RUST_LOG=ingestion_api=debug,rdkafka=info cargo run -p ingestion-api
```

## Testing

### Manual Testing

#### 1. Test Health Endpoint
```bash
curl http://localhost:3000/health
# Expected: HTTP 200
```

#### 2. Test Document Submission
```bash
curl -X POST http://localhost:3000/submit \
  -H "Content-Type: application/json" \
  -d '{
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "content": "Senior Rust developer with expertise in async programming"
  }'
# Expected: HTTP 202
```

#### 3. Verify Kafka Message
```bash
podman exec -it kafka-broker kafka-console-consumer \
  --topic resume_received \
  --from-beginning \
  --bootstrap-server localhost:9092 \
  --max-messages 1
```

Expected output:
```json
{
  "payload": {
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "content": "Senior Rust developer with expertise in async programming"
  }
}
```

### Integration Testing Strategy

For automated tests, consider:

1. **Unit Tests**: Test individual functions (serialization, config loading)
2. **Integration Tests**: Use `testcontainers` to spin up Kafka for E2E testing
3. **Contract Tests**: Validate event schema matches shared-types definitions

### Example Unit Test Skeleton

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_defaults() {
        let config = Config::default_config();
        assert_eq!(config.kafka_topic, "resume_received");
        assert_eq!(config.server_port, 3000);
    }
    
    #[tokio::test]
    async fn test_health_check() {
        let response = handlers::health_check().await;
        assert_eq!(response, StatusCode::OK);
    }
}
```

---

## Related Documentation

- [Shared Types](../shared-types/README.md) - Common type definitions and events
- [Project Root](../README.md) - Overall architecture and setup

## Contributing

When making changes:

1. Ensure code compiles: `cargo check -p ingestion-api`
2. Format code: `cargo fmt`
3. Run linter: `cargo clippy`
4. Test manually with running Kafka instance
5. Update this README if adding new endpoints or configuration options

## License

See [LICENSE](../LICENSE) in the repository root.
