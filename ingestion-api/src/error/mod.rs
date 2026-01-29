use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

/// Custom error type for the ingestion API
#[derive(Debug)]
pub enum ApiError {
    SerializationError(String),
    KafkaPublishError(String),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            ApiError::KafkaPublishError(e) => write!(f, "Kafka publish error: {}", e),
        }
    }
}

impl std::error::Error for ApiError {}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::SerializationError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e),
            ApiError::KafkaPublishError(e) => (StatusCode::BAD_GATEWAY, e),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}
