---
title: "Galaxy MCP Adapter Deployment Guide"
description: "Guide for integrating and deploying the Galaxy MCP adapter crate"
version: "0.1.0"
last_updated: "2025-03-27"
status: "draft"
owners:
  primary: ["DataScienceBioLab", "mcp-team"]
  reviewers: ["core-team", "devops-team"]
---

# Galaxy MCP Adapter Deployment Guide

## 1. Overview

This guide covers the integration and deployment of the Galaxy MCP adapter crate within your Rust applications. As a crate-based implementation, the deployment process is significantly simplified compared to standalone service deployment.

## 2. Integration Methods

### 2.1 Cargo Dependency

The simplest way to integrate the Galaxy MCP adapter is as a Cargo dependency in your Rust project:

```toml
# Cargo.toml
[dependencies]
galaxy-mcp = { path = "../path/to/galaxy-mcp" }  # For local development
# or
galaxy-mcp = "0.1.0"  # When published to crates.io
```

### 2.2 Workspace Member

For development within a workspace:

```toml
# Root Cargo.toml
[workspace]
members = [
    "crates/mcp",
    "crates/context",
    "crates/galaxy-mcp",
    # Other crates...
]
```

## 3. Prerequisites

Before using the Galaxy MCP adapter, ensure:

1. **Galaxy Access**:
   - Access to a Galaxy instance
   - API key with appropriate permissions
   - Network connectivity to the Galaxy server

2. **Rust Environment**:
   - Rust toolchain (1.65.0 or newer)
   - Cargo package manager
   - Development libraries for required dependencies

## 4. Application Integration

### 4.1 Basic Integration

```rust
use galaxy_mcp::{GalaxyAdapter, GalaxyAdapterConfig};
use mcp::Protocol;
use context::Manager;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create configuration
    let config = GalaxyAdapterConfig::default()
        .with_url("https://usegalaxy.org/api")
        .with_api_key(std::env::var("GALAXY_API_KEY").ok());
    
    // Create adapter
    let adapter = GalaxyAdapter::new(config);
    
    // Use the adapter in your application
    let tools = adapter.list_tools()?;
    println!("Available Galaxy tools: {}", tools.len());
    
    Ok(())
}
```

### 4.2 Integration with Existing MCP Handler

```rust
use galaxy_mcp::{GalaxyAdapter, GalaxyAdapterConfig};
use mcp::{Protocol, Message};

async fn handle_mcp_message(message: Message) -> Result<Message, Error> {
    // Create Galaxy adapter
    let config = GalaxyAdapterConfig::from_env();
    let adapter = GalaxyAdapter::new(config);
    
    // Based on message type, delegate to Galaxy adapter
    match message.message_type() {
        MessageType::ToolDiscovery => adapter.handle_tool_discovery(message).await,
        MessageType::ToolExecution => adapter.handle_tool_execution(message).await,
        MessageType::JobStatus => adapter.handle_job_status(message).await,
        // Other message types...
        _ => Err(Error::UnsupportedMessageType),
    }
}
```

## 5. Configuration

### 5.1 Configuration File

Create a configuration file for your application that includes Galaxy adapter settings:

```toml
# config.toml
[mcp]
log_level = "info"

[context]
data_dir = "./data"

[galaxy]
url = "https://usegalaxy.org/api"
timeout = 30
max_retries = 3
```

### 5.2 Environment Variables

Configure the adapter using environment variables:

```bash
export GALAXY_MCP_URL="https://usegalaxy.org/api"
export GALAXY_MCP_API_KEY="your-api-key-here"
export GALAXY_MCP_TIMEOUT=30
export GALAXY_MCP_MAX_RETRIES=3
```

## 6. Development Environment

### 6.1 Local Development Setup

For local development:

1. Clone the repository containing the Galaxy MCP adapter:
   ```bash
   git clone https://github.com/your-org/galaxy-mcp.git
   cd galaxy-mcp
   ```

2. Build the crate:
   ```bash
   cargo build
   ```

3. Run tests:
   ```bash
   cargo test
   ```

### 6.2 Local Galaxy Instance

For testing with a local Galaxy instance:

1. Start a local Galaxy server with Docker:
   ```bash
   docker run -d -p 8080:80 quay.io/bgruening/galaxy
   ```

2. Configure the adapter to use the local instance:
   ```rust
   let config = GalaxyAdapterConfig::default()
       .with_url("http://localhost:8080/api")
       .with_api_key("admin");
   ```

## 7. Testing Your Integration

### 7.1 Unit Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use galaxy_mcp::{GalaxyAdapter, GalaxyAdapterConfig};
    
    #[test]
    fn test_adapter_creation() {
        let config = GalaxyAdapterConfig::default()
            .with_url("https://usegalaxy.org/api");
        let adapter = GalaxyAdapter::new(config);
        assert!(adapter.is_ok());
    }
    
    #[tokio::test]
    async fn test_tool_listing() {
        // Use a mock Galaxy API for testing
        let mock_api = MockGalaxyApi::new()
            .with_tools(vec![mock_tool_1(), mock_tool_2()]);
        
        let adapter = GalaxyAdapter::with_api(mock_api);
        let tools = adapter.list_tools().await.unwrap();
        
        assert_eq!(tools.len(), 2);
    }
}
```

### 7.2 Integration Testing

```rust
#[cfg(test)]
mod integration_tests {
    use galaxy_mcp::{GalaxyAdapter, GalaxyAdapterConfig};
    
