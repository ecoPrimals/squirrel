---
title: CLI Integration Testing Strategy
version: 1.0.0
date: 2024-06-25
status: active
priority: high
owner: DataScienceBioLab
related:
  - TESTING.md
  - core-mcp-integration.md
  - plugin-mcp-integration.md
---

# CLI Integration Testing Strategy

## Overview

This document outlines the comprehensive strategy for testing the integration between the Squirrel CLI crate and other system components. It serves as a guide for the integration team to implement robust cross-crate tests that verify the correct interaction between the CLI and other parts of the Squirrel system.

## Integration Points

The CLI crate has several critical integration points with other system components:

1. **CLI-Commands Integration**: The CLI depends on the commands crate for command definition, registration, and execution.
2. **CLI-Core Integration**: The CLI uses core functionality for error handling, configuration, and utility functions.
3. **CLI-MCP Integration**: The CLI interacts with the MCP system for remote command execution and machine context management.
4. **CLI-Plugins Integration**: The CLI loads and manages plugins that provide additional commands and functionality.

## Test Environment Setup

### 1. Cross-Crate Test Harness

Create a dedicated test harness that can instantiate and connect components from multiple crates:

```rust
// integration_tests/src/harness.rs
use std::sync::Arc;
use tokio::sync::Mutex;

use squirrel_cli::command_adapter::{CommandAdapterTrait, RegistryAdapter};
use squirrel_commands::CommandRegistry;
use squirrel_core::config::ConfigManager;
use squirrel_mcp::client::McpClient;
use squirrel_plugins::PluginManager;

/// Test harness for CLI integration tests
pub struct CliIntegrationTestHarness {
    /// Command registry
    pub registry: Arc<Mutex<CommandRegistry>>,
    /// CLI adapter
    pub adapter: Arc<dyn CommandAdapterTrait>,
    /// Config manager
    pub config: Arc<ConfigManager>,
    /// MCP client (optional)
    pub mcp_client: Option<Arc<McpClient>>,
    /// Plugin manager (optional)
    pub plugin_manager: Option<Arc<PluginManager>>,
}

impl CliIntegrationTestHarness {
    /// Create a new test harness with minimal components
    pub async fn new() -> Self {
        let registry = Arc::new(Mutex::new(CommandRegistry::new()));
        let adapter = Arc::new(RegistryAdapter::new(registry.clone()));
        let config = Arc::new(ConfigManager::new());
        
        Self {
            registry,
            adapter,
            config,
            mcp_client: None,
            plugin_manager: None,
        }
    }
    
    /// Add MCP client to the test harness
    pub async fn with_mcp_client(mut self) -> Self {
        let mcp_client = Arc::new(McpClient::new_for_testing());
        self.mcp_client = Some(mcp_client);
        self
    }
    
    /// Add plugin manager to the test harness
    pub async fn with_plugin_manager(mut self) -> Self {
        let plugin_manager = Arc::new(PluginManager::new());
        self.plugin_manager = Some(plugin_manager);
        self
    }
    
    /// Execute a command through the CLI adapter
    pub async fn execute_command(&self, command: &str, args: Vec<String>) -> Result<String, Box<dyn std::error::Error>> {
        let result = self.adapter.execute_command(command, args).await?;
        Ok(result)
    }
}
```

### 2. Test Isolation

Ensure each integration test runs in isolation by creating a fresh test environment for each test:

```rust
// integration_tests/src/fixtures.rs
use std::env;
use std::path::PathBuf;
use tempfile::TempDir;

pub struct TestIsolation {
    pub temp_dir: TempDir,
    pub original_dir: PathBuf,
}

impl TestIsolation {
    pub fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = env::current_dir().unwrap();
        
        // Change to temp directory for test isolation
        env::set_current_dir(&temp_dir.path()).unwrap();
        
        Self {
            temp_dir,
            original_dir,
        }
    }
}

impl Drop for TestIsolation {
    fn drop(&mut self) {
        // Restore original directory
        let _ = env::set_current_dir(&self.original_dir);
    }
}
```

## Test Categories

### 1. CLI-Commands Integration Tests

Verify that the CLI correctly interacts with the commands crate:

```rust
#[tokio::test]
async fn test_cli_commands_registry_integration() {
    let harness = CliIntegrationTestHarness::new().await;
    
    // Register a test command in the registry
    harness.registry.lock().await.register(
        "test_command",
        Arc::new(TestCommand::new("test_command", "Test command", "Test result")),
    ).unwrap();
    
    // Execute the command through the CLI adapter
    let result = harness.execute_command("test_command", vec![]).await.unwrap();
    
    // Verify expected output
    assert_eq!(result, "Test result");
}

#[tokio::test]
async fn test_cli_commands_error_propagation() {
    let harness = CliIntegrationTestHarness::new().await;
    
    // Register a command that returns an error
    harness.registry.lock().await.register(
        "error_command",
        Arc::new(ErrorCommand::new("error_command", "Error command", "Simulated error")),
    ).unwrap();
    
    // Execute the command and expect an error
    let result = harness.execute_command("error_command", vec![]).await;
    assert!(result.is_err());
    
    // Verify error details
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Simulated error"));
}
```

