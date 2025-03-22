// WebSocket server for dashboard updates
//
// This module provides a real-time WebSocket server for dashboard updates
// that allows clients to receive live updates about system metrics, health status,
// and other monitoring information.

use std::sync::Arc;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;
use axum::{
    extract::{State, WebSocketUpgrade, Path, Query},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use axum::extract::ws::{WebSocket, Message};
use axum::http::{StatusCode, Method};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize};
use serde_json::Value;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use time::OffsetDateTime;
use tracing::{info, error, debug};

use squirrel_core::error::{Result, SquirrelError};
use super::{Manager, Layout, Component, Update, DashboardError};

// Function to start the dashboard server
pub async fn start_server(addr: SocketAddr, manager: Arc<dyn Manager>) -> Result<()> {
    info!("Starting dashboard server on {}", addr);
    
    // Create router with routes
    let router = create_router(manager);
    
    // Start the server
    axum::serve(tokio::net::TcpListener::bind(addr).await?, router)
        .await
        .map_err(|e| SquirrelError::Server(format!("Failed to start dashboard server: {}", e)))
}

// Rest of the file remains the same
// ... existing code ... 