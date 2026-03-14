// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! End-to-End MCP Integration Tests
//!
//! Tests complete Model Context Protocol (MCP) integration workflows including:
//! - Tool discovery and registration
//! - Resource access and management
//! - Transport layer operations
//! - Message routing
//! - Error handling and recovery

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::timeout;
use uuid::Uuid;

/// Mock MCP tool definition
#[derive(Debug, Clone)]
struct MCPTool {
    name: String,
    description: String,
    parameters: Vec<MCPParameter>,
    enabled: bool,
}

#[derive(Debug, Clone)]
struct MCPParameter {
    name: String,
    param_type: String,
    required: bool,
    description: String,
}

/// Mock MCP resource
#[derive(Debug, Clone)]
struct MCPResource {
    uri: String,
    name: String,
    mime_type: String,
    content: String,
}

/// Mock MCP transport layer
struct MCPTransport {
    tools: Arc<RwLock<HashMap<String, MCPTool>>>,
    resources: Arc<RwLock<HashMap<String, MCPResource>>>,
    message_queue: Arc<RwLock<Vec<MCPMessage>>>,
    connections: Arc<RwLock<HashMap<String, MCPConnection>>>,
}

#[derive(Debug, Clone)]
struct MCPMessage {
    id: String,
    message_type: MCPMessageType,
    payload: serde_json::Value,
    timestamp: std::time::SystemTime,
}

#[derive(Debug, Clone)]
enum MCPMessageType {
    ToolDiscovery,
    ToolInvocation,
    ResourceRequest,
    ResourceResponse,
    Error,
}

#[derive(Debug, Clone)]
struct MCPConnection {
    id: String,
    client_id: String,
    connected_at: std::time::SystemTime,
    last_activity: std::time::SystemTime,
    active: bool,
}

impl MCPTransport {
    fn new() -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            resources: Arc::new(RwLock::new(HashMap::new())),
            message_queue: Arc::new(RwLock::new(Vec::new())),
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn register_tool(&self, tool: MCPTool) -> Result<(), String> {
        if tool.name.is_empty() {
            return Err("Tool name cannot be empty".to_string());
        }

        let mut tools = self.tools.write().await;
        tools.insert(tool.name.clone(), tool);
        Ok(())
    }

    async fn discover_tools(&self) -> Vec<MCPTool> {
        let tools = self.tools.read().await;
        tools.values().filter(|t| t.enabled).cloned().collect()
    }