### 2. CLI-Core Integration Tests

Verify that the CLI correctly uses core functionality:

```rust
#[tokio::test]
async fn test_cli_core_config_integration() {
    let harness = CliIntegrationTestHarness::new().await;
    
    // Create a test configuration
    harness.config.set("test.key", "test_value").unwrap();
    
    // Register a command that uses configuration
    harness.registry.lock().await.register(
        "config_command",
        Arc::new(ConfigTestCommand::new(harness.config.clone())),
    ).unwrap();
    
    // Execute the command
    let result = harness.execute_command("config_command", vec!["test.key".to_string()]).await.unwrap();
    
    // Verify configuration integration
    assert_eq!(result, "test_value");
}

#[tokio::test]
async fn test_cli_core_error_handling() {
    let harness = CliIntegrationTestHarness::new().await;
    
    // Register a command that triggers a core error
    harness.registry.lock().await.register(
        "core_error_command",
        Arc::new(CoreErrorCommand::new()),
    ).unwrap();
    
    // Execute the command and expect a properly wrapped error
    let result = harness.execute_command("core_error_command", vec![]).await;
    assert!(result.is_err());
    
    // Verify error wrapping
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Core error"));
}
```

### 3. CLI-MCP Integration Tests

Verify that the CLI correctly interacts with the MCP system:

```rust
#[tokio::test]
async fn test_cli_mcp_client_integration() {
    let harness = CliIntegrationTestHarness::new().await.with_mcp_client().await;
    
    // Register the MCP command
    harness.registry.lock().await.register(
        "mcp_command",
        Arc::new(McpCommand::new(harness.mcp_client.clone().unwrap())),
    ).unwrap();
    
    // Execute the MCP command
    let result = harness.execute_command("mcp_command", vec!["status".to_string()]).await.unwrap();
    
    // Verify MCP integration
    assert!(result.contains("MCP Status"));
}

#[tokio::test]
async fn test_cli_mcp_authentication() {
    let harness = CliIntegrationTestHarness::new().await.with_mcp_client().await;
    let mcp_client = harness.mcp_client.clone().unwrap();
    
    // Configure authentication
    mcp_client.set_auth_token("test_token").await;
    
    // Register a secured command
    harness.registry.lock().await.register(
        "secured_command",
        Arc::new(SecuredCommand::new(mcp_client.clone())),
    ).unwrap();
    
    // Execute with authentication
    let result = harness.execute_command("secured_command", vec![]).await.unwrap();
    assert!(result.contains("Authentication successful"));
    
    // Execute with invalid authentication
    mcp_client.set_auth_token("invalid_token").await;
    let result = harness.execute_command("secured_command", vec![]).await;
    assert!(result.is_err());
}
```

### 4. CLI-Plugins Integration Tests

Verify that the CLI correctly loads and manages plugins:

```rust
#[tokio::test]
async fn test_cli_plugin_lifecycle() {
    let test_isolation = TestIsolation::new();
    let harness = CliIntegrationTestHarness::new().await.with_plugin_manager().await;
    
    // Create a test plugin
    create_test_plugin("test_plugin", &test_isolation.temp_dir);
    
    // Register the plugin command
    harness.registry.lock().await.register(
        "plugin_command",
        Arc::new(PluginCommand::new(harness.plugin_manager.clone().unwrap())),
    ).unwrap();
    
    // Load the plugin
    let result = harness.execute_command(
        "plugin_command", 
        vec!["load".to_string(), "test_plugin".to_string()]
    ).await.unwrap();
    assert!(result.contains("Plugin loaded"));
    
    // Verify plugin commands are registered
    let commands = harness.adapter.list_commands().await.unwrap();
    assert!(commands.contains(&"test_plugin_command".to_string()));
    
    // Execute a plugin command
    let result = harness.execute_command("test_plugin_command", vec![]).await.unwrap();
    assert!(result.contains("Plugin command executed"));
    
    // Unload the plugin
    let result = harness.execute_command(
        "plugin_command", 
        vec!["unload".to_string(), "test_plugin".to_string()]
    ).await.unwrap();
    assert!(result.contains("Plugin unloaded"));
    
    // Verify plugin commands are removed
    let commands = harness.adapter.list_commands().await.unwrap();
    assert!(!commands.contains(&"test_plugin_command".to_string()));
}
```

## Mock Implementations

Create mock implementations of dependencies to simplify testing:

### 1. Mock Commands

