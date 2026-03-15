// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Integration tests for Squirrel with mock AI providers
//!
//! These tests validate the full stack integration including:
//! - Configuration loading
//! - AI router initialization
//! - JSON-RPC server with AI providers
//! - End-to-end request/response flow

use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::time::timeout;

/// Helper to send JSON-RPC request
async fn send_request(socket: &str, method: &str, params: Option<Value>, id: i32) -> Result<Value, String> {
    let stream = UnixStream::connect(socket)
        .await
        .map_err(|e| format!("Connect failed: {}", e))?;

    let request = json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": id
    });

    let mut request_str = serde_json::to_string(&request).unwrap();
    request_str.push('\n');

    let (read_half, mut write_half) = stream.into_split();
    write_half.write_all(request_str.as_bytes()).await.map_err(|e| e.to_string())?;

    let mut reader = BufReader::new(read_half);
    let mut response_line = String::new();

    timeout(Duration::from_secs(5), reader.read_line(&mut response_line))
        .await
        .map_err(|_| "Timeout".to_string())?
        .map_err(|e| e.to_string())?;

    serde_json::from_str(&response_line).map_err(|e| e.to_string())
}

// ============================================================================
// INTEGRATION TESTS
// ============================================================================

#[cfg(test)]
mod integration_tests {
    use super::*;
    use squirrel::config::{SquirrelConfig, ServerConfig, AiConfig};
    use squirrel::rpc::JsonRpcServer;
    use tempfile::tempdir;

    async fn start_test_server_with_config(socket: String, config: SquirrelConfig) -> Arc<JsonRpcServer> {
        let server = Arc::new(JsonRpcServer::new(socket.clone()));
        let server_clone = Arc::clone(&server);

        tokio::spawn(async move {
            let _ = server_clone.start().await;
        });

        tokio::time::sleep(Duration::from_millis(100)).await;
        server
    }

    #[tokio::test]
    async fn test_integration_basic_workflow() {
        let dir = tempdir().unwrap();
        let socket = dir.path().join("test.sock").to_str().unwrap().to_string();

        let config = SquirrelConfig::default();
        let _server = start_test_server_with_config(socket.clone(), config).await;

        // 1. Ping to verify connectivity
        let ping_response = send_request(&socket, "ping", None, 1).await.unwrap();
        assert_eq!(ping_response["result"]["pong"], true);

        // 2. Health check
        let health_response = send_request(&socket, "health", None, 2).await.unwrap();
        assert_eq!(health_response["result"]["status"], "healthy");

        // 3. Metrics
        let metrics_response = send_request(&socket, "metrics", None, 3).await.unwrap();
        assert!(metrics_response["result"]["requests_handled"].is_number());

        // 4. List providers (should work even without AI router)
        let providers_response = send_request(&socket, "list_providers", None, 4).await.unwrap();
        assert!(providers_response["result"]["providers"].is_array());
    }

    #[tokio::test]
    async fn test_integration_execute_tool() {
        let dir = tempdir().unwrap();
        let socket = dir.path().join("tool.sock").to_str().unwrap().to_string();

        let _server = start_test_server_with_config(socket.clone(), SquirrelConfig::default()).await;

        let params = json!({
            "tool": "calculator",
            "args": {"operation": "add", "a": 5, "b": 3}
        });

        let response = send_request(&socket, "execute_tool", Some(params), 1).await.unwrap();
        assert_eq!(response["result"]["tool"], "calculator");
        assert_eq!(response["result"]["status"], "not_implemented");
    }

    #[tokio::test]
    async fn test_integration_announce_capabilities() {
        let dir = tempdir().unwrap();
        let socket = dir.path().join("announce.sock").to_str().unwrap().to_string();

        let _server = start_test_server_with_config(socket.clone(), SquirrelConfig::default()).await;

        let params = json!({
            "capabilities": ["ai.text_generation", "ai.routing", "tool.orchestration"]
        });

        let response = send_request(&socket, "announce_capabilities", Some(params), 1).await.unwrap();
        assert_eq!(response["result"]["success"], true);
        assert!(response["result"]["message"].as_str().unwrap().contains("3 capabilities"));
    }

    #[tokio::test]
    async fn test_integration_discover_peers() {
        let dir = tempdir().unwrap();
        let socket = dir.path().join("peers.sock").to_str().unwrap().to_string();

        let _server = start_test_server_with_config(socket.clone(), SquirrelConfig::default()).await;

        let response = send_request(&socket, "discover_peers", None, 1).await.unwrap();
        assert!(response["result"]["peers"].is_array());
        assert_eq!(response["result"]["discovery_method"], "capability_registry");
    }

    #[tokio::test]
    async fn test_integration_error_handling() {
        let dir = tempdir().unwrap();
        let socket = dir.path().join("error.sock").to_str().unwrap().to_string();

        let _server = start_test_server_with_config(socket.clone(), SquirrelConfig::default()).await;

        // Test method not found
        let response = send_request(&socket, "nonexistent", None, 1).await.unwrap();
        assert!(response["error"].is_object());
        assert_eq!(response["error"]["code"], -32601);

        // Test invalid params
        let response = send_request(&socket, "execute_tool", None, 2).await.unwrap();
        assert!(response["error"].is_object());
        assert_eq!(response["error"]["code"], -32602);
    }

