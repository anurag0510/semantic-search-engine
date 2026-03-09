# Matching Service

The **Matching Service** is an HTTP API that accepts a natural language query, converts it to a semantic embedding vector using a local BERT model, and performs an approximate nearest-neighbour (ANN) search in a Qdrant vector database to return the most semantically similar documents.

## Architecture

```
HTTP POST /match
      │
      ▼
Vectorize query text
(BERT via rust-bert — blocking task on spawn_blocking)
      │
      ▼
ANN search in Qdrant
(top-k nearest neighbours by cosine/dot-product similarity)
      │
      ▼
Return scored results (document UUID + similarity score)
```

## Endpoints

| Method | Path      | Description                           |
|--------|-----------|---------------------------------------|
| `GET`  | `/health` | Liveness check — returns `200 OK`     |
| `POST` | `/match`  | Find top-k matches for a text query   |

### `POST /match`

**Request body:**

```json
{
  "content": "Experienced software engineer with Rust and distributed systems background",
  "top_k": 5
}
```

**Response:**

```json
{
  "matches": [
    { "id": "550e8400-e29b-41d4-a716-446655440000", "score": 0.92 },
    { "id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8", "score": 0.87 }
  ]
}
```

**Error responses:**

| Status | Condition                               |
|--------|-----------------------------------------|
| `500`  | BERT model failed to produce a vector   |
| `502`  | Qdrant is unreachable or returned error |

## Configuration

Configuration is loaded from `config.yaml` at startup. Falls back to defaults if the file is missing.

```yaml
qdrant_grpc_url: "http://localhost:6334"
collection_name: "resumes"
server_host: "0.0.0.0"
server_port: 3001
```

| Field             | Default                     | Description                          |
|-------------------|-----------------------------|--------------------------------------|
| `qdrant_grpc_url` | `http://localhost:6334`     | Qdrant gRPC endpoint                 |
| `collection_name` | `resumes`                   | Qdrant collection to search          |
| `server_host`     | `0.0.0.0`                   | Host address to bind                 |
| `server_port`     | `3001`                      | Port to listen on                    |

## Module Structure

```
src/
├── main.rs          # Server startup and dependency wiring only
├── config/          # Config struct — loads config.yaml
├── error/           # MatcherError with IntoResponse impl
├── handlers/        # Axum route handlers (health_check, find_matches)
├── qdrant/          # Qdrant client creation and search helper
├── routes/          # Router construction
└── state/           # AppState (BERT model + Qdrant client + Config)
```

## Running

```bash
# From the workspace root
cargo run -p matching-service
```

Ensure Qdrant is running first. Start the full stack with:

```bash
docker compose up -d
```

## Dependencies

- [`axum`](https://github.com/tokio-rs/axum) — HTTP framework
- [`rust-bert`](https://github.com/guillaume-be/rust-bert) — BERT sentence embeddings via LibTorch
- [`qdrant-client`](https://github.com/qdrant/rust-client) — Qdrant gRPC client
- [`shared-types`](../shared-types) — Shared request/response types (`MatchQuery`, `MatchResponse`, `MatchResult`)
