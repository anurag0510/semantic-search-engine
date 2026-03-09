use crate::handlers;
use crate::state::AppState;
use axum::Router;
use axum::routing::{get, post};
use std::sync::Arc;

/// Build the application router with all routes and shared state
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(handlers::health_check))
        .route("/match", post(handlers::find_matches))
        .with_state(state)
}