    async fn invoke_tool(
        &self,
        tool_name: &str,
        arguments: HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value, String> {
        let tools = self.tools.read().await;
        let tool = tools
            .get(tool_name)
            .ok_or_else(|| format!("Tool '{}' not found", tool_name))?;

        if !tool.enabled {
            return Err(format!("Tool '{}' is disabled", tool_name));
        }

        // Validate required parameters
        for param in &tool.parameters {
            if param.required && !arguments.contains_key(&param.name) {
                return Err(format!("Missing required parameter: {}", param.name));
            }
        }

        // Simulate tool execution
        Ok(serde_json::json!({
            "tool": tool_name,
            "result": "success",
            "arguments": arguments,
            "executed_at": chrono::Utc::now().to_rfc3339()
        }))
    }

    async fn register_resource(&self, resource: MCPResource) -> Result<(), String> {
        if resource.uri.is_empty() {
            return Err("Resource URI cannot be empty".to_string());
        }

        let mut resources = self.resources.write().await;
        resources.insert(resource.uri.clone(), resource);
        Ok(())
    }

    async fn get_resource(&self, uri: &str) -> Result<MCPResource, String> {
        let resources = self.resources.read().await;
        resources
            .get(uri)
            .cloned()
            .ok_or_else(|| format!("Resource '{}' not found", uri))
    }

    async fn send_message(&self, message: MCPMessage) -> Result<(), String> {
        let mut queue = self.message_queue.write().await;
        queue.push(message);
        Ok(())
    }

    async fn get_message_count(&self) -> usize {
        self.message_queue.read().await.len()
    }

    async fn connect(&self, client_id: String) -> Result<String, String> {
        let connection_id = Uuid::new_v4().to_string();
        let connection = MCPConnection {
            id: connection_id.clone(),
            client_id,
            connected_at: std::time::SystemTime::now(),
            last_activity: std::time::SystemTime::now(),
            active: true,
        };

        let mut connections = self.connections.write().await;
        connections.insert(connection_id.clone(), connection);
        Ok(connection_id)
    }

    async fn disconnect(&self, connection_id: &str) -> Result<(), String> {
        let mut connections = self.connections.write().await;
        let connection = connections
            .get_mut(connection_id)
            .ok_or_else(|| "Connection not found".to_string())?;

        connection.active = false;
        Ok(())
    }

    async fn get_active_connections(&self) -> usize {
        let connections = self.connections.read().await;
        connections.values().filter(|c| c.active).count()
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// E2E MCP INTEGRATION TESTS
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn test_tool_discovery_flow() {
    let transport = MCPTransport::new();

    // Register multiple tools
    let tools = vec![
        MCPTool {
            name: "calculator".to_string(),
            description: "Performs calculations".to_string(),
            parameters: vec![MCPParameter {
                name: "expression".to_string(),
                param_type: "string".to_string(),
                required: true,
                description: "Mathematical expression".to_string(),
            }],
            enabled: true,
        },
        MCPTool {
            name: "translator".to_string(),
            description: "Translates text".to_string(),
            parameters: vec![
                MCPParameter {
                    name: "text".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    description: "Text to translate".to_string(),
                },
                MCPParameter {
                    name: "language".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    description: "Target language".to_string(),
                },
            ],
            enabled: true,
        },
        MCPTool {
            name: "disabled_tool".to_string(),
            description: "This tool is disabled".to_string(),
            parameters: vec![],
            enabled: false,
        },
    ];

    for tool in tools {
        transport
            .register_tool(tool)
            .await
            .expect("Tool registration should succeed");
    }

    // Discover tools
    let discovered = transport.discover_tools().await;

    assert_eq!(discovered.len(), 2, "Should discover 2 enabled tools");
    assert!(
        discovered.iter().any(|t| t.name == "calculator"),
        "Should discover calculator tool"
    );
    assert!(
        discovered.iter().any(|t| t.name == "translator"),
        "Should discover translator tool"
    );
    assert!(
        !discovered.iter().any(|t| t.name == "disabled_tool"),
        "Should not discover disabled tool"
    );
}

#[tokio::test]
async fn test_tool_invocation() {
    let transport = MCPTransport::new();

    // Register a tool
    let tool = MCPTool {
        name: "echo".to_string(),
        description: "Echoes input".to_string(),
        parameters: vec![MCPParameter {
            name: "message".to_string(),
            param_type: "string".to_string(),
            required: true,
            description: "Message to echo".to_string(),
        }],
        enabled: true,
    };

    transport
        .register_tool(tool)
        .await
        .expect("Tool registration should succeed");

    // Invoke tool with valid arguments
    let mut args = HashMap::new();
    args.insert(
        "message".to_string(),
        serde_json::Value::String("Hello, MCP!".to_string()),
    );

    let result = transport.invoke_tool("echo", args).await;
    assert!(result.is_ok(), "Tool invocation should succeed");

    let response = result.unwrap();
    assert_eq!(
        response["tool"].as_str().unwrap(),
        "echo",
        "Response should contain tool name"
    );
    assert_eq!(
        response["result"].as_str().unwrap(),
        "success",
        "Result should be success"
    );
}

#[tokio::test]
async fn test_tool_invocation_with_missing_parameters() {
    let transport = MCPTransport::new();

    // Register a tool with required parameters
    let tool = MCPTool {
        name: "required_params".to_string(),
        description: "Tool with required params".to_string(),
        parameters: vec![
            MCPParameter {
                name: "param1".to_string(),
                param_type: "string".to_string(),
                required: true,
                description: "Required parameter 1".to_string(),
            },
            MCPParameter {
                name: "param2".to_string(),
                param_type: "string".to_string(),
                required: true,
                description: "Required parameter 2".to_string(),
            },
        ],
        enabled: true,
    };

    transport
        .register_tool(tool)
        .await
        .expect("Tool registration should succeed");

    // Try to invoke with missing parameters
    let mut args = HashMap::new();
    args.insert("param1".to_string(), serde_json::Value::String("value1".to_string()));

    let result = transport.invoke_tool("required_params", args).await;
    assert!(result.is_err(), "Should fail with missing required parameter");
    assert!(
        result.unwrap_err().contains("Missing required parameter"),
        "Error should mention missing parameter"
    );
}

#[tokio::test]
async fn test_resource_registration_and_retrieval() {
    let transport = MCPTransport::new();

    // Register a resource
    let resource = MCPResource {
        uri: "mcp://localhost/data/config.json".to_string(),
        name: "Configuration".to_string(),
        mime_type: "application/json".to_string(),
        content: r#"{"key": "value"}"#.to_string(),
    };

    transport
        .register_resource(resource.clone())
        .await
        .expect("Resource registration should succeed");

    // Retrieve the resource
    let retrieved = transport
        .get_resource("mcp://localhost/data/config.json")
        .await
        .expect("Resource retrieval should succeed");

    assert_eq!(retrieved.uri, resource.uri);
    assert_eq!(retrieved.name, resource.name);
    assert_eq!(retrieved.mime_type, resource.mime_type);
    assert_eq!(retrieved.content, resource.content);
}

#[tokio::test]
async fn test_resource_not_found() {
    let transport = MCPTransport::new();

    let result = transport.get_resource("mcp://localhost/nonexistent").await;
    assert!(result.is_err(), "Should fail for nonexistent resource");
    assert!(
        result.unwrap_err().contains("not found"),
        "Error should mention resource not found"
    );
}

#[tokio::test]
async fn test_message_routing() {
    let transport = MCPTransport::new();

    // Send multiple messages
    let messages = vec![
        MCPMessage {
            id: Uuid::new_v4().to_string(),
            message_type: MCPMessageType::ToolDiscovery,
            payload: serde_json::json!({"action": "discover"}),
            timestamp: std::time::SystemTime::now(),
        },
        MCPMessage {
            id: Uuid::new_v4().to_string(),
            message_type: MCPMessageType::ToolInvocation,
            payload: serde_json::json!({"tool": "calculator", "args": {}}),
            timestamp: std::time::SystemTime::now(),
        },
        MCPMessage {
            id: Uuid::new_v4().to_string(),
            message_type: MCPMessageType::ResourceRequest,
            payload: serde_json::json!({"uri": "mcp://localhost/data"}),
            timestamp: std::time::SystemTime::now(),
        },
    ];

    for message in messages {
        transport
            .send_message(message)
            .await
            .expect("Message sending should succeed");
    }

    let count = transport.get_message_count().await;
    assert_eq!(count, 3, "Should have 3 messages in queue");
}

#[tokio::test]
async fn test_connection_lifecycle() {
    let transport = MCPTransport::new();

    // Connect a client
    let connection_id = transport
        .connect("client-123".to_string())
        .await
        .expect("Connection should succeed");

    assert!(!connection_id.is_empty(), "Should return connection ID");

    // Verify active connections
    let active = transport.get_active_connections().await;
    assert_eq!(active, 1, "Should have 1 active connection");

    // Disconnect
    transport
        .disconnect(&connection_id)
        .await
        .expect("Disconnect should succeed");

    // Verify no active connections
    let active_after = transport.get_active_connections().await;
    assert_eq!(active_after, 0, "Should have 0 active connections");
}

#[tokio::test]
async fn test_concurrent_tool_invocations() {
    let transport = Arc::new(MCPTransport::new());

    // Register a tool
    let tool = MCPTool {
        name: "concurrent_test".to_string(),
        description: "Tool for concurrent testing".to_string(),
        parameters: vec![MCPParameter {
            name: "input".to_string(),
            param_type: "string".to_string(),
            required: true,
            description: "Input parameter".to_string(),
        }],
        enabled: true,
    };

    transport
        .register_tool(tool)
        .await
        .expect("Tool registration should succeed");

    // Spawn concurrent invocations
    let mut handles = vec![];
    for i in 0..10 {
        let transport_clone = Arc::clone(&transport);
        let handle = tokio::spawn(async move {
            let mut args = HashMap::new();
            args.insert(
                "input".to_string(),
                serde_json::Value::String(format!("test_{}", i)),
            );
            transport_clone.invoke_tool("concurrent_test", args).await
        });
        handles.push(handle);
    }

    // Wait for all invocations
    let mut results = vec![];
    for handle in handles {
        let result = handle.await.expect("Task should complete");
        results.push(result);
    }

    // All invocations should succeed
    for result in &results {
        assert!(result.is_ok(), "All concurrent invocations should succeed");
    }

    assert_eq!(results.len(), 10, "Should have 10 results");
}

#[tokio::test]
async fn test_multiple_concurrent_connections() {
    let transport = Arc::new(MCPTransport::new());

    // Spawn concurrent connections
    let mut handles = vec![];
    for i in 0..20 {
        let transport_clone = Arc::clone(&transport);
        let handle = tokio::spawn(async move {
            transport_clone
                .connect(format!("client-{}", i))
                .await
        });
        handles.push(handle);
    }

    // Wait for all connections
    let mut connection_ids = vec![];
    for handle in handles {
        let result = handle.await.expect("Task should complete");
        assert!(result.is_ok(), "Connection should succeed");
        connection_ids.push(result.unwrap());
    }

    // Verify all connections are active
    let active = transport.get_active_connections().await;
    assert_eq!(active, 20, "Should have 20 active connections");

    // All connection IDs should be unique
    let unique_ids: std::collections::HashSet<String> =
        connection_ids.iter().cloned().collect();
    assert_eq!(unique_ids.len(), 20, "All connection IDs should be unique");
}

#[tokio::test]
async fn test_transport_with_timeout() {
    let transport = MCPTransport::new();

    // Register tool with timeout
    let tool = MCPTool {
        name: "timeout_test".to_string(),
        description: "Tool for timeout testing".to_string(),
        parameters: vec![],
        enabled: true,
    };

    let result = timeout(Duration::from_secs(1), transport.register_tool(tool)).await;
    assert!(result.is_ok(), "Registration should complete within timeout");

    // Discover tools with timeout
    let discovery_result = timeout(Duration::from_secs(1), transport.discover_tools()).await;
    assert!(
        discovery_result.is_ok(),
        "Discovery should complete within timeout"
    );

    let tools = discovery_result.unwrap();
    assert_eq!(tools.len(), 1, "Should discover 1 tool");
}

#[tokio::test]
async fn test_error_handling_in_tool_invocation() {
    let transport = MCPTransport::new();

    // Try to invoke non-existent tool
    let result = transport
        .invoke_tool("nonexistent_tool", HashMap::new())
        .await;
    assert!(result.is_err(), "Should fail for nonexistent tool");

    // Register but disable a tool
    let tool = MCPTool {
        name: "disabled".to_string(),
        description: "Disabled tool".to_string(),
        parameters: vec![],
        enabled: false,
    };

    transport
        .register_tool(tool)
        .await
        .expect("Registration should succeed");

    // Try to invoke disabled tool
    let result = transport.invoke_tool("disabled", HashMap::new()).await;
    assert!(result.is_err(), "Should fail for disabled tool");
    assert!(
        result.unwrap_err().contains("disabled"),
        "Error should mention tool is disabled"
    );
}

#[tokio::test]
async fn test_complete_mcp_workflow() {
    let transport = MCPTransport::new();

    // 1. Connect a client
    let connection_id = transport
        .connect("workflow-client".to_string())
        .await
        .expect("Connection should succeed");

    // 2. Register tools
    let tool = MCPTool {
        name: "workflow_tool".to_string(),
        description: "Tool for workflow testing".to_string(),
        parameters: vec![MCPParameter {
            name: "data".to_string(),
            param_type: "string".to_string(),
            required: true,
            description: "Data parameter".to_string(),
        }],
        enabled: true,
    };

    transport
        .register_tool(tool)
        .await
        .expect("Tool registration should succeed");

    // 3. Register resources
    let resource = MCPResource {
        uri: "mcp://workflow/data".to_string(),
        name: "Workflow Data".to_string(),
        mime_type: "application/json".to_string(),
        content: r#"{"status": "ready"}"#.to_string(),
    };

    transport
        .register_resource(resource)
        .await
        .expect("Resource registration should succeed");

    // 4. Discover tools
    let tools = transport.discover_tools().await;
    assert_eq!(tools.len(), 1, "Should discover 1 tool");

    // 5. Invoke tool
    let mut args = HashMap::new();
    args.insert(
        "data".to_string(),
        serde_json::Value::String("workflow_data".to_string()),
    );

    let invocation_result = transport.invoke_tool("workflow_tool", args).await;
    assert!(invocation_result.is_ok(), "Tool invocation should succeed");

    // 6. Access resource
    let resource_result = transport.get_resource("mcp://workflow/data").await;
    assert!(resource_result.is_ok(), "Resource access should succeed");

    // 7. Send messages
    let message = MCPMessage {
        id: Uuid::new_v4().to_string(),
        message_type: MCPMessageType::ResourceResponse,
        payload: serde_json::json!({"data": "workflow_complete"}),
        timestamp: std::time::SystemTime::now(),
    };

    transport
        .send_message(message)
        .await
        .expect("Message sending should succeed");

    // 8. Verify message queue
    let message_count = transport.get_message_count().await;
    assert_eq!(message_count, 1, "Should have 1 message in queue");

    // 9. Disconnect
    transport
        .disconnect(&connection_id)
        .await
        .expect("Disconnect should succeed");

    let active = transport.get_active_connections().await;
    assert_eq!(active, 0, "Should have no active connections");
}

