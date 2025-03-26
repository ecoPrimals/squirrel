//! API Documentation Module (temporarily disabled)
//! 
//! This module provides a placeholder for the OpenAPI documentation.
//! The real implementation is currently disabled due to dependency issues.

use std::sync::Arc;
use axum::Router;
use crate::state::AppState;

/// Create API documentation router (temporarily disabled)
pub fn docs_router() -> Router<Arc<AppState>> {
    // Return an empty router since documentation is temporarily disabled
    Router::new()
} 