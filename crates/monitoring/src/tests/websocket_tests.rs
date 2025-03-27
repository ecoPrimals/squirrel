use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, mpsc};
use tokio::task::JoinHandle;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, WebSocketStream, MaybeTlsStream};
use url::Url;
use serde_json::json;
use rand::{thread_rng, Rng};
use rand::seq::SliceRandom;
use tracing::{error, debug, info};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use futures_util::stream::FuturesUnordered;
use tokio::time::timeout;
use anyhow::{Result, anyhow};
use serde_json::Value;
use chrono::Utc;
use tokio::net::TcpListener;
use std::net::SocketAddr;
use tokio::time::sleep;
use crate::api::MonitoringAPI;

// Skip tests since the dashboard has been moved to its own crate
#[tokio::test]
#[ignore = "Skipping WebSocket tests as dashboard has been moved to a separate crate"]
async fn test_single_client_connection() {
    // This test is now skipped since the dashboard functionality has been moved
}

// Skip tests since the dashboard has been moved to its own crate
#[tokio::test]
#[ignore = "Skipping WebSocket tests as dashboard has been moved to a separate crate"]
async fn test_multiple_clients() {
    // This test is now skipped since the dashboard functionality has been moved
}

// Define a type alias for errors that can be sent across threads
type BoxError = Box<dyn std::error::Error + Send + Sync>;

// Helper function to create a test client
async fn connect_to_server(addr: &str) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>> {
    let url = Url::parse(addr)?;
    let (ws_stream, _) = connect_async(url).await?;
    Ok(ws_stream)
}

// Structure to represent a WebSocket client to be used in future tests
struct TestClient {
    id: String,
    subscriptions: Vec<String>,
    messages_received: usize,
    last_message: Option<String>,
    connection_time_ms: u64,  // Store as milliseconds since epoch instead of Instant
}

// Other test functions are commented out since they depend on dashboard functionality
// that has been moved to a separate crate 