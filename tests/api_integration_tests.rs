// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! API Integration Tests
//!
//! Comprehensive end-to-end tests for Squirrel's API endpoints.
//! These tests start an actual Squirrel server instance and verify:
//! - Health checks and monitoring endpoints
//! - Ecosystem integration endpoints
//! - Service discovery and registration
//! - Metrics collection
//! - Error handling and edge cases
//!
//! Unlike mock-based tests, these verify actual HTTP communication
//! and real component integration.

use anyhow::Result;
use reqwest::{Client, StatusCode};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{oneshot, Mutex};
use std::sync::OnceLock;

// Global mutex to serialize port allocation and prevent race conditions
static PORT_ALLOC_MUTEX: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

// Import Squirrel components
use squirrel::api::ApiServer;
use squirrel::ecosystem::{EcosystemConfig, EcosystemManager};
use squirrel::shutdown::ShutdownManager;
use squirrel::MetricsCollector;

/// Test helper to start a Squirrel server on a specific port
struct TestSquirrelServer {
    port: u16,
    base_url: String,
    shutdown_tx: Option<oneshot::Sender<()>>,
    handle: Option<tokio::task::JoinHandle<()>>,
}

impl TestSquirrelServer {
    /// Start a new test server
    async fn start(port: u16) -> Result<Self> {
        let base_url = format!("http://localhost:{}", port);
        
        // Initialize components
        let metrics_collector = Arc::new(MetricsCollector::new());
        let ecosystem_config = EcosystemConfig::default();
        let ecosystem_manager = Arc::new(EcosystemManager::new(
            ecosystem_config,
            metrics_collector.clone(),
        ));
        let shutdown_manager = Arc::new(ShutdownManager::new());

        // Create API server
        let api_server = ApiServer::new(
            port,
            ecosystem_manager,
            metrics_collector,
            shutdown_manager,
        );

        // Start server in background task
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let handle = tokio::spawn(async move {
            let server_future = api_server.start();
            tokio::select! {
                _ = server_future => {},
                _ = shutdown_rx => {},
            }
        });

        // MODERNIZED: Event-driven server readiness detection
        // Poll health endpoint with exponential backoff (no fixed delay)
        let client = Client::new();
        let health_url = format!("{}/health", base_url);
        
        let mut backoff = Duration::from_millis(10);
        let max_wait = Duration::from_secs(3);
        let start = tokio::time::Instant::now();
        
        loop {
            if start.elapsed() > max_wait {
                return Err(anyhow::anyhow!("Server failed to start within timeout"));
            }
            
            if let Ok(response) = client.get(&health_url).send().await {
                if response.status().is_success() {
                    return Ok(Self {
                        port,
                        base_url,
                        shutdown_tx: Some(shutdown_tx),
                        handle: Some(handle),
                    });
                }
            }
            
            // Exponential backoff: 10ms, 20ms, 40ms, 80ms, 160ms (max)
            tokio::time::sleep(backoff).await;
            backoff = (backoff * 2).min(Duration::from_millis(160));
        }
    }

    /// Get the base URL
    fn url(&self) -> &str {
        &self.base_url
    }

    /// Shutdown the server
    async fn shutdown(mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        if let Some(handle) = self.handle.take() {
            // Give it a moment to shut down gracefully
            let _ = tokio::time::timeout(Duration::from_secs(2), handle).await;
        }
    }
}

