use crate::{handlers, state::AppState};
use axum::{
    Router,
    routing::{get, post},
};
use std::sync::Arc;

/// Build and configure the application router
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(handlers::health_check))
        .route("/submit", post(handlers::submit_resume))
        .with_state(state)
}
