//! HTTP API for tickit-sync server

use axum::{
    Json, Router,
    extract::State,
    http::{StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use chrono::Utc;
use std::sync::Arc;

use crate::config::Config;
use crate::db::Database;
use crate::models::{SyncRequest, SyncResponse};

/// Application state shared across handlers
pub struct AppState {
    pub db: Database,
    pub config: Config,
}

impl AppState {
    pub fn new(db: Database, config: Config) -> Arc<Self> {
        Arc::new(Self { db, config })
    }
}

/// Create the API router
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/v1/sync", post(sync))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .with_state(state)
}

/// Health check endpoint (no auth required)
async fn health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "service": "tickit-sync",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// Auth middleware - validates Bearer token
async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    request: axum::http::Request<axum::body::Body>,
    next: Next,
) -> Response {
    // Skip auth for health check
    if request.uri().path() == "/health" {
        return next.run(request).await;
    }

    // Extract Authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    let token = match auth_header {
        Some(h) if h.starts_with("Bearer ") => &h[7..],
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "Missing or invalid Authorization header" })),
            )
                .into_response();
        }
    };

    // Validate token
    if !state.config.validate_token(token) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Invalid API token" })),
        )
            .into_response();
    }

    next.run(request).await
}

/// Main sync endpoint
async fn sync(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SyncRequest>,
) -> Result<Json<SyncResponse>, ApiError> {
    tracing::info!(
        device_id = %request.device_id,
        last_sync = ?request.last_sync,
        changes = request.changes.len(),
        "Sync request received"
    );

    // Apply incoming changes
    let conflicts = state.db.apply_changes(&request.changes)?;

    if !conflicts.is_empty() {
        tracing::info!(conflicts = ?conflicts, "Sync conflicts detected");
    }

    // Get changes for the client (since their last sync)
    let changes = state.db.get_changes_since(request.last_sync)?;

    let server_time = Utc::now();

    // Update device sync timestamp
    state
        .db
        .update_device_sync(request.device_id, server_time)?;

    tracing::info!(
        device_id = %request.device_id,
        outgoing_changes = changes.len(),
        conflicts = conflicts.len(),
        "Sync complete"
    );

    Ok(Json(SyncResponse {
        server_time,
        changes,
        conflicts,
    }))
}

/// API error type
#[derive(Debug)]
pub struct ApiError(anyhow::Error);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        tracing::error!(error = %self.0, "API error");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": self.0.to_string() })),
        )
            .into_response()
    }
}

impl<E> From<E> for ApiError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
