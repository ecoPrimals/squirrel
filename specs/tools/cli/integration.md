---
title: CLI Integration Specification
version: 1.0.0
date: 2024-03-23
status: draft
priority: high
---

# CLI Integration Specification

## Overview

This document defines how the Squirrel Command-Line Interface (CLI) integrates with other components of the Squirrel platform. It provides specifications for the interfaces, communication patterns, and requirements for successful integration.

## Integration Architecture

The CLI interfaces with several other components in the Squirrel platform:

```
┌───────────────────────────────────────────────────────────────────┐
│                                                                   │
│                         Squirrel CLI                              │
│                                                                   │
└───────────┬───────────────────┬──────────────────┬───────────────┘
            │                   │                  │
            ▼                   ▼                  ▼
┌───────────────────┐  ┌────────────────┐  ┌────────────────────┐
│                   │  │                │  │                    │
│   Squirrel Core   │  │   MCP Client   │  │   Plugin System    │
│                   │  │                │  │                    │
└─────────┬─────────┘  └────────┬───────┘  └──────────┬─────────┘
          │                     │                     │
          ▼                     ▼                     ▼
┌─────────────────┐  ┌──────────────────┐  ┌──────────────────────┐
│                 │  │                  │  │                      │
│  Local Services │  │   MCP Services   │  │  Third-party Plugins │
│                 │  │                  │  │                      │
└─────────────────┘  └──────────────────┘  └──────────────────────┘
```

## Core Integration Points

### 1. Squirrel Core Integration

The CLI interfaces with the Squirrel Core component for local operations and command execution.

#### Interface Definition

```rust
pub trait CoreInterface: Send + Sync {
    /// Initialize the core system
    fn initialize(&self, config: CoreConfig) -> Result<()>;
    
    /// Execute a core command
    fn execute_command(&self, command: &str, args: &[String]) -> Result<CommandResult>;
    
    /// Get system status
    fn get_status(&self) -> Result<SystemStatus>;
    
    /// Clean up resources
    fn shutdown(&self) -> Result<()>;
}
```

#### Command Flow

1. CLI parses user input
2. CLI validates command and arguments
3. CLI calls appropriate Core method
4. Core executes the operation
5. Core returns result to CLI
6. CLI formats and displays result

#### Example Usage

```rust
fn handle_core_command(core: &dyn CoreInterface, args: &[String]) -> Result<String> {
    let command = args[0].as_str();
    let command_args = &args[1..];
    
    // Execute the command
    let result = core.execute_command(command, command_args)?;
    
    // Format and return the result
    Ok(format!("{}", result))
}
```

#### Error Handling

Errors from the Core component should be wrapped with context and propagated with appropriate CLI-specific error information:

```rust
fn execute_core_command(core: &dyn CoreInterface, cmd: &str, args: &[String]) -> Result<String, CliError> {
    core.execute_command(cmd, args)
        .map_err(|e| CliError::CoreError(format!("Core command '{}' failed: {}", cmd, e)))
}
```

### 2. MCP Client Integration

The CLI interfaces with the MCP Client component for communication with remote MCP services.

#### Interface Definition

```rust
pub trait McpClientInterface: Send + Sync {
    /// Connect to an MCP server
    async fn connect(&self, host: &str, port: u16) -> Result<()>;
    
    /// Send a command to the MCP server
    async fn send_command(&self, command: &str, args: &[String]) -> Result<McpResponse>;
    
    /// Close the connection
    async fn disconnect(&self) -> Result<()>;
    
    /// Check connection status
    async fn is_connected(&self) -> Result<bool>;
}
```

#### Command Flow

1. CLI parses user input
2. CLI validates MCP command and arguments
3. CLI checks connection status, connects if needed
4. CLI sends command to MCP server
5. CLI waits for and processes response
6. CLI formats and displays result

#### Example Usage

