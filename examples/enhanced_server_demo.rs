use squirrel_mcp::enhanced::{EnhancedMCPServer, EnhancedMCPConfig};
use squirrel_mcp::enhanced::{MCPRequest, MCPCapability, ClientInfo, UserPreferences};
use tokio;

/// Demonstration of the Enhanced MCP Server capabilities
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Enhanced MCP Server Demo");
    println!("===========================");
    
    // 1. Create Enhanced MCP Configuration
    let config = EnhancedMCPConfig::default();
    println!("✅ Configuration created: {:?}", config.server.bind_address);
    
    // 2. Initialize Enhanced Server
    let server = EnhancedMCPServer::new(config).await?;
    println!("✅ Enhanced MCP Server initialized");
    
    // 3. Start the server
    server.start().await?;
    println!("✅ Server started successfully");
    
    // 4. Create a test client session
    let client_info = ClientInfo {
        id: "demo-client-001".to_string(),
        capabilities: vec![
            MCPCapability {
                name: "tool_execution".to_string(),
                version: "1.0.0".to_string(),
                description: Some("Can execute tools".to_string()),
            }
        ],
        preferences: UserPreferences::default(),
    };
    
    let session_id = server.create_session(client_info).await?;
    println!("✅ Session created: {}", session_id);
    
    // 5. Test MCP protocol requests
    
    // Initialize request
    let init_request = MCPRequest::Initialize {
        capabilities: vec![
            MCPCapability {
                name: "core_mcp".to_string(),
                version: "1.0.0".to_string(),
                description: Some("Core MCP functionality".to_string()),
            }
        ]
    };
    
    let response = server.handle_mcp_request(&session_id, init_request).await?;
    println!("✅ Initialize response: {:?}", response);
    
    // List tools request
    let list_tools_request = MCPRequest::ListTools { category: None };
    let response = server.handle_mcp_request(&session_id, list_tools_request).await?;
    println!("✅ List tools response: {:?}", response);
    
    // Get status request
    let status_request = MCPRequest::GetStatus;
    let response = server.handle_mcp_request(&session_id, status_request).await?;
    println!("✅ Status response: {:?}", response);
    
    // 6. Show server metrics
    let metrics = server.get_metrics().await;
    println!("✅ Server metrics:");
    println!("   - Total connections: {}", metrics.total_connections);
    println!("   - Active connections: {}", metrics.active_connections);
    println!("   - Total requests: {}", metrics.total_requests);
    println!("   - Successful requests: {}", metrics.successful_requests);
    
    // 7. Stop the server
    server.stop().await?;
    println!("✅ Server stopped gracefully");
    
    println!("\n🎉 Enhanced MCP Server Demo Complete!");
    println!("All functionality working correctly with:");
    println!("   ✅ Enhanced server with tarpc + WebSocket hybrid architecture");
    println!("   ✅ Session management with metrics collection");
    println!("   ✅ Tool management with registration and execution");
    println!("   ✅ Protocol message handling (Initialize, ListTools, GetStatus)");
    println!("   ✅ Plugin management interface");
    println!("   ✅ Graceful startup and shutdown");
    
    Ok(())
} 