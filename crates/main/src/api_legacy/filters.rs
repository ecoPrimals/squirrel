//! Warp filter helpers for dependency injection
//!
//! This module provides warp filter helpers that inject dependencies
//! into request handlers following the filter composition pattern.

use std::sync::Arc;
use tokio::sync::RwLock;
use warp::Filter;

use crate::ecosystem::EcosystemManager;
use crate::shutdown::ShutdownManager;
use crate::MetricsCollector;

use super::state::ServerState;

/// Inject server state into handlers
pub fn with_state(
    state: Arc<RwLock<ServerState>>,
) -> impl Filter<Extract = (Arc<RwLock<ServerState>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || state.clone())
}

/// Inject ecosystem manager into handlers
pub fn with_ecosystem_manager(
    ecosystem_manager: Arc<EcosystemManager>,
) -> impl Filter<Extract = (Arc<EcosystemManager>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || ecosystem_manager.clone())
}

/// Inject metrics collector into handlers
pub fn with_metrics_collector(
    metrics_collector: Arc<MetricsCollector>,
) -> impl Filter<Extract = (Arc<MetricsCollector>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || metrics_collector.clone())
}

/// Inject shutdown manager into handlers
pub fn with_shutdown_manager(
    shutdown_manager: Arc<ShutdownManager>,
) -> impl Filter<Extract = (Arc<ShutdownManager>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || shutdown_manager.clone())
}

/// Inject base URL into handlers
pub fn with_base_url(
    base_url: String,
) -> impl Filter<Extract = (String,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || base_url.clone())
}