```rust
async fn handle_mcp_command(
    mcp_client: &dyn McpClientInterface, 
    host: &str, 
    port: u16,
    command: &str, 
    args: &[String]
) -> Result<String> {
    // Connect if not already connected
    if !mcp_client.is_connected().await? {
        mcp_client.connect(host, port).await?;
    }
    
    // Send command
    let response = mcp_client.send_command(command, args).await?;
    
    // Format and return result
    Ok(format!("{}", response))
}
```

#### Authentication

MCP commands that require authentication should use the following pattern:

```rust
async fn authenticated_mcp_command(
    mcp_client: &dyn McpClientInterface,
    auth_provider: &dyn AuthProvider,
    command: &str,
    args: &[String]
) -> Result<String> {
    // Get authentication token
    let token = auth_provider.get_token()?;
    
    // Create authenticated command
    let auth_command = format!("auth {} {}", token, command);
    
    // Execute command
    mcp_client.send_command(&auth_command, args).await
}
```

### 3. Plugin System Integration

The CLI interfaces with the Plugin System to extend functionality through plugins.

#### Interface Definition

```rust
pub trait PluginManagerInterface: Send + Sync {
    /// Load all available plugins
    fn load_plugins(&self) -> Result<Vec<PluginInfo>>;
    
    /// Get a plugin by name
    fn get_plugin(&self, name: &str) -> Result<Box<dyn Plugin>>;
    
    /// List all available plugins
    fn list_plugins(&self) -> Result<Vec<PluginInfo>>;
    
    /// Register plugin commands with a command registry
    fn register_plugin_commands(&self, registry: &CommandRegistry) -> Result<()>;
}
```

#### Plugin Registration Flow

```
┌────────────┐      ┌─────────────────┐      ┌───────────┐
│            │      │                 │      │           │
│    CLI     │      │ Plugin Manager  │      │  Plugins  │
│            │      │                 │      │           │
└─────┬──────┘      └────────┬────────┘      └─────┬─────┘
      │                      │                     │
      │ Initialize           │                     │
      │─────────────────────>│                     │
      │                      │                     │
      │                      │ Discover Plugins    │
      │                      │────────────────────>│
      │                      │                     │
      │                      │<────────────────────│
      │                      │                     │
      │                      │ Load Plugins        │
      │                      │────────────────────>│
      │                      │                     │
      │                      │<────────────────────│
      │                      │                     │
      │ Register Commands    │                     │
      │─────────────────────>│                     │
      │                      │                     │
      │                      │ Register Commands   │
      │                      │────────────────────>│
      │                      │                     │
      │                      │<────────────────────│
      │                      │                     │
      │<─────────────────────│                     │
      │                      │                     │
```

#### Example Usage

```rust
fn initialize_plugins(registry: &CommandRegistry) -> Result<()> {
    // Create plugin manager
    let plugin_manager = PluginManager::new();
    
    // Load plugins
    plugin_manager.load_plugins()?;
    
    // Register plugin commands
    plugin_manager.register_plugin_commands(registry)?;
    
    Ok(())
}
```

## Integration Requirements

### Performance Requirements

| Integration Point | Requirement | Target Value |
|-------------------|-------------|--------------|
| Core Command Execution | Maximum latency | < 100ms |
| MCP Command Execution | Maximum latency | < 500ms |
| Plugin Loading | Maximum time | < 1000ms |
| Command Registration | Maximum time | < 200ms |

### Reliability Requirements

| Integration Point | Requirement | Target Value |
|-------------------|-------------|--------------|
| Core Command Execution | Error rate | < 0.1% |
| MCP Connection | Reconnection capability | Auto-reconnect after < 5s |
| Plugin System | Isolation | Plugin failures must not crash CLI |

### Security Requirements

| Integration Point | Requirement | Description |
|-------------------|-------------|-------------|
| Core | Safe command execution | Validate all command inputs |
| MCP | Secure communication | TLS for all connections |
| MCP | Authentication | Token-based authentication |
| Plugins | Sandboxing | Limit plugin capabilities |

## Integration Tests

