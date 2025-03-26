# Squirrel CLI

A command-line interface for the Squirrel platform with a powerful plugin system.

## Features

- Extensible plugin architecture
- Command registry for uniform command handling
- Support for multiple output formats (text, JSON, YAML)
- Dynamic loading of plugin libraries
- Plugin lifecycle management
- Machine Context Protocol (MCP) integration

## Project Structure

The Squirrel project is organized as a Rust workspace with the following structure:

```
squirrel/
├── Cargo.toml           # Workspace manifest
├── src/                 # Main crate - integration point and public API
│   ├── adapter/         # Adapter interfaces for MCP integration
│   ├── security/        # Security-related functionality
│   ├── tests/           # Integration tests
│   └── lib.rs           # Main entry point and re-exports
│
├── crates/              # Individual implementation crates
│   ├── app/             # Application core implementation
│   ├── cli/             # Command Line Interface implementation
│   ├── commands/        # Command System implementation
│   ├── common/          # Common utilities and shared code
│   ├── context/         # Context Management implementation
│   ├── context-adapter/ # Context adapter implementation
│   ├── core/            # Core utilities
│   ├── galaxy/          # Galaxy integration
│   ├── interfaces/      # Shared interfaces
│   ├── mcp/             # MCP Protocol implementation
│   ├── monitoring/      # Monitoring System implementation
│   ├── plugins/         # Plugin System implementation
│   ├── test-utils/      # Testing utilities
│   └── web/             # Web Interface implementation
│
├── specs/               # Detailed specifications
│   ├── commands/        # Command System specs
│   ├── context/         # Context Management specs
│   ├── mcp/             # MCP protocol specs
│   ├── monitoring/      # Monitoring system specs
│   └── ...              # Other specifications
│
└── plugins/             # Plugin implementations
```

### Main Crate (`src/`)

The main crate in the `src/` directory serves as the primary integration point for the Squirrel ecosystem. It provides:

1. **Public API Surface**: Curated re-exports from other crates to provide a clean, unified API
2. **Integration Interfaces**: Adapters and interfaces for cross-crate communication
3. **Entry Point**: Central access point for applications using the Squirrel ecosystem

For more information, see the [src/README.md](src/README.md).

### Implementation Crates (`crates/`)

Each crate in the `crates/` directory contains a focused implementation of a specific subsystem with minimal dependencies on other crates. This separation allows for loose coupling between components and better maintainability.

## Built-in Commands

- `help`: Display help information
- `version`: Display version information
- `status`: Show system status
- `config`: Manage configuration settings
- `plugin`: Manage plugins
- `secrets`: Manage secrets
- `mcp`: Machine Context Protocol operations

### MCP Command

The MCP command provides functionality for working with the Machine Context Protocol, which enables communication between various components and services. 

Subcommands:

- `mcp server`: Start an MCP server
- `mcp client`: Connect to an MCP server
- `mcp status`: Check MCP server status
- `mcp protocol`: Manage MCP protocol operations
  - `validate`: Validate an MCP message
  - `generate`: Generate an MCP message template
  - `convert`: Convert between protocol versions

## Plugin Management

The CLI includes a robust plugin system that allows for extending functionality through plugins. Plugins can add new commands, modify existing functionality, or provide additional services.

### Plugin Commands

- `plugin list`: List installed plugins
- `plugin info <n>`: Show information about a specific plugin
- `plugin enable <n>`: Enable a plugin
- `plugin disable <n>`: Disable a plugin
- `plugin install <path>`: Install a plugin from a path
- `plugin uninstall <n>`: Uninstall a plugin
- `plugin reload`: Reload all plugins

### Creating Plugins

Plugins can be created as Rust libraries that implement the `Plugin` trait. Here's a simple example:

```rust
use async_trait::async_trait;
use squirrel_cli::plugins::{Plugin, PluginError};
use squirrel_commands::CommandRegistry;

pub struct MyPlugin;

#[async_trait]
impl Plugin for MyPlugin {
    fn name(&self) -> &str {
        "my-plugin"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn description(&self) -> Option<&str> {
        Some("My awesome plugin")
    }
    
    async fn initialize(&self) -> Result<(), PluginError> {
        // Initialize plugin
        Ok(())
    }
    
    fn register_commands(&self, registry: &mut CommandRegistry) -> Result<(), PluginError> {
        // Register plugin commands
        Ok(())
    }
    
    async fn execute(&self, args: &[String]) -> Result<String, PluginError> {
        // Execute plugin functionality
        Ok("Plugin executed".to_string())
    }
    
    async fn cleanup(&self) -> Result<(), PluginError> {
        // Clean up resources
        Ok(())
    }
}

#[no_mangle]
pub fn create_plugin() -> Result<std::sync::Arc<dyn Plugin>, PluginError> {
    Ok(std::sync::Arc::new(MyPlugin))
}
```

### Plugin Directory Structure

Plugins should follow this directory structure:

```
my-plugin/
├── Cargo.toml         # Rust package manifest
├── plugin.toml        # Plugin metadata
└── src/
    └── lib.rs         # Plugin implementation
```

The `plugin.toml` file should contain:

```toml
name = "my-plugin"
version = "1.0.0"
description = "My awesome plugin"
author = "Your Name"
homepage = "https://example.com"
```

## Building and Running

To build the CLI, run:

```
cargo build
```

To run the CLI:

```
cargo run -- [command] [options]
```

## Development

To create a new built-in command, create a new file in `crates/cli/src/commands/` and register it in `crates/cli/src/commands/mod.rs`.

To create a new plugin, use the structure outlined above and place it in the plugins directory.

### Running the Web Server

To build the UI assets and run the web server in one step:

**Windows:**
```
.\build-and-run.ps1
```

**Unix/Linux/macOS:**
```
./build-and-run.sh
```

This will:
1. Stop any existing web server processes
2. Build the UI assets from `crates/ui-web/web` into `crates/ui-web/dist`
3. Start the web server from the `crates/web` directory

The server will be available at http://localhost:3000.

## License

MIT License 