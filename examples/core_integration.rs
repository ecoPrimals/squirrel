use squirrel_mcp::integration::CoreMCPAdapter;
use squirrel_mcp::protocol::{MCPProtocol, MCPMessage, MessageType};
use squirrel_mcp::types::{CoreState, StateUpdate};
use squirrel_mcp::error::MCPResult;

// ... existing code ...

    let mcp_protocol = squirrel_mcp::protocol::InMemoryMCPProtocol::new();

// ... existing code ...

    let auth_manager = Arc::new(squirrel_mcp::security::SimpleAuthManager::new());
    let metrics = Arc::new(squirrel_mcp::metrics::MetricsCollector::new());
    let logger = squirrel_mcp::logging::Logger::new();

// ... existing code ... 