The following integration tests should be implemented to verify proper integration:

### Core Integration Tests

```rust
#[test]
fn test_core_command_execution() {
    // Create mock core
    let core = MockCore::new();
    core.expect_execute_command()
        .with(eq("status"), eq(&[] as &[String]))
        .returning(|_, _| Ok(CommandResult::new("OK")));
    
    // Create CLI with mock core
    let cli = Cli::new(core);
    
    // Execute command
    let result = cli.execute(&["squirrel", "status"]);
    
    // Verify result
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "OK");
}
```

### MCP Integration Tests

```rust
#[tokio::test]
async fn test_mcp_command_execution() {
    // Create mock MCP client
    let mcp_client = MockMcpClient::new();
    mcp_client.expect_connect()
        .returning(|_, _| Ok(()));
    mcp_client.expect_send_command()
        .with(eq("ping"), eq(&[] as &[String]))
        .returning(|_, _| Ok(McpResponse::new("pong")));
    
    // Create CLI with mock MCP client
    let cli = Cli::new_with_mcp(mcp_client);
    
    // Execute command
    let result = cli.execute(&["squirrel", "mcp", "ping"]).await;
    
    // Verify result
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "pong");
}
```

### Plugin Integration Tests

```rust
#[test]
fn test_plugin_command_registration() {
    // Create mock plugin manager
    let plugin_manager = MockPluginManager::new();
    plugin_manager.expect_load_plugins()
        .returning(|| Ok(vec![PluginInfo::new("test-plugin", "1.0.0")]));
    plugin_manager.expect_register_plugin_commands()
        .returning(|_| Ok(()));
    
    // Create CLI with mock plugin manager
    let cli = Cli::new_with_plugins(plugin_manager);
    
    // Initialize CLI
    let result = cli.initialize();
    
    // Verify result
    assert!(result.is_ok());
    
    // Verify command registration
    let commands = cli.list_commands();
    assert!(commands.contains(&"test-plugin".to_string()));
}
```

## Implementation Guidelines

### Command Registration

Commands should be registered with the Command Registry using a consistent pattern:

```rust
fn register_core_commands(registry: &CommandRegistry) -> Result<()> {
    // Register built-in commands
    registry.register_command("help", Box::new(HelpCommand::new(registry)))?;
    registry.register_command("version", Box::new(VersionCommand::new()))?;
    registry.register_command("status", Box::new(StatusCommand::new()))?;
    
    Ok(())
}

fn register_mcp_commands(registry: &CommandRegistry, mcp_client: Arc<dyn McpClientInterface>) -> Result<()> {
    // Register MCP-related commands
    registry.register_command("connect", Box::new(ConnectCommand::new(mcp_client.clone())))?;
    registry.register_command("send", Box::new(SendCommand::new(mcp_client.clone())))?;
    registry.register_command("disconnect", Box::new(DisconnectCommand::new(mcp_client)))?;
    
    Ok(())
}
```

### Error Handling

Errors should be propagated with appropriate context to help diagnose integration issues:

```rust
fn execute_with_error_handling(registry: &CommandRegistry, args: &[String]) -> Result<String> {
    match registry.execute(&args[0], &args[1..]) {
        Ok(result) => Ok(result),
        Err(e) => {
            match e.downcast_ref::<CoreError>() {
                Some(core_err) => Err(CliError::CoreError(format!("Core error: {}", core_err))),
                None => match e.downcast_ref::<McpError>() {
                    Some(mcp_err) => Err(CliError::McpError(format!("MCP error: {}", mcp_err))),
                    None => Err(CliError::UnknownError(format!("Unknown error: {}", e))),
                },
            }
        }
    }
}
```

### Configuration Integration

The CLI should integrate with the configuration system as follows:

```rust
fn load_integrated_config() -> Result<Config> {
    // Load core configuration
    let core_config = Config::load_from_file("squirrel.toml")?;
    
    // Load CLI-specific configuration
    let cli_config = Config::load_from_file("squirrel-cli.toml").unwrap_or_default();
    
    // Merge configurations with CLI-specific overriding core
    let mut config = core_config;
    config.merge(cli_config);
    
    // Apply environment variable overrides
    config.merge(Config::from_env("SQUIRREL_")?);
    
    Ok(config)
}
```

## Verification Process

The integration verification process involves the following steps:

1. **Unit Testing**: Test each component in isolation
2. **Interface Testing**: Test the interfaces between components
3. **Integration Testing**: Test end-to-end workflows
4. **Load Testing**: Test under load conditions
5. **Failure Testing**: Test failure scenarios and recovery

### Integration Verification Matrix

| Component | Core | MCP | Plugins | Configuration | Authentication |
|-----------|------|-----|---------|---------------|----------------|
| Core Commands | ✓ | N/A | N/A | ✓ | ✓ |
| MCP Commands | ✓ | ✓ | N/A | ✓ | ✓ |
| Plugin Commands | ✓ | ✓ | ✓ | ✓ | ✓ |

## Deployment Integration

### Packaging

The CLI should be packaged in a way that ensures proper integration with all components:

```toml
# Cargo.toml
[package]
name = "squirrel-cli"
version = "1.0.0"
edition = "2021"

[dependencies]
squirrel-core = { version = "1.0.0", features = ["cli"] }
squirrel-mcp = { version = "1.0.0", features = ["client"] }
squirrel-plugins = { version = "1.0.0" }
```

### Configuration Files

The CLI should look for configuration files in the following order:

1. Path specified by `--config` command-line option
2. `.squirrel.toml` in the current directory
3. `squirrel.toml` in the user's configuration directory (platform-specific)
4. `/etc/squirrel/squirrel.toml` (on Unix-like systems)

## Migration Guidelines

When migrating from older versions, the following guidelines should be followed:

### Version 0.x to 1.0

* Commands that previously used positional arguments now require flags
* Authentication is now token-based instead of username/password
* Plugin API has changed, plugins need to be updated

Example migration script:

```rust
fn migrate_config_from_v0_to_v1(old_config: &str) -> Result<String> {
    // Parse old config
    let v0_config: ConfigV0 = toml::from_str(old_config)?;
    
    // Create new config
    let v1_config = ConfigV1 {
        core: CoreConfig {
            log_level: v0_config.log_level,
            data_dir: v0_config.data_dir,
        },
        mcp: McpConfig {
            host: v0_config.mcp_host,
            port: v0_config.mcp_port,
            use_tls: true, // New in v1
        },
        plugins: PluginConfig {
            enabled: v0_config.enable_plugins,
            directory: v0_config.plugin_dir,
            allow_unsigned: false, // New in v1
        },
    };
    
    // Serialize new config
    Ok(toml::to_string(&v1_config)?)
}
```

## Troubleshooting

### Common Integration Issues

| Issue | Possible Causes | Resolution |
|-------|----------------|------------|
| Command not found | Plugin not loaded | Check plugin loading |
| Command not found | Command not registered | Check command registration |
| Connection failed | MCP server not running | Start MCP server |
| Connection failed | Network issues | Check network connectivity |
| Authentication failed | Invalid token | Renew authentication token |
| Plugin error | Incompatible plugin | Update plugin to latest version |

### Diagnostic Commands

The CLI should provide diagnostic commands to help troubleshoot integration issues:

```
squirrel diag commands   # List all registered commands
squirrel diag plugins    # List all loaded plugins
squirrel diag mcp        # Check MCP connection
squirrel diag config     # Show effective configuration
```

## Future Enhancements

### Planned Integration Improvements

1. **Remote Plugin Repository**: Allow installing plugins from remote repositories
2. **Component Health Monitoring**: Monitor and report on component health
3. **Cross-Component Tracing**: Implement distributed tracing across components
4. **Federation**: Support for federated MCP services
5. **Multi-Transport Support**: Add support for alternative transport protocols 