use crate::{error::ApiError, kafka, state::AppState};
use axum::{Json, extract::State, http::StatusCode};
use shared_types::DocumentPayload;
use std::sync::Arc;

/// Health check endpoint
pub async fn health_check() -> StatusCode {
    StatusCode::OK
}

/// Submit resume/document handler
/// Accepts JSON payload and publishes to Kafka
pub async fn submit_resume(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<DocumentPayload>,
) -> Result<StatusCode, ApiError> {
    let doc_id = payload.id.to_string();
    tracing::info!(%doc_id, "Received document submission");

    kafka::publish_document_event(&state.producer, &state.config, payload).await?;

    // Return 202 Accepted upon successful handoff to Kafka
    Ok(StatusCode::ACCEPTED)
}
