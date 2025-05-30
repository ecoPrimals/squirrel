# CLI Adapter Pattern Integration

This document outlines the approach for integrating the Command Adapter Pattern into the Squirrel CLI application, focusing on how to properly connect the existing command system with the adapter interfaces.

## Overview

The CLI adapter integration will transform the existing command execution model to use the adapter pattern, allowing for more flexible command execution through multiple interfaces (direct CLI, MCP protocol, plugins, etc.) while maintaining a consistent API.

## Integration Goals

1. Decouple command execution from specific interface implementations
2. Allow commands to be executed through multiple interfaces without modification
3. Support authentication and authorization when executing commands remotely
4. Enable seamless integration between CLI, MCP, and plugin systems
5. Provide a consistent interface for command discovery and help information
6. Ensure thread safety and proper async execution

## Architecture Changes

### Current Architecture

```
┌────────────────┐     ┌────────────────┐     ┌────────────────┐
│   CLI Parser   │────▶│ Command Registry ────▶│   Commands    │
└────────────────┘     └────────────────┘     └────────────────┘
                              │
                              ▼
                       ┌────────────────┐     
                       │ Command Executor │     
                       └────────────────┘     
```

### New Architecture

```
                       ┌────────────────┐
                       │   Commands    │
                       └────────────────┘
                              ▲
                              │
┌────────────────┐     ┌────────────────┐
│  CLI Parser    │────▶│ Command Registry │
└────────────────┘     └────────────────┘
                              │
                              ▼
┌────────────────┐     ┌────────────────┐     ┌────────────────┐
│ Registry Adapter │◄───│ Adapter Factory │───▶│  MCP Adapter   │
└────────────────┘     └────────────────┘     └────────────────┘
        │                                             │
        ▼                                             ▼
┌────────────────┐                         ┌────────────────┐
│  CLI Executor  │                         │  MCP Protocol  │
└────────────────┘                         └────────────────┘
```

## Implementation Plan

### Phase 1: Refactor Command Registry

1. Modify the Command Registry to support thread-safe access
2. Update the registry to support async operations
3. Standardize the command execution interface

```rust
// Current implementation
pub struct CommandRegistry {
    commands: HashMap<String, Box<dyn Command>>,
}

// New implementation
pub struct CommandRegistry {
    commands: HashMap<String, Arc<dyn Command>>,
}

impl CommandRegistry {
    pub fn register_command(&mut self, name: &str, command: Arc<dyn Command>) -> Result<(), CommandError> { ... }
    pub fn execute_command(&self, name: &str, args: Vec<String>) -> Result<String, CommandError> { ... }
    pub fn get_command_help(&self, name: &str) -> Result<String, CommandError> { ... }
    pub fn list_commands(&self) -> Result<Vec<String>, CommandError> { ... }
}
```

### Phase 2: Implement Command Adapters

1. Create the Registry Adapter
2. Implement the MCP Adapter with authentication
3. Implement the Plugin Adapter for plugin integration

```rust
// Registry Adapter
pub struct CommandRegistryAdapter {
    registry: Arc<Mutex<CommandRegistry>>,
}

#[async_trait]
impl CommandAdapter for CommandRegistryAdapter {
    async fn execute_command(&self, name: &str, args: Vec<String>) -> Result<String, CommandAdapterError> { ... }
    async fn get_command_help(&self, name: &str) -> Result<String, CommandAdapterError> { ... }
    async fn list_commands(&self) -> Result<Vec<String>, CommandAdapterError> { ... }
}

// MCP Adapter
pub struct McpCommandAdapter {
    registry_adapter: Arc<CommandRegistryAdapter>,
    auth_manager: Arc<AuthManager>,
}

// Plugin Adapter
pub struct CommandsPluginAdapter {
    registry_adapter: Arc<CommandRegistryAdapter>,
    plugin_metadata: PluginMetadata,
}
```

### Phase 3: Modify Command Execution Flow

1. Update the command execution flow to use adapters
2. Create an adapter factory to provide appropriate adapters
3. Modify the CLI entry point to use the registry adapter

