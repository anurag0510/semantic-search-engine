use crate::{error::MatcherError, qdrant, state::AppState};
use axum::{Json, extract::State, http::StatusCode};
use qdrant_client::qdrant::point_id::PointIdOptions;
use shared_types::{MatchQuery, MatchResponse, MatchResult};
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

/// Health check endpoint
pub async fn health_check() -> StatusCode {
    StatusCode::OK
}

/// Find matches handler
///
/// 1. Vectorizes the query text using the BERT model (blocking CPU task)
/// 2. Searches Qdrant for the nearest neighbours (async I/O)
/// 3. Maps scored points to the API response format
pub async fn find_matches(
    State(state): State<Arc<AppState>>,
    Json(query): Json<MatchQuery>,
) -> Result<Json<MatchResponse>, MatcherError> {
    tracing::info!("Received match query, length: {}", query.content.len());

    // 1. Vectorize the Query Text (blocking CPU task)
    let state_ref = state.clone();
    let query_text = query.content.clone();

    let vector_result = tokio::task::spawn_blocking(move || {
        let model_guard = state_ref.model.lock().expect("Mutex poisoned");
        model_guard.encode(&[query_text])
    })
    .await
    .map_err(|e| MatcherError::InternalError(e.to_string()))?;

    let query_vector = vector_result
        .map_err(|e| MatcherError::VectorizationError(e.to_string()))?
        .into_iter()
        .next()
        .ok_or_else(|| MatcherError::VectorizationError("No vector generated".into()))?;

    // 2. Search Qdrant (async I/O task)
    tracing::info!("Executing ANN search in Qdrant...");
    let scored_points = qdrant::search_nearest(
        &state.qdrant,
        &state.config.collection_name,
        query_vector,
        query.top_k,
    )
    .await?;

    // 3. Map results to API response format
    let results: Vec<MatchResult> = scored_points
        .into_iter()
        .filter_map(|scored_point| {
            if let Some(PointIdOptions::Uuid(uuid_str)) =
                scored_point.id.and_then(|id| id.point_id_options)
            {
                Uuid::from_str(&uuid_str).ok().map(|uuid| MatchResult {
                    id: uuid,
                    score: scored_point.score,
                })
            } else {
                None
            }
        })
        .collect();

    tracing::info!("Found {} matches", results.len());

    Ok(Json(MatchResponse { matches: results }))
}
