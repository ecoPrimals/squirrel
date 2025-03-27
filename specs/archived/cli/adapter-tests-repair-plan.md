---
description: Plan for repairing and enhancing adapter module tests
date: 2024-03-26
status: planning
priority: high
owner: DataScienceBioLab
---

# Adapter Tests Repair Plan

## Context

The adapter module in the CLI project is experiencing several test failures due to type mismatches, interface incompatibilities, and implementation issues. This document outlines a comprehensive plan to repair the existing tests and establish a robust testing framework for the adapter module.

## Current Issues

Based on our analysis, we've identified these key issues:

1. **Type System Conflicts**:
   - Multiple definitions of `CommandError` across crates
   - Inconsistent usage of `CommandError` types
   - Conversion issues between error types

2. **Interface Incompatibilities**:
   - Method signature mismatches in adapter implementations
   - Missing trait implementations for adapter types
   - Incorrect function parameter types (particularly command arguments)

3. **Borrowing/Ownership Problems**:
   - Borrowed data escaping method scope in `parser()` implementations
   - Issues with lifetime annotations
   - Issues with `Clone` vs borrowing in various contexts

4. **Async Function Handling**:
   - Improper handling of futures from async functions
   - Missing `.await` calls in test cases
   - Incorrect use of `unwrap()` on `Pin<Box<dyn Future<...>>>`

5. **Structural Issues**:
   - Mixed concerns between test modules
   - Missing mock implementations for testing
   - Unclear test boundaries

## Repair Strategy

### Phase 1: Isolated Testing Infrastructure

1. **Create Completely Isolated Test Module**:
   - Location: `crates/cli/src/commands/adapter/isolated_tests.rs`
   - Purpose: Testing adapter concepts without dependencies on problematic code
   - Components:
     - Minimal `Command` trait implementation
     - Mock adapter implementations
     - Basic test cases

2. **Implement Mock Components**:
   - `MockCommand` with simple implementation
   - `MockAdapter` trait with minimal interface
   - `MockRegistry` for command registration

3. **Basic Test Cases**:
   - Command registration
   - Command execution
   - Error handling
   - Help text retrieval

### Phase 2: Address Core Issues

1. **Fix Type System Conflicts**:
   - Implement proper type conversions between error types
   - Use `Into` trait for error conversions
   - Fix signatures to use consistent error types

2. **Resolve Interface Incompatibilities**:
   - Update method signatures to match trait requirements
   - Implement missing trait methods
   - Fix parameter types (particularly `Vec<String>` vs `&[String]`)

3. **Fix Borrowing/Ownership Issues**:
   - Implement proper `Clone` where needed
   - Fix lifetimes in parser methods
   - Resolve borrowed data escaping method scope

4. **Address Async Function Handling**:
   - Properly await async functions in tests
   - Fix `unwrap()` usage on futures
   - Use proper async test patterns

### Phase 3: Comprehensive Test Suite

1. **Command Registry Adapter Tests**:
   - Test command registration
   - Test command execution
   - Test help text retrieval
   - Test error handling

2. **MCP Adapter Tests**:
   - Test authentication
   - Test command execution
   - Test error handling
   - Test protocol integration

3. **Plugin Adapter Tests**:
   - Test plugin loading
   - Test command execution
   - Test error handling
   - Test plugin lifecycle

4. **Integration Tests**:
   - Test adapter interoperability
   - Test end-to-end command execution
   - Test error propagation
   - Test help system

## Implementation Plan

### Task 1: Fix `isolated_tests.rs`