```rust
#[derive(Clone)]
struct TestCommand {
    name: String,
    description: String,
    result: String,
}

impl TestCommand {
    fn new(name: &str, description: &str, result: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            result: result.to_string(),
        }
    }
}

#[async_trait]
impl TestCommand for TestCommand {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    async fn execute(&self, _args: Vec<String>) -> Result<String, AdapterError> {
        Ok(self.result.clone())
    }

    fn parser(&self) -> clap::Command {
        clap::Command::new(&self.name)
            .about(&self.description)
    }
}
```

### 2. Mock MCP Client

```rust
/// Mock MCP Client for testing
pub struct MockMcpClient {
    auth_token: Arc<Mutex<Option<String>>>,
    responses: Arc<Mutex<HashMap<String, String>>>,
}

impl MockMcpClient {
    pub fn new() -> Self {
        let mut responses = HashMap::new();
        responses.insert("status".to_string(), "MCP Status: Running".to_string());
        
        Self {
            auth_token: Arc::new(Mutex::new(None)),
            responses: Arc::new(Mutex::new(responses)),
        }
    }
    
    pub async fn set_auth_token(&self, token: &str) {
        let mut auth = self.auth_token.lock().await;
        *auth = Some(token.to_string());
    }
    
    pub async fn get_auth_token(&self) -> Option<String> {
        let auth = self.auth_token.lock().await;
        auth.clone()
    }
    
    pub async fn add_response(&self, command: &str, response: &str) {
        let mut responses = self.responses.lock().await;
        responses.insert(command.to_string(), response.to_string());
    }
    
    pub async fn execute(&self, command: &str) -> Result<String, String> {
        let auth = self.auth_token.lock().await;
        
        // Validate authentication for secure commands
        if command.starts_with("secure_") && (auth.is_none() || auth.as_ref().unwrap() != "test_token") {
            return Err("Authentication failed".to_string());
        }
        
        let responses = self.responses.lock().await;
        if let Some(response) = responses.get(command) {
            Ok(response.clone())
        } else {
            Err(format!("Unknown command: {}", command))
        }
    }
}
```

## Performance Testing

Measure and optimize the performance of cross-crate calls:

```rust
#[tokio::test]
async fn test_cli_cross_crate_performance() {
    let harness = CliIntegrationTestHarness::new().await;
    
    // Register a performance test command
    harness.registry.lock().await.register(
        "perf_command",
        Arc::new(TestCommand::new("perf_command", "Performance test", "Performance result")),
    ).unwrap();
    
    // Measure execution time
    let start = std::time::Instant::now();
    
    // Execute the command multiple times
    for _ in 0..100 {
        harness.execute_command("perf_command", vec![]).await.unwrap();
    }
    
    let duration = start.elapsed();
    println!("Average execution time: {:?}", duration / 100);
    
    // Assert reasonable performance
    assert!(duration < std::time::Duration::from_secs(1));
}
```

## Test Organization

Organize tests in a way that makes them easy to run and maintain:

```
integration_tests/
├── src/
│   ├── harness.rs           # Test harness
│   ├── fixtures.rs          # Test fixtures
│   ├── mocks/               # Mock implementations
│   │   ├── commands.rs      # Mock commands
│   │   ├── mcp.rs           # Mock MCP client
│   │   └── plugins.rs       # Mock plugins
│   ├── cli_commands/        # CLI-Commands tests
│   ├── cli_core/            # CLI-Core tests
│   ├── cli_mcp/             # CLI-MCP tests
│   ├── cli_plugins/         # CLI-Plugins tests
│   └── performance/         # Performance tests
├── Cargo.toml
└── README.md
```

## Implementation Plan

### Phase 1: Setup (Week 1-2)
1. Create integration test infrastructure
2. Implement test harness and fixtures
3. Create basic mock implementations
4. Set up CI pipeline for integration tests

### Phase 2: Basic Tests (Week 3-4)
1. Implement CLI-Commands integration tests
2. Implement CLI-Core integration tests
3. Create and test cross-crate interfaces

### Phase 3: Advanced Tests (Week 5-6)
1. Implement CLI-MCP integration tests
2. Implement CLI-Plugins integration tests
3. Add performance testing

### Phase 4: Refinement (Week 7-8)
1. Improve test coverage
2. Optimize performance bottlenecks
3. Document integration patterns and best practices

## Best Practices

1. **Use Interfaces**: Interact with other crates through well-defined interfaces
2. **Minimize Dependencies**: Keep dependencies between crates to a minimum
3. **Error Handling**: Ensure errors are properly propagated and transformed
4. **Async Consistency**: Maintain consistent async patterns across crate boundaries
5. **Test Isolation**: Each test should run in isolation
6. **Mock Dependencies**: Use mocks for external dependencies
7. **Performance Awareness**: Be conscious of cross-crate call performance
8. **Concurrent Testing**: Test behavior under concurrent access

## Conclusion

This integration testing strategy provides a comprehensive approach to verifying the interaction between the CLI crate and other components of the Squirrel system. By following this plan, the integration team can ensure that the CLI works seamlessly with other parts of the system, providing a reliable and performant command-line interface.

<version>1.0.0</version> 