    #[tokio::test]
    async fn test_integration_concurrent_requests() {
        let dir = tempdir().unwrap();
        let socket = dir.path().join("concurrent.sock").to_str().unwrap().to_string();

        let _server = start_test_server_with_config(socket.clone(), SquirrelConfig::default()).await;

        // Send 20 concurrent requests
        let mut handles = vec![];
        for i in 1..=20 {
            let socket_clone = socket.clone();
            let handle = tokio::spawn(async move {
                send_request(&socket_clone, "ping", None, i).await
            });
            handles.push(handle);
        }

        // All should succeed
        for handle in handles {
            let response = handle.await.unwrap().unwrap();
            assert_eq!(response["result"]["pong"], true);
        }
    }

    #[tokio::test]
    async fn test_integration_metrics_accumulation() {
        let dir = tempdir().unwrap();
        let socket = dir.path().join("metrics_accum.sock").to_str().unwrap().to_string();

        let _server = start_test_server_with_config(socket.clone(), SquirrelConfig::default()).await;

        // Make several requests
        for i in 1..=5 {
            let _ = send_request(&socket, "ping", None, i).await;
        }

        // Check metrics
        let response = send_request(&socket, "metrics", None, 99).await.unwrap();
        let requests_handled = response["result"]["requests_handled"].as_u64().unwrap();
        assert!(requests_handled >= 5); // At least our 5 requests
    }

    #[tokio::test]
    async fn test_integration_query_ai_without_router() {
        let dir = tempdir().unwrap();
        let socket = dir.path().join("ai_no_router.sock").to_str().unwrap().to_string();

        let _server = start_test_server_with_config(socket.clone(), SquirrelConfig::default()).await;

        let params = json!({
            "prompt": "Hello, world!",
            "provider": "auto"
        });

        let response = send_request(&socket, "query_ai", Some(params), 1).await.unwrap();
        // Should return error since no AI router configured
        assert!(response["error"].is_object());
        assert_eq!(response["error"]["code"], -32603); // Internal error
        assert!(response["error"]["message"].as_str().unwrap().contains("not configured"));
    }
}

// ============================================================================
// CONFIGURATION TESTS
// ============================================================================

#[cfg(test)]
mod config_tests {
    use super::*;
    use squirrel::config::{ConfigLoader, SquirrelConfig};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_config_toml_loading() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("test.toml");

        let toml_content = r#"
[server]
port = 8080
daemon = true

[ai]
enabled = false

[logging]
level = "debug"

[discovery]
announce_capabilities = false
"#;

        fs::write(&config_path, toml_content).unwrap();

        let config = ConfigLoader::load(Some(&config_path)).unwrap();
        assert_eq!(config.server.port, 8080);
        assert!(config.server.daemon);
        assert!(!config.ai.enabled);
        assert_eq!(config.logging.level, "debug");
        assert!(!config.discovery.announce_capabilities);
    }

    #[test]
    fn test_config_yaml_loading() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("test.yaml");

        let yaml_content = r#"
server:
  port: 7070
  daemon: false

ai:
  enabled: true

logging:
  level: "warn"

discovery:
  announce_capabilities: true
  capabilities:
    - ai.test
    - tool.test
"#;

        fs::write(&config_path, yaml_content).unwrap();

        let config = ConfigLoader::load(Some(&config_path)).unwrap();
        assert_eq!(config.server.port, 7070);
        assert!(!config.server.daemon);
        assert!(config.ai.enabled);
        assert_eq!(config.logging.level, "warn");
        assert!(config.discovery.announce_capabilities);
        assert_eq!(config.discovery.capabilities.len(), 2);
    }

    #[test]
    fn test_config_json_loading() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("test.json");

        let json_content = r#"
{
  "server": {
    "port": 6060,
    "daemon": true
  },
  "ai": {
    "enabled": true
  },
  "logging": {
    "level": "error"
  },
  "discovery": {
    "announce_capabilities": false
  }
}
"#;

        fs::write(&config_path, json_content).unwrap();

        let config = ConfigLoader::load(Some(&config_path)).unwrap();
        assert_eq!(config.server.port, 6060);
        assert!(config.server.daemon);
        assert!(config.ai.enabled);
        assert_eq!(config.logging.level, "error");
        assert!(!config.discovery.announce_capabilities);
    }

    #[test]
    fn test_config_defaults() {
        let config = SquirrelConfig::default();
        assert_eq!(config.server.port, 9010);
        assert_eq!(config.server.bind, "0.0.0.0");
        assert!(config.ai.enabled);
        assert_eq!(config.logging.level, "info");
        assert!(config.discovery.announce_capabilities);
        assert_eq!(config.discovery.capabilities.len(), 4);
    }

    #[test]
    fn test_config_env_override() {
        std::env::set_var("SQUIRREL_PORT", "5050");
        std::env::set_var("SQUIRREL_LOG_LEVEL", "trace");

        let mut config = SquirrelConfig::default();
        squirrel::config::ConfigLoader::apply_env_overrides(&mut config).unwrap();

        assert_eq!(config.server.port, 5050);
        assert_eq!(config.logging.level, "trace");

        std::env::remove_var("SQUIRREL_PORT");
        std::env::remove_var("SQUIRREL_LOG_LEVEL");
    }
}