```rust
// Isolated test module with no external dependencies
mod isolated_tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    // Simple Command trait with minimal interface
    pub trait Command {
        fn name(&self) -> &str;
        fn description(&self) -> &str;
        fn execute(&self, args: Vec<String>) -> Result<String, String>;
    }

    // Mock command implementation
    pub struct MockCommand {
        name: String,
        description: String,
        result: String,
    }

    impl MockCommand {
        pub fn new(name: &str, description: &str, result: &str) -> Self {
            Self {
                name: name.to_string(),
                description: description.to_string(),
                result: result.to_string(),
            }
        }
    }

    impl Command for MockCommand {
        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            &self.description
        }

        fn execute(&self, args: Vec<String>) -> Result<String, String> {
            if args.is_empty() {
                Ok(self.result.clone())
            } else {
                Ok(format!("{}: {}", self.result, args.join(" ")))
            }
        }
    }

    // Mock adapter trait
    pub trait MockAdapter {
        fn register_command(&mut self, command: Arc<dyn Command>) -> Result<(), String>;
        fn execute_command(&self, name: &str, args: Vec<String>) -> Result<String, String>;
        fn get_help(&self, name: &str) -> Result<String, String>;
        fn list_commands(&self) -> Result<Vec<String>, String>;
    }

    // Simple adapter implementation
    pub struct SimpleMockAdapter {
        commands: HashMap<String, Arc<dyn Command>>,
    }

    impl SimpleMockAdapter {
        pub fn new() -> Self {
            Self {
                commands: HashMap::new(),
            }
        }
    }

    impl MockAdapter for SimpleMockAdapter {
        fn register_command(&mut self, command: Arc<dyn Command>) -> Result<(), String> {
            let name = command.name().to_string();
            self.commands.insert(name, command);
            Ok(())
        }

        fn execute_command(&self, name: &str, args: Vec<String>) -> Result<String, String> {
            match self.commands.get(name) {
                Some(cmd) => cmd.execute(args),
                None => Err(format!("Command not found: {}", name)),
            }
        }

        fn get_help(&self, name: &str) -> Result<String, String> {
            match self.commands.get(name) {
                Some(cmd) => Ok(format!("{}: {}", cmd.name(), cmd.description())),
                None => Err(format!("Command not found: {}", name)),
            }
        }

        fn list_commands(&self) -> Result<Vec<String>, String> {
            Ok(self.commands.keys().cloned().collect())
        }
    }

    // Tests
    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_command_registration() {
            let mut adapter = SimpleMockAdapter::new();
            let command = Arc::new(MockCommand::new("test", "Test command", "Test result"));
            
            assert!(adapter.register_command(command).is_ok());
            assert_eq!(adapter.list_commands().unwrap(), vec!["test"]);
        }

        #[test]
        fn test_command_execution() {
            let mut adapter = SimpleMockAdapter::new();
            let command = Arc::new(MockCommand::new("test", "Test command", "Test result"));
            
            adapter.register_command(command).unwrap();
            
            let result = adapter.execute_command("test", vec![]).unwrap();
            assert_eq!(result, "Test result");
            
            let result = adapter.execute_command("test", vec!["arg1".to_string(), "arg2".to_string()]).unwrap();
            assert_eq!(result, "Test result: arg1 arg2");
        }

        #[test]
        fn test_help_retrieval() {
            let mut adapter = SimpleMockAdapter::new();
            let command = Arc::new(MockCommand::new("test", "Test command", "Test result"));
            
            adapter.register_command(command).unwrap();
            
            let help = adapter.get_help("test").unwrap();
            assert_eq!(help, "test: Test command");
        }

        #[test]
        fn test_command_not_found() {
            let adapter = SimpleMockAdapter::new();
            
            let result = adapter.execute_command("nonexistent", vec![]);
            assert!(result.is_err());
            
            let help = adapter.get_help("nonexistent");
            assert!(help.is_err());
        }
    }
}
```

### Task 2: Fix Error Type Conversions

Update `CommandError` implementations to allow proper conversion between error types from different crates:

```rust
// In crates/cli/src/commands/error.rs
impl From<squirrel_commands::CommandError> for CommandError {
    fn from(err: squirrel_commands::CommandError) -> Self {
        match err {
            squirrel_commands::CommandError::ValidationError(msg) => CommandError::ValidationError(msg),
            squirrel_commands::CommandError::ExecutionError(msg) => CommandError::ExecutionError(msg),
            squirrel_commands::CommandError::InvalidArguments(msg) => CommandError::InvalidArguments(msg),
            // Map other variants as needed
        }
    }
}

// Reverse conversion if needed
impl From<CommandError> for squirrel_commands::CommandError {
    fn from(err: CommandError) -> Self {
        match err {
            CommandError::ValidationError(msg) => squirrel_commands::CommandError::ValidationError(msg),
            CommandError::ExecutionError(msg) => squirrel_commands::CommandError::ExecutionError(msg),
            CommandError::InvalidArguments(msg) => squirrel_commands::CommandError::InvalidArguments(msg),
            // Map other variants as needed
        }
    }
}
```