```rust
// Adapter factory
pub fn create_adapter(adapter_type: AdapterType) -> Arc<dyn CommandAdapter> {
    match adapter_type {
        AdapterType::Registry => Arc::new(CommandRegistryAdapter::new()),
        AdapterType::Mcp => Arc::new(McpCommandAdapter::new()),
        AdapterType::Plugin => Arc::new(CommandsPluginAdapter::new()),
    }
}

// CLI entry point
pub fn execute_cli_command(command: &str, args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let adapter = create_adapter(AdapterType::Registry);
    
    // Use async runtime to execute the command
    let runtime = tokio::runtime::Runtime::new()?;
    let result = runtime.block_on(adapter.execute_command(command, args))?;
    
    println!("{}", result);
    Ok(())
}
```

### Phase 4: Integrate with MCP and Plugin Systems

1. Connect the MCP server to the MCP adapter
2. Register the plugin adapter with the plugin system
3. Implement proper authentication flow for MCP commands

```rust
// MCP command handler
pub async fn handle_mcp_command(request: &McpCommandRequest) -> McpCommandResponse {
    let adapter = create_adapter(AdapterType::Mcp).downcast::<McpCommandAdapter>().unwrap();
    
    // Extract authentication info
    let auth = match &request.credentials {
        Some(creds) => Auth::User(creds.username.clone(), creds.password.clone()),
        None => Auth::None,
    };
    
    // Execute with auth
    match adapter.execute_with_auth(&request.command, request.arguments.clone(), auth).await {
        Ok(output) => McpCommandResponse {
            success: true,
            output: Some(output),
            error: None,
        },
        Err(e) => McpCommandResponse {
            success: false,
            output: None,
            error: Some(e.to_string()),
        },
    }
}

// Plugin registration
pub fn register_command_plugin(registry: Arc<Mutex<PluginRegistry>>) -> Result<(), PluginError> {
    let adapter = create_adapter(AdapterType::Plugin).downcast::<CommandsPluginAdapter>().unwrap();
    registry.lock().unwrap().register_plugin(adapter)
}
```

## Testing Strategy

1. Unit tests for individual adapters
2. Integration tests for adapter interactions
3. End-to-end tests for CLI command execution
4. Authentication/authorization tests for MCP adapter
5. Plugin system integration tests

## CLI Changes

The CLI interface itself won't change significantly from a user perspective, but the internal implementation will be updated to use the adapters:

```rust
// Command trait implementation
pub trait Command: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn execute(&self, args: Vec<String>) -> Result<String, CommandError>;
}

// Command implementation (unchanged from user perspective)
pub struct MyCommand {
    // Command-specific fields
}

impl Command for MyCommand {
    fn name(&self) -> &str {
        "my-command"
    }
    
    fn description(&self) -> &str {
        "My custom command"
    }
    
    fn execute(&self, args: Vec<String>) -> Result<String, CommandError> {
        // Command implementation
    }
}
```

## Backward Compatibility

To ensure backward compatibility during the transition:

1. Keep the existing command registry interface until all commands are migrated
2. Provide adapter wrappers for legacy command implementations
3. Update commands gradually to use the new interfaces
4. Maintain command-line argument compatibility

## Performance Considerations

1. Minimize lock contention in the registry adapter
2. Use efficient async patterns for command execution
3. Cache command metadata where appropriate
4. Profile and optimize authentication flow for MCP adapter
5. Ensure thread-safety without excessive locking

## Security Considerations

1. Properly validate and sanitize command arguments
2. Implement robust authentication for MCP commands
3. Enforce proper authorization for privileged commands
4. Validate plugin permissions before execution
5. Maintain audit logs for security-sensitive operations

## Next Steps

1. Implement the CommandRegistry refactoring
2. Create and test the basic registry adapter
3. Implement the MCP adapter with authentication
4. Integrate with the plugin system
5. Update CLI entry points to use the adapter pattern
6. Test thoroughly across all interfaces

## Related Documents

- [Command Adapter Pattern](command-adapter-pattern.md)
- [MCP Protocol Integration](../mcp/protocol-integration.md)
- [Plugin System Architecture](../plugins/plugin-architecture.md) 