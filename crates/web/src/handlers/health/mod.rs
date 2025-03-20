//! Health check handler for the API.

use axum::{response::IntoResponse, Json};
use serde_json::json;

/// Health check endpoint
pub async fn check() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "version": squirrel_core::build_info::version(),
    }))
} 