### Task 3: Fix Async Test Implementation

Properly implement async tests for the adapter module:

```rust
#[tokio::test]
async fn test_registry_adapter_with_async() {
    // Setup test environment
    let registry = CommandRegistry::new();
    let adapter = Arc::new(CommandRegistryAdapter::new(Arc::new(Mutex::new(registry))));
    
    // Register test command
    let command = Arc::new(TestCommand::new("test", "Test command", "Test result"));
    adapter.register_command("test", command.clone()).await.expect("Failed to register command");
    
    // Test command execution
    let result = adapter.execute_command("test", vec![]).await.expect("Failed to execute command");
    assert_eq!(result, "Test result");
    
    // Test help retrieval
    let help = adapter.get_help("test").await.expect("Failed to get help");
    assert!(help.contains("Test command"));
    
    // Test command listing
    let commands = adapter.list_commands().await.expect("Failed to list commands");
    assert!(commands.contains(&"test".to_string()));
}
```

### Task 4: Fix Parser Method Lifetime Issues

Address the borrowed data escaping method scope issue in `parser()` implementations:

```rust
// Fix for the parser method to avoid borrowed data escaping
fn parser(&self) -> ClapCommand {
    // Clone the data to avoid borrowing issues
    let name = self.name().to_string();
    let description = self.description().to_string();
    
    ClapCommand::new(name)
        .about(description)
        .arg(Arg::new("args")
            .multiple_values(true)
            .help("Command arguments"))
}
```

## Testing Timeline

1. **Week 1: Infrastructure Setup**
   - Implement isolated test module
   - Fix error conversion implementations
   - Set up basic test cases

2. **Week 2: Core Functionality Testing**
   - Implement registry adapter tests
   - Implement MCP adapter tests
   - Fix async test implementations

3. **Week 3: Integration Testing**
   - Implement plugin adapter tests
   - Implement cross-adapter tests
   - Validate command execution flow

4. **Week 4: Finalization**
   - Performance testing
   - Edge case testing
   - Documentation

## Success Criteria

1. All test modules pass successfully
2. No type system errors or conflicts
3. Clean interface implementations
4. Proper async function handling
5. No borrowing/lifetime issues
6. Comprehensive test coverage for all adapters
7. Documentation of test patterns and conventions

## Next Steps

After implementing this plan, we should:

1. Create a test framework document for future adapter implementations
2. Establish guidelines for async testing patterns
3. Implement continuous integration tests for the adapter module
4. Develop regression tests for identified issues
5. Expand test coverage to include error handling edge cases

## MCP-Specific Testing Considerations

The Machine Context Protocol (MCP) adapter requires special attention in our testing strategy due to its role as a bridge between command execution and external machine communication.

### MCP Authentication Testing

The MCP adapter incorporates authentication mechanisms that must be properly tested:

```rust
#[tokio::test]
async fn test_mcp_authentication() {
    // Setup mock authentication provider
    let auth_provider = Arc::new(MockAuthProvider::new());
    let registry_adapter = Arc::new(MockRegistryAdapter::new());
    
    // Setup MCP adapter with auth provider
    let mcp_adapter = MockMcpAdapter::new(auth_provider, registry_adapter);
    
    // Test valid authentication
    let valid_request = McpRequest {
        command: "test".to_string(),
        args: vec![],
        credentials: Some(MockCredentials::new("valid_user", "valid_token")),
    };
    
    let response = mcp_adapter.handle_command(&valid_request).await;
    assert!(response.is_ok());
    
    // Test invalid authentication
    let invalid_request = McpRequest {
        command: "test".to_string(),
        args: vec![],
        credentials: Some(MockCredentials::new("invalid_user", "invalid_token")),
    };
    
    let response = mcp_adapter.handle_command(&invalid_request).await;
    assert!(response.is_err());
    assert!(response.unwrap_err().to_string().contains("Authentication failed"));
}
```

### MCP Protocol Message Testing

The MCP adapter must correctly handle protocol messages:

```rust
#[tokio::test]
async fn test_mcp_message_handling() {
    let adapter = setup_mcp_test_adapter().await;
    
    // Test command execution via MCP message
    let message = McpMessage::Command {
        id: "cmd-123".to_string(),
        command: "test".to_string(),
        args: vec!["arg1".to_string()],
    };
    
    let response = adapter.process_message(message).await.unwrap();
    
    match response {
        McpMessage::Response { id, status, result } => {
            assert_eq!(id, "cmd-123");
            assert_eq!(status, "success");
            assert!(result.contains("Test result: arg1"));
        }
        _ => panic!("Expected Response message"),
    }
    
    // Test error handling via MCP message
    let error_message = McpMessage::Command {
        id: "cmd-456".to_string(),
        command: "nonexistent".to_string(),
        args: vec![],
    };
    
    let error_response = adapter.process_message(error_message).await.unwrap();
    
    match error_response {
        McpMessage::Response { id, status, result } => {
            assert_eq!(id, "cmd-456");
            assert_eq!(status, "error");
            assert!(result.contains("Command not found"));
        }
        _ => panic!("Expected Response message"),
    }
}
```

### MCP Context Integration

The MCP adapter must properly integrate with the execution context:

```rust
#[tokio::test]
async fn test_mcp_context_integration() {
    let context = Arc::new(MockContext::new());
    let adapter = setup_mcp_test_adapter_with_context(context.clone()).await;
    
    // Test command that requires context
    let message = McpMessage::Command {
        id: "ctx-123".to_string(),
        command: "context_command".to_string(), 
        args: vec![],
    };
    
    let response = adapter.process_message(message).await.unwrap();
    
    match response {
        McpMessage::Response { status, result, .. } => {
            assert_eq!(status, "success");
            assert!(result.contains("Context accessed successfully"));
            
            // Verify context was accessed
            assert!(context.was_accessed());
        }
        _ => panic!("Expected Response message"),
    }
}
```

### MCP Security Considerations

To ensure the MCP adapter maintains proper security:

1. **Authentication Verification**:
   - Test authentication success/failure cases
   - Verify proper token validation
   - Test expired credentials

2. **Authorization Testing**:
   - Test command access based on user roles
   - Verify admin-only commands are protected
   - Test command restrictions

3. **Input Validation**:
   - Test command injection prevention
   - Verify argument sanitization
   - Test malformed request handling

These MCP-specific tests will ensure that our adapter properly implements the Machine Context Protocol while maintaining security and reliability in command execution.

## Plugin Adapter Testing Considerations

The Plugin Adapter represents another critical component of our system, providing extensibility through dynamically loaded plugins. Testing this adapter requires special attention to dynamic loading, lifecycle management, and integration.

### Plugin Lifecycle Testing

Test the complete lifecycle of plugins, from loading to unloading:

```rust
#[tokio::test]
async fn test_plugin_lifecycle() {
    // Setup test environment with plugin directory
    let plugin_dir = setup_test_plugin_directory().await;
    let plugin_adapter = PluginAdapter::new(plugin_dir.path());
    
    // Test plugin initialization
    assert!(plugin_adapter.initialize().await.is_ok());
    
    // Test plugin discovery
    let plugins = plugin_adapter.discover_plugins().await.unwrap();
    assert!(!plugins.is_empty());
    assert!(plugins.iter().any(|p| p.name() == "test_plugin"));
    
    // Test plugin loading
    assert!(plugin_adapter.load_plugin("test_plugin").await.is_ok());
    assert!(plugin_adapter.is_plugin_loaded("test_plugin"));
    
    // Test plugin commands
    let commands = plugin_adapter.get_commands_for_plugin("test_plugin").await.unwrap();
    assert!(!commands.is_empty());
    assert!(commands.iter().any(|c| c == "test_command"));
    
    // Test plugin unloading
    assert!(plugin_adapter.unload_plugin("test_plugin").await.is_ok());
    assert!(!plugin_adapter.is_plugin_loaded("test_plugin"));
    
    // Test cleanup
    assert!(plugin_adapter.shutdown().await.is_ok());
}
```

### Plugin Command Execution

Test executing commands via plugins:

```rust
#[tokio::test]
async fn test_plugin_command_execution() {
    let plugin_adapter = setup_test_plugin_adapter().await;
    
    // Load the test plugin
    plugin_adapter.load_plugin("test_plugin").await.unwrap();
    
    // Execute a command from the plugin
    let result = plugin_adapter.execute_command("test_plugin.test_command", vec![])
        .await
        .unwrap();
    
    assert!(result.contains("Plugin command executed successfully"));
    
    // Test with arguments
    let result_with_args = plugin_adapter
        .execute_command("test_plugin.test_command", vec!["arg1".to_string()])
        .await
        .unwrap();
    
    assert!(result_with_args.contains("arg1"));
    
    // Test nonexistent command
    let error = plugin_adapter
        .execute_command("test_plugin.nonexistent", vec![])
        .await;
    
    assert!(error.is_err());
    assert!(error.unwrap_err().to_string().contains("Command not found"));
}
```

### Plugin Error Handling

Test plugin error scenarios:

```rust
#[tokio::test]
async fn test_plugin_error_handling() {
    let plugin_adapter = setup_test_plugin_adapter().await;
    
    // Test loading nonexistent plugin
    let load_error = plugin_adapter.load_plugin("nonexistent_plugin").await;
    assert!(load_error.is_err());
    assert!(load_error.unwrap_err().to_string().contains("Plugin not found"));
    
    // Load the test plugin
    plugin_adapter.load_plugin("test_plugin").await.unwrap();
    
    // Test plugin command that triggers an error
    let execution_error = plugin_adapter
        .execute_command("test_plugin.error_command", vec![])
        .await;
    
    assert!(execution_error.is_err());
    assert!(execution_error.unwrap_err().to_string().contains("Plugin error"));
    
    // Test plugin that crashes during initialization
    let crash_error = plugin_adapter.load_plugin("crashing_plugin").await;
    assert!(crash_error.is_err());
    assert!(crash_error.unwrap_err().to_string().contains("Plugin crashed"));
}
```

### Plugin Security Testing

Ensure plugins operate securely:

```rust
#[tokio::test]
async fn test_plugin_security() {
    let plugin_adapter = setup_test_plugin_adapter().await;
    
    // Test unsigned plugin loading with security enforced
    plugin_adapter.set_require_signatures(true);
    
    let unsigned_load = plugin_adapter.load_plugin("unsigned_plugin").await;
    assert!(unsigned_load.is_err());
    assert!(unsigned_load.unwrap_err().to_string().contains("Signature verification failed"));
    
    // Test sandbox violation
    plugin_adapter.load_plugin("test_plugin").await.unwrap();
    
    let sandbox_violation = plugin_adapter
        .execute_command("test_plugin.sandbox_escape", vec![])
        .await;
    
    assert!(sandbox_violation.is_err());
    assert!(sandbox_violation.unwrap_err().to_string().contains("Sandbox violation"));
}
```

### Plugin Isolation Testing

Verify that plugins are properly isolated:

```rust
#[tokio::test]
async fn test_plugin_isolation() {
    let plugin_adapter = setup_test_plugin_adapter().await;
    
    // Load two plugins
    plugin_adapter.load_plugin("plugin_a").await.unwrap();
    plugin_adapter.load_plugin("plugin_b").await.unwrap();
    
    // Verify that plugin A can't access plugin B's data
    let isolation_test = plugin_adapter
        .execute_command("plugin_a.access_plugin_b", vec![])
        .await;
    
    assert!(isolation_test.is_err());
    assert!(isolation_test.unwrap_err().to_string().contains("Permission denied"));
    
    // Verify that unloading plugin A doesn't affect plugin B
    plugin_adapter.unload_plugin("plugin_a").await.unwrap();
    
    let plugin_b_result = plugin_adapter
        .execute_command("plugin_b.test_command", vec![])
        .await;
    
    assert!(plugin_b_result.is_ok());
}
```

## Integration Testing of Adapters

Beyond testing individual adapters, we must ensure they work together properly:

```rust
#[tokio::test]
async fn test_adapter_integration() {
    // Setup integrated test environment
    let registry_adapter = setup_registry_adapter().await;
    let mcp_adapter = setup_mcp_adapter_with_registry(registry_adapter.clone()).await;
    let plugin_adapter = setup_plugin_adapter_with_registry(registry_adapter.clone()).await;
    
    // Load plugin and register its commands
    plugin_adapter.load_plugin("test_plugin").await.unwrap();
    plugin_adapter.register_plugin_commands("test_plugin").await.unwrap();
    
    // Verify MCP adapter can execute plugin command
    let mcp_request = McpRequest {
        command: "test_plugin.test_command".to_string(),
        args: vec!["integration_test".to_string()],
        credentials: Some(valid_credentials()),
    };
    
    let mcp_response = mcp_adapter.handle_command(&mcp_request).await.unwrap();
    assert!(mcp_response.result.contains("integration_test"));
    
    // Verify plugin adapter can find registry commands
    let registry_command_result = plugin_adapter
        .execute_command("core.help", vec![])
        .await
        .unwrap();
    
    assert!(registry_command_result.contains("Available commands"));
}
```

