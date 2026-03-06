use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

/// Custom error types for the matching service
#[derive(Debug)]
pub enum MatcherError {
    VectorizationError(String),
    QdrantSearchError(String),
    InternalError(String),
}

impl std::fmt::Display for MatcherError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MatcherError::VectorizationError(e) => write!(f, "Vectorization error: {}", e),
            MatcherError::QdrantSearchError(e) => write!(f, "Qdrant search error: {}", e),
            MatcherError::InternalError(e) => write!(f, "Internal error: {}", e),
        }
    }
}

impl std::error::Error for MatcherError {}

impl IntoResponse for MatcherError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            MatcherError::VectorizationError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.clone()),
            MatcherError::QdrantSearchError(e) => (StatusCode::BAD_GATEWAY, e.clone()),
            MatcherError::InternalError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.clone()),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}
