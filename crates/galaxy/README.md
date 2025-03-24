# Galaxy MCP Adapter

This crate provides an adapter for integrating Galaxy bioinformatics tools with the Machine Context Protocol (MCP). It enables AI assistants to discover, execute, and orchestrate Galaxy tools through a standardized protocol.

## Implementation Status

The Galaxy MCP Adapter has been successfully implemented with the following components:

- ✅ Error handling system with comprehensive error types
- ✅ Configuration management with flexible options
- ✅ Galaxy API client for communicating with Galaxy servers
- ✅ Data models for Galaxy resources (tools, workflows, datasets, etc.)
- ✅ Plugin architecture for extending Galaxy functionality
- ✅ Adapter core for integrating with MCP protocol
- ✅ Tool discovery and execution functionality
- ✅ MCP protocol message handling
- ✅ Example code for common use cases
- 🔄 Testing infrastructure (in progress)
- 🔄 Documentation (in progress)

See [specs/galaxy/IMPLEMENTATION_STATUS.md](../../specs/galaxy/IMPLEMENTATION_STATUS.md) for detailed implementation status.

## Features

- **Tool Discovery**: AI assistants can discover and understand Galaxy tools
- **Parameter Mapping**: Galaxy tool parameters are translated to MCP tool definitions
- **Execution Management**: Tools can be executed and their results retrieved
- **Workflow Automation**: Complex workflows can be constructed and executed
- **Security Controls**: Secure authentication with Galaxy API
- **Data Management**: Complete data handling from upload to processing
- **Configuration Flexibility**: Adaptable to different Galaxy instances
- **Plugin System**: Extensible architecture with plugin support

## Usage

### Basic Usage

```rust
use galaxy::{create_adapter, GalaxyConfig};

#[tokio::main]
async fn main() -> Result<(), galaxy::Error> {
    // Create a new adapter with default configuration
    let adapter = create_adapter()?;
    
    // List available tools
    let tools = adapter.list_tools().await?;
    println!("Found {} tools", tools.len());
    
    // Execute a tool
    let parameters = std::collections::HashMap::new();
    let job_id = adapter.execute_tool("tool_id", &parameters).await?;
    println!("Tool execution started: {}", job_id);
    
    // Check job status
    let status = adapter.get_job_status(&job_id).await?;
    println!("Job status: {:?}", status);
    
    // Get results when complete
    if let galaxy::models::tool::JobState::Completed = status {
        let outputs = adapter.get_job_results(&job_id).await?;
        println!("Job outputs: {:?}", outputs);
    }
    
    Ok(())
}
```

### Custom Configuration

```rust
use galaxy::{create_adapter_with_config, GalaxyConfig};

let config = GalaxyConfig::new("https://custom-galaxy.org/api")
    .with_api_key("your-api-key")
    .with_timeout(60); // 60 seconds

let adapter = create_adapter_with_config(config)?;
```

### Plugin System

The Galaxy adapter supports a plugin system that allows extending its functionality:

```rust
use std::sync::Arc;
use galaxy::{
    create_adapter_with_config, 
    create_plugin_manager,
    create_tool_plugin,
    GalaxyConfig
};

#[tokio::main]
async fn main() -> Result<(), galaxy::Error> {
    // Create adapter
    let config = GalaxyConfig::default();
    let adapter = Arc::new(create_adapter_with_config(config)?);
    
    // Create plugin manager
    let mut plugin_manager = create_plugin_manager(Arc::clone(&adapter));
    
    // Create and register a tool plugin
    let tool_plugin = create_tool_plugin(
        "GenomicsTools",
        "1.0.0",
        "Genomics analysis tools"
    );
    plugin_manager.register_plugin(Arc::new(tool_plugin)).await?;
    
    // Find plugins by capability
    let tool_plugins = plugin_manager.get_plugins_by_capability("galaxy-tool");
    println!("Found {} tool plugins", tool_plugins.len());
    
    Ok(())
}
```

### MCP Integration

When the `mcp-integration` feature is enabled, the adapter can handle MCP protocol messages:

```rust
use galaxy::{create_adapter, GalaxyConfig};
use mcp::protocol::Message;

#[tokio::main]
async fn main() -> Result<(), galaxy::Error> {
    let mut adapter = create_adapter()?;
    
    // Initialize MCP integration
    adapter.initialize_mcp()?;
    
    // Handle an MCP message
    let message = Message::new_tool_discovery_request();
    let response = adapter.handle_message(message).await?;
    
    println!("Response: {:?}", response);
    
    Ok(())
}
```

## Crate Structure

```
crates/galaxy/
├── src/
│   ├── adapter/        # Core adapter implementation
│   ├── api/            # Galaxy API endpoint definitions 
│   ├── client/         # HTTP client for Galaxy API
│   ├── config/         # Configuration management
│   ├── data/           # Data handling utilities
│   ├── error/          # Error handling
│   ├── models/         # Data models
│   │   ├── tool.rs     # Galaxy tool models
│   │   ├── workflow.rs # Workflow models
│   │   ├── dataset.rs  # Dataset models
│   │   ├── job.rs      # Job models
│   │   ├── history.rs  # History models
│   │   └── library.rs  # Library models
│   ├── plugin/         # Plugin architecture
│   │   ├── mod.rs              # Plugin trait definitions
│   │   ├── default_plugin.rs   # Default plugin implementation
│   │   ├── tool_plugin.rs      # Tool plugin implementation
│   │   ├── workflow_plugin.rs  # Workflow plugin implementation
│   │   └── dataset_plugin.rs   # Dataset plugin implementation
│   ├── security/       # Authentication and security
│   ├── tools/          # Tool-specific functionality
│   ├── utils/          # Utility functions
│   ├── workflows/      # Workflow-specific functionality
│   └── lib.rs          # Crate entry point
├── examples/           # Usage examples
│   ├── list_tools.rs        # Tool discovery example
│   ├── execute_tool.rs      # Tool execution example
│   ├── plugin_example.rs    # Plugin system example
│   └── mcp_integration.rs   # MCP integration example
└── tests/              # Integration tests
```

## Design

The crate is designed as an adapter that leverages the existing MCP and context crates from the Squirrel MCP project. It follows the adapter pattern for dependency injection to maintain clean integration with the surrounding codebase.

### Components

- **Adapter**: Core component that bridges MCP and Galaxy
- **Client**: HTTP client for Galaxy API communication
- **Models**: Data models representing Galaxy objects
- **Configuration**: Configuration management
- **Plugin System**: Extensible architecture with plugin support
- **Error**: Comprehensive error handling

### Plugin Types

The Galaxy adapter supports the following plugin types:

- **GalaxyPlugin**: Base plugin interface for all Galaxy plugins
- **GalaxyToolPlugin**: Plugin for Galaxy tool-related functionality
- **GalaxyWorkflowPlugin**: Plugin for Galaxy workflow-related functionality
- **GalaxyDatasetPlugin**: Plugin for Galaxy dataset-related functionality

## Features

- `mcp-integration`: Enables integration with MCP protocol (enabled by default)
- `test-utils`: Enables testing utilities

## License

This project is licensed under the MIT License.

## Credits

Developed by DataScienceBioLab. 