These comprehensive tests will ensure that our plugin adapter maintains proper isolation, security, and functionality while integrating correctly with other components in the system.

## Implementation Timeline and Immediate Steps

### Week 1: Foundational Fixes (April 1-7)

1. **Day 1-2: Create Isolated Test Module**
   - Create `isolated_tests.rs` with completely independent test infrastructure
   - Implement simplified mock types for testing adapter concepts
   - Run initial tests to verify isolation works

2. **Day 3-4: Fix Error Type Conversions**
   - Implement proper `From` traits between error types
   - Fix error handling in adapters
   - Add error conversion tests

3. **Day 5-7: Fix Parser and Lifetime Issues**
   - Address borrowed data escaping issues in parser methods
   - Fix lifetime annotations
   - Implement proper cloning where needed

### Week 2: Core Adapter Tests (April 8-14)

1. **Day 1-2: Registry Adapter Tests**
   - Implement comprehensive test suite for CommandRegistryAdapter
   - Fix async test implementations
   - Verify command registration and execution works

2. **Day 3-5: MCP Adapter Tests**
   - Implement authentication tests
   - Add message handling tests
   - Test context integration

3. **Day 6-7: Plugin Adapter Tests**
   - Implement lifecycle tests
   - Add command execution tests
   - Test error handling

### Week 3: Integration and Security (April 15-21)

1. **Day 1-3: Cross-Adapter Integration Tests**
   - Test adapter interoperability
   - Verify end-to-end command execution
   - Test error propagation

2. **Day 4-5: Security Tests**
   - Implement authentication tests
   - Add authorization tests
   - Test input validation

3. **Day 6-7: Performance Tests**
   - Measure adapter performance
   - Test under load
   - Identify bottlenecks

### Week 4: Finalization (April 22-28)

1. **Day 1-3: Edge Cases and Regression Tests**
   - Test error handling edge cases
   - Add regression tests for known issues
   - Test rare scenarios

2. **Day 4-5: Documentation**
   - Document test patterns
   - Create test guidelines
   - Update API documentation

3. **Day 6-7: CI Integration**
   - Set up CI tests
   - Configure test automation
   - Verify test coverage

## Immediate Next Steps (Next 48 Hours)

1. **Create the isolated test module**:
   ```bash
   # Create the file with initial structure
   touch crates/cli/src/commands/adapter/isolated_tests.rs
   
   # Update mod.rs to include it
   # Add "pub mod isolated_tests;" to crates/cli/src/commands/adapter/mod.rs
   ```

2. **Implement the minimal mock types**:
   - Create `Command` trait
   - Implement `MockCommand` struct
   - Create `MockAdapter` trait
   - Implement `SimpleMockAdapter` struct
   - Add basic tests

3. **Fix the most critical error type issues**:
   - Add `From` implementations for `CommandError` types
   - Fix error conversions in adapter methods
   - Update error handling in tests

4. **Run isolated tests**:
   ```bash
   # Run only the isolated tests
   cargo test --package squirrel-cli --lib commands::adapter::isolated_tests
   ```

5. **Document lessons learned**:
   - Update this plan with findings
   - Identify additional issues
   - Refine the implementation strategy
   
## Progress Tracking

We will track progress in a separate document at `specs/cli/adapter-tests-progress.md`, which will be updated daily with:

1. Completed tasks
2. Encountered issues
3. Solutions implemented
4. Test results
5. Next steps

The progress document will be linked to this plan and will serve as a daily log of the implementation effort.

---

This implementation plan provides a structured approach to solving the adapter test issues, with clear steps, timelines, and tracking mechanisms. By focusing first on isolated testing and then expanding to integration testing, we'll ensure a robust test suite that validates our adapter implementations. 