    #[tokio::test]
    #[ignore] // Requires a running Galaxy instance
    async fn test_with_real_galaxy() {
        let config = GalaxyAdapterConfig::default()
            .with_url(std::env::var("TEST_GALAXY_URL").unwrap())
            .with_api_key(std::env::var("TEST_GALAXY_API_KEY").ok());
        
        let adapter = GalaxyAdapter::new(config).unwrap();
        let tools = adapter.list_tools().await.unwrap();
        
        assert!(!tools.is_empty());
    }
}
```

## 8. Production Deployment

### 8.1 Application Packaging

When deploying an application using the Galaxy MCP adapter:

1. Ensure all dependencies are properly specified in your Cargo.toml
2. Build an optimized release binary:
   ```bash
   cargo build --release
   ```
3. Consider using a container for deployment:
   ```dockerfile
   FROM rust:1.65 as builder
   WORKDIR /app
   COPY . .
   RUN cargo build --release
   
   FROM debian:bullseye-slim
   COPY --from=builder /app/target/release/your-app /usr/local/bin/
   CMD ["your-app"]
   ```

### 8.2 Configuration Management

In production environments:

1. Use environment variables for sensitive values (API keys)
2. Mount configuration files from secure locations
3. Consider using a configuration management system

Example Docker run command:
```bash
docker run -d \
  -e GALAXY_MCP_URL=https://usegalaxy.org/api \
  -e GALAXY_MCP_API_KEY=your-api-key-here \
  -v /path/to/config:/app/config \
  your-app-image
```

### 8.3 Security Considerations

1. **API Key Management**: 
   - Store API keys securely
   - Rotate keys periodically
   - Use environment variables rather than configuration files

2. **Network Security**:
   - Use HTTPS for Galaxy API communication
   - Implement proper timeout and retry handling
   - Consider restricting network access to only required endpoints

## 9. Troubleshooting

### 9.1 Common Issues

| Issue | Possible Cause | Solution |
|-------|----------------|----------|
| Connection timeout | Network latency or Galaxy server load | Increase the timeout setting |
| Authentication failure | Invalid or expired API key | Check API key and regenerate if needed |
| Resource not found | Tool ID or workflow ID doesn't exist | Verify the ID exists in Galaxy |
| Rate limiting | Too many requests to Galaxy API | Implement backoff strategy |

### 9.2 Logging and Diagnostics

Enable detailed logging for troubleshooting:

```rust
use env_logger;

fn main() {
    // Initialize logger with debug level for galaxy-mcp
    env_logger::Builder::from_default_env()
        .filter(Some("galaxy_mcp"), log::LevelFilter::Debug)
        .init();
    
    // Rest of your application...
}
```

### 9.3 Performance Optimization

For improved performance:

1. Enable connection pooling in the adapter configuration
2. Utilize appropriate caching settings
3. Consider batch operations for multiple tool executions
4. Monitor resource usage and adjust configuration accordingly

## 10. Examples

### 10.1 Basic Tool Execution

```rust
use galaxy_mcp::{GalaxyAdapter, GalaxyAdapterConfig, ToolInput};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create adapter
    let config = GalaxyAdapterConfig::from_env();
    let adapter = GalaxyAdapter::new(config)?;
    
    // Define tool inputs
    let mut inputs = HashMap::new();
    inputs.insert("input1".to_string(), ToolInput::File("dataset_123".to_string()));
    inputs.insert("param1".to_string(), ToolInput::String("value1".to_string()));
    
    // Execute tool
    let job_id = adapter.execute_tool("toolshed.g2.bx.psu.edu/repos/devteam/fastqc/fastqc/0.73", inputs).await?;
    println!("Job started with ID: {}", job_id);
    
    // Wait for completion
    adapter.wait_for_job(job_id).await?;
    
    // Get results
    let results = adapter.get_job_results(job_id).await?;
    println!("Job completed with {} output datasets", results.outputs.len());
    
    Ok(())
}
```

### 10.2 Workflow Execution

```rust
use galaxy_mcp::{GalaxyAdapter, GalaxyAdapterConfig, WorkflowInput};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create adapter
    let config = GalaxyAdapterConfig::from_env();
    let adapter = GalaxyAdapter::new(config)?;
    
    // Define workflow inputs
    let mut inputs = HashMap::new();
    inputs.insert("0".to_string(), WorkflowInput::Dataset("dataset_123".to_string()));
    
    // Execute workflow
    let invocation_id = adapter.run_workflow("workflow_id_123", inputs).await?;
    println!("Workflow started with invocation ID: {}", invocation_id);
    
    // Wait for completion
    adapter.wait_for_workflow(invocation_id).await?;
    
    // Get results
    let results = adapter.get_workflow_results(invocation_id).await?;
    println!("Workflow completed with {} outputs", results.outputs.len());
    
    Ok(())
}
```

## 11. Related Specifications

- [Galaxy MCP Integration Plan](galaxy-mcp-integration.md)
- [API Mapping](api-mapping.md)
- [Configuration Management](configuration-management.md)
- [Security Model](security-model.md)

<version>0.1.0</version> 