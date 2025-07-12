//! Simple Enhanced MCP Test Suite
//!
//! Basic tests for enhanced MCP functionality that work with current implementation.

use squirrel::enhanced::*;

/// Test enhanced MCP server creation and basic functionality
#[tokio::test]
async fn test_enhanced_mcp_server_creation() -> Result<(), Box<dyn std::error::Error>> {
    // Test server creation with default configuration
    let server = EnhancedMCPServer::default();
    
    // Verify server is properly initialized
    assert!(server.is_initialized());
    
    // Test server configuration
    let config = server.get_config();
    assert!(config.is_some());
    
    Ok(())
}

/// Test enhanced MCP server request processing
#[tokio::test]
async fn test_enhanced_mcp_request_processing() -> Result<(), Box<dyn std::error::Error>> {
    let server = EnhancedMCPServer::default();
    
    // Create a valid test request
    let request = MCPRequest {
        id: "test-request-001".to_string(),
        method: "test.method".to_string(),
        params: serde_json::json!({
            "test_param": "test_value"
        }),
        metadata: Some(RequestMetadata {
            timestamp: chrono::Utc::now(),
            client_id: "test-client".to_string(),
            session_id: Some("test-session".to_string()),
            correlation_id: Some("test-correlation".to_string()),
        }),
    };
    
    // Test request processing
    let response = server.process_request(request).await;
    
    // Verify response structure
    assert!(response.is_ok());
    let response = response.unwrap();
    assert_eq!(response.id, "test-request-001");
    assert!(response.success);
    
    Ok(())
}

/// Test enhanced MCP server error handling
#[tokio::test]
async fn test_enhanced_mcp_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    let server = EnhancedMCPServer::default();
    
    // Test error handling for invalid request (empty ID)
    let invalid_request = MCPRequest {
        id: "".to_string(), // Invalid empty ID
        method: "invalid.method".to_string(),
        params: serde_json::json!({}),
        metadata: None,
    };
    
    let invalid_response = server.process_request(invalid_request).await;
    assert!(invalid_response.is_err());
    
    // Test error handling for invalid method (empty method)
    let invalid_method_request = MCPRequest {
        id: "test-request".to_string(),
        method: "".to_string(), // Invalid empty method
        params: serde_json::json!({}),
        metadata: None,
    };
    
    let invalid_method_response = server.process_request(invalid_method_request).await;
    assert!(invalid_method_response.is_err());
    
    Ok(())
}

/// Test enhanced MCP server session management
#[tokio::test]
async fn test_enhanced_mcp_session_management() -> Result<(), Box<dyn std::error::Error>> {
    let server = EnhancedMCPServer::default();
    
    // Create test client info
    let client_info = ClientInfo {
        name: "test-client".to_string(),
        version: "1.0.0".to_string(),
        platform: Some("test-platform".to_string()),
    };
    
    // Test session creation
    let session_id = server.create_session(client_info.clone()).await?;
    assert!(!session_id.is_empty());
    
    // Test session ID is valid UUID format
    let _uuid = uuid::Uuid::parse_str(&session_id);
    assert!(_uuid.is_ok());
    
    Ok(())
}

/// Test enhanced MCP server metrics
#[tokio::test]
async fn test_enhanced_mcp_server_metrics() -> Result<(), Box<dyn std::error::Error>> {
    let server = EnhancedMCPServer::default();
    
    // Test server metrics retrieval
    let metrics = server.get_metrics().await;
    
    // Verify metrics structure
    assert!(metrics.total_connections >= 0);
    assert!(metrics.active_connections >= 0);
    assert!(metrics.total_requests >= 0);
    assert!(metrics.successful_requests >= 0);
    assert!(metrics.successful_requests <= metrics.total_requests);
    
    Ok(())
}

/// Test utilities for enhanced MCP operations
pub mod test_utils {
    use super::*;
    
    /// Create a test MCP server with default configuration
    pub fn create_test_server() -> EnhancedMCPServer {
        EnhancedMCPServer::default()
    }
    
    /// Create test client info with default values
    pub fn create_test_client_info() -> ClientInfo {
        ClientInfo {
            name: "test-client".to_string(),
            version: "1.0.0".to_string(),
            platform: Some("test-platform".to_string()),
        }
    }
    
    /// Create a test MCP request with valid parameters
    pub fn create_test_request(method: &str) -> MCPRequest {
        MCPRequest {
            id: format!("test-request-{}", uuid::Uuid::new_v4()),
            method: method.to_string(),
            params: serde_json::json!({
                "test_param": "test_value"
            }),
            metadata: Some(RequestMetadata {
                timestamp: chrono::Utc::now(),
                client_id: "test-client".to_string(),
                session_id: Some("test-session".to_string()),
                correlation_id: Some("test-correlation".to_string()),
            }),
        }
    }
} 