impl Drop for TestSquirrelServer {
    fn drop(&mut self) {
        // Best effort cleanup
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

// Helper to find an available port (with mutex to prevent race conditions)
async fn find_available_port() -> u16 {
    use std::sync::atomic::{AtomicU16, Ordering};
    static PORT_COUNTER: AtomicU16 = AtomicU16::new(50000);
    
    // Acquire mutex to serialize port allocation across tests
    let _guard = PORT_ALLOC_MUTEX.lock().await;
    
    // Try up to 1000 ports from the counter position
    let start_port = PORT_COUNTER.fetch_add(1, Ordering::SeqCst);
    for offset in 0..1000 {
        let port = start_port.wrapping_add(offset) % 10000 + 50000;
        if tokio::net::TcpListener::bind(("127.0.0.1", port)).await.is_ok() {
            // Modern pattern: If bind succeeds, port is immediately available
            return port;
        }
    }
    panic!("No available ports found after checking 1000 ports");
}

/// Test 1: Basic server startup and health check
#[tokio::test]
async fn test_server_startup_and_health() -> Result<()> {
    let port = find_available_port().await;
    let server = TestSquirrelServer::start(port).await?;
    let client = Client::new();

    // Test /health endpoint
    let response = client
        .get(format!("{}/health", server.url()))
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    
    let body: Value = response.json().await?;
    assert_eq!(body["status"], "healthy");
    assert!(body["timestamp"].is_string());
    assert!(body["uptime_seconds"].is_number());

    server.shutdown().await;
    Ok(())
}

/// Test 2: Health check endpoints (live and ready)
#[tokio::test]
async fn test_health_endpoints() -> Result<()> {
    let port = find_available_port().await;
    let server = TestSquirrelServer::start(port).await?;
    let client = Client::new();

    // Test /health/live
    let response = client
        .get(format!("{}/health/live", server.url()))
        .send()
        .await?;
    
    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await?;
    assert_eq!(body["status"], "live");

    // Test /health/ready
    let response = client
        .get(format!("{}/health/ready", server.url()))
        .send()
        .await?;
    
    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await?;
    assert_eq!(body["status"], "ready");

    server.shutdown().await;
    Ok(())
}

/// Test 3: Ecosystem status endpoint
#[tokio::test]
async fn test_ecosystem_status() -> Result<()> {
    let port = find_available_port().await;
    let server = TestSquirrelServer::start(port).await?;
    let client = Client::new();

    let response = client
        .get(format!("{}/api/v1/ecosystem/status", server.url()))
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    
    let body: Value = response.json().await?;
    assert!(body["discovered_primals"].is_number());
    assert!(body["active_integrations"].is_array());
    assert!(body["cross_primal_status"].is_string());

    server.shutdown().await;
    Ok(())
}

/// Test 4: Service mesh status endpoint
#[tokio::test]
async fn test_service_mesh_status() -> Result<()> {
    let port = find_available_port().await;
    let server = TestSquirrelServer::start(port).await?;
    let client = Client::new();

    let response = client
        .get(format!("{}/api/v1/service-mesh/status", server.url()))
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    
    let body: Value = response.json().await?;
    // Should have service mesh status fields
    assert!(body.is_object());

    server.shutdown().await;
    Ok(())
}

/// Test 5: Primals list endpoint
#[tokio::test]
async fn test_primals_list() -> Result<()> {
    let port = find_available_port().await;
    let server = TestSquirrelServer::start(port).await?;
    let client = Client::new();

    let response = client
        .get(format!("{}/api/v1/primals", server.url()))
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    
    let body: Value = response.json().await?;
    assert!(body["primals"].is_array());
    // At minimum, Squirrel should be in the list
    assert!(body["primals"].as_array().unwrap().len() >= 1);

    server.shutdown().await;
    Ok(())
}

/// Test 6: Primal status endpoint
#[tokio::test]
async fn test_primal_status() -> Result<()> {
    let port = find_available_port().await;
    let server = TestSquirrelServer::start(port).await?;
    let client = Client::new();

    // Test getting Squirrel's own status
    let response = client
        .get(format!("{}/api/v1/primals/squirrel", server.url()))
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    
    let body: Value = response.json().await?;
    assert!(body["name"].is_string());
    assert!(body["status"].is_string());

    server.shutdown().await;
    Ok(())
}

/// Test 7: Metrics endpoint
#[tokio::test]
async fn test_metrics_endpoint() -> Result<()> {
    let port = find_available_port().await;
    let server = TestSquirrelServer::start(port).await?;
    let client = Client::new();

    // Make a few requests to generate metrics
    for _ in 0..3 {
        let _ = client.get(format!("{}/health", server.url())).send().await?;
    }

    let response = client
        .get(format!("{}/api/v1/metrics", server.url()))
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    
    let body: Value = response.json().await?;
    assert!(body["request_count"].is_number());
    assert!(body["uptime_seconds"].is_number());
    
    // Should have at least 4 requests (3 health + 1 metrics)
    let request_count = body["request_count"].as_u64().unwrap();
    assert!(request_count >= 4);

    server.shutdown().await;
    Ok(())
}

/// Test 8: Services list endpoint
#[tokio::test]
async fn test_services_list() -> Result<()> {
    let port = find_available_port().await;
    let server = TestSquirrelServer::start(port).await?;
    let client = Client::new();

    let response = client
        .get(format!("{}/api/v1/services", server.url()))
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    
    let body: Value = response.json().await?;
    assert!(body["services"].is_array());

    server.shutdown().await;
    Ok(())
}

/// Test 9: Songbird registration endpoint
#[tokio::test]
async fn test_songbird_register() -> Result<()> {
    let port = find_available_port().await;
    let server = TestSquirrelServer::start(port).await?;
    let client = Client::new();

    // Port configurable via TEST_SONGBIRD_PORT environment variable
    let songbird_port = std::env::var("TEST_SONGBIRD_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(9020);
    let registration_data = json!({
        "service_name": "test_songbird",
        "service_url": format!("http://localhost:{}", songbird_port),
        "capabilities": ["orchestration"]
    });

    let response = client
        .post(format!("{}/api/v1/songbird/register", server.url()))
        .json(&registration_data)
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::OK);

    server.shutdown().await;
    Ok(())
}

/// Test 10: Songbird heartbeat endpoint
#[tokio::test]
async fn test_songbird_heartbeat() -> Result<()> {
    let port = find_available_port().await;
    let server = TestSquirrelServer::start(port).await?;
    let client = Client::new();

    let heartbeat_data = json!({
        "service_name": "songbird",
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    let response = client
        .post(format!("{}/api/v1/songbird/heartbeat", server.url()))
        .json(&heartbeat_data)
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::OK);

    server.shutdown().await;
    Ok(())
}

/// Test 11: Concurrent requests
#[tokio::test]
async fn test_concurrent_requests() -> Result<()> {
    let port = find_available_port().await;
    let server = TestSquirrelServer::start(port).await?;
    let client = Client::new();

    // Make 10 concurrent health check requests
    let mut handles = vec![];
    for _ in 0..10 {
        let client = client.clone();
        let url = format!("{}/health", server.url());
        let handle = tokio::spawn(async move {
            client.get(&url).send().await
        });
        handles.push(handle);
    }

    // All should succeed
    for handle in handles {
        let response = handle.await??;
        assert_eq!(response.status(), StatusCode::OK);
    }

    server.shutdown().await;
    Ok(())
}

/// Test 12: Invalid endpoint returns 404
#[tokio::test]
async fn test_invalid_endpoint() -> Result<()> {
    let port = find_available_port().await;
    let server = TestSquirrelServer::start(port).await?;
    let client = Client::new();

    let response = client
        .get(format!("{}/api/v1/nonexistent", server.url()))
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    server.shutdown().await;
    Ok(())
}

/// Test 13: Request counting accuracy
#[tokio::test]
async fn test_request_counting() -> Result<()> {
    let port = find_available_port().await;
    let server = TestSquirrelServer::start(port).await?;
    let client = Client::new();

    // Make exactly 5 requests
    for _ in 0..5 {
        let _ = client.get(format!("{}/health", server.url())).send().await?;
    }

    // Check metrics
    let response = client
        .get(format!("{}/api/v1/metrics", server.url()))
        .send()
        .await?;

    let body: Value = response.json().await?;
    let count = body["request_count"].as_u64().unwrap();
    
    // Should be at least 6 (5 health + 1 metrics)
    assert!(count >= 6, "Expected at least 6 requests, got {}", count);

    server.shutdown().await;
    Ok(())
}

/// Test 14: Health check response structure
#[tokio::test]
async fn test_health_response_structure() -> Result<()> {
    let port = find_available_port().await;
    let server = TestSquirrelServer::start(port).await?;
    let client = Client::new();

    let response = client
        .get(format!("{}/health", server.url()))
        .send()
        .await?;

    let body: Value = response.json().await?;
    
    // Verify all expected fields exist
    assert!(body.get("status").is_some());
    assert!(body.get("timestamp").is_some());
    assert!(body.get("uptime_seconds").is_some());
    assert!(body.get("service_mesh").is_some());
    assert!(body.get("ecosystem").is_some());
    
    // Verify service_mesh structure
    let service_mesh = &body["service_mesh"];
    assert!(service_mesh.get("registered").is_some());
    assert!(service_mesh.get("connection_status").is_some());
    
    // Verify ecosystem structure
    let ecosystem = &body["ecosystem"];
    assert!(ecosystem.get("discovered_primals").is_some());
    assert!(ecosystem.get("active_integrations").is_some());

    server.shutdown().await;
    Ok(())
}

/// Test 15: CORS headers
#[tokio::test]
async fn test_cors_headers() -> Result<()> {
    let port = find_available_port().await;
    let server = TestSquirrelServer::start(port).await?;
    let client = Client::new();

    let response = client
        .get(format!("{}/health", server.url()))
        .header("Origin", "http://example.com")
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    
    // Should have CORS headers
    let headers = response.headers();
    assert!(headers.contains_key("access-control-allow-origin"));

    server.shutdown().await;
    Ok(())
}

/// Test 16: Server uptime tracking
#[tokio::test]
async fn test_uptime_tracking() -> Result<()> {
    let port = find_available_port().await;
    let server = TestSquirrelServer::start(port).await?;
    let client = Client::new();

    // Get initial uptime
    let response1 = client
        .get(format!("{}/health", server.url()))
        .send()
        .await?;
    let body1: Value = response1.json().await?;
    let uptime1 = body1["uptime_seconds"].as_u64().unwrap();

    // LEGITIMATE SLEEP: Testing uptime increases over real time
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Get uptime again
    let response2 = client
        .get(format!("{}/health", server.url()))
        .send()
        .await?;
    let body2: Value = response2.json().await?;
    let uptime2 = body2["uptime_seconds"].as_u64().unwrap();

    // Uptime should have increased by at least 2 seconds
    assert!(uptime2 >= uptime1 + 2, 
        "Uptime should increase: initial={}, after_2s={}", uptime1, uptime2);

    server.shutdown().await;
    Ok(())
}

/// Test 17: Multiple concurrent servers (port isolation)
#[tokio::test]
async fn test_multiple_servers() -> Result<()> {
    let port1 = find_available_port().await;
    let port2 = find_available_port().await;
    
    let server1 = TestSquirrelServer::start(port1).await?;
    let server2 = TestSquirrelServer::start(port2).await?;
    
    let client = Client::new();

    // Both servers should be accessible
    let response1 = client.get(format!("{}/health", server1.url())).send().await?;
    let response2 = client.get(format!("{}/health", server2.url())).send().await?;

    assert_eq!(response1.status(), StatusCode::OK);
    assert_eq!(response2.status(), StatusCode::OK);

    server1.shutdown().await;
    server2.shutdown().await;
    Ok(())
}

/// Test 18: Endpoint response times
#[tokio::test]
async fn test_response_times() -> Result<()> {
    let port = find_available_port().await;
    let server = TestSquirrelServer::start(port).await?;
    let client = Client::new();

    // All endpoints should respond quickly (< 1 second)
    let endpoints = vec![
        "/health",
        "/health/live",
        "/health/ready",
        "/api/v1/ecosystem/status",
        "/api/v1/primals",
        "/api/v1/metrics",
        "/api/v1/services",
    ];

    for endpoint in endpoints {
        let start = std::time::Instant::now();
        let response = client
            .get(format!("{}{}", server.url(), endpoint))
            .send()
            .await?;
        let duration = start.elapsed();

        assert_eq!(response.status(), StatusCode::OK, 
            "Endpoint {} returned non-OK status", endpoint);
        assert!(duration < Duration::from_secs(1), 
            "Endpoint {} took too long: {:?}", endpoint, duration);
    }

    server.shutdown().await;
    Ok(())
}

/// Test 19: JSON response validity
#[tokio::test]
async fn test_json_responses() -> Result<()> {
    let port = find_available_port().await;
    let server = TestSquirrelServer::start(port).await?;
    let client = Client::new();

    let endpoints = vec![
        "/health",
        "/api/v1/ecosystem/status",
        "/api/v1/primals",
        "/api/v1/metrics",
        "/api/v1/services",
    ];

    for endpoint in endpoints {
        let response = client
            .get(format!("{}{}", server.url(), endpoint))
            .send()
            .await?;

        // Should have JSON content type
        let content_type = response.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        
        assert!(content_type.contains("application/json"), 
            "Endpoint {} should return JSON, got: {}", endpoint, content_type);

        // Should parse as JSON
        let _: Value = response.json().await
            .expect(&format!("Failed to parse JSON from {}", endpoint));
    }

    server.shutdown().await;
    Ok(())
}

/// Test 20: Graceful shutdown
#[tokio::test]
async fn test_graceful_shutdown() -> Result<()> {
    let port = find_available_port().await;
    let server = TestSquirrelServer::start(port).await?;
    let client = Client::new();

    // Verify server is running
    let response = client.get(format!("{}/health", server.url())).send().await?;
    assert_eq!(response.status(), StatusCode::OK);

    // Shutdown server
    server.shutdown().await;

    // Modern pattern: Check immediately - timeout will handle if server is slow to stop
    let result = client
        .get(format!("http://localhost:{}/health", port))
        .timeout(Duration::from_secs(1))
        .send()
        .await;
    
    assert!(result.is_err(), "Server should not respond after shutdown");

    Ok(())
}

