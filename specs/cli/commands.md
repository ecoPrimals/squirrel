---
title: CLI Commands Specification
version: 1.0.0
date: 2024-03-23
status: draft
priority: high
---

# CLI Commands Specification

## Overview

This document specifies the command structure, arguments, options, and behavior for the Squirrel Command-Line Interface (CLI). It serves as a reference for implementing and extending the CLI's command set.

## Command Structure

All Squirrel CLI commands follow a consistent structure:

```
squirrel [global options] <command> [command options] [arguments]
```

### Global Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--verbose` | `-v` | Enable verbose output | `false` |
| `--quiet` | `-q` | Suppress non-essential output | `false` |
| `--log-level` | `-l` | Set logging level (error, warn, info, debug, trace) | `info` |
| `--config` | `-c` | Path to config file | `~/.squirrel/config.toml` |
| `--help` | `-h` | Show help information | - |
| `--version` | `-V` | Show version information | - |

## Core Commands

### 1. help

Displays help information for available commands.

**Usage:**
```
squirrel help [command]
```

**Arguments:**
- `command` (optional): Command to show help for

**Examples:**
```bash
# Show general help
squirrel help

# Show help for a specific command
squirrel help status
```

**Implementation:**
```rust
#[derive(Parser, Debug)]
struct HelpCommand {
    /// Command to show help for
    command: Option<String>,
}

impl HelpCommand {
    fn execute(&self, registry: &CommandRegistry) -> Result<(), Box<dyn Error>> {
        if let Some(cmd) = &self.command {
            // Show help for specific command
            let help = registry.get_help(cmd)?;
            println!("{}", help);
        } else {
            // Show general help
            let commands = registry.list_commands()?;
            println!("Squirrel CLI - Available Commands:");
            
            for cmd in commands {
                let help = registry.get_help(&cmd)?;
                println!("  {}", help);
            }
        }
        
        Ok(())
    }
}
```

### 2. version

Displays version information for the Squirrel system components.

**Usage:**
```
squirrel version [--json] [--check <version>]
```

**Options:**
- `--json`: Output version info in JSON format
- `--check <version>`: Check if the current version meets the minimum requirement

**Examples:**
```bash
# Show version information
squirrel version

# Show version in JSON format
squirrel version --json

# Check if current version meets requirement
squirrel version --check 0.2.0
```

**Implementation:**
```rust
#[derive(Parser, Debug)]
struct VersionCommand {
    /// Output version info in JSON format
    #[arg(long)]
    json: bool,
    
    /// Check if the current version meets the minimum requirement
    #[arg(long)]
    check: Option<String>,
}

impl VersionCommand {
    fn execute(&self) -> Result<(), Box<dyn Error>> {
        let version = squirrel_core::build_info::version();
        
        if let Some(check_version) = &self.check {
            // Check version requirement
            if version < check_version {
                println!("Version requirement not met: {} < {}", version, check_version);
                return Err("Version requirement not met".into());
            } else {
                println!("Version requirement met: {} >= {}", version, check_version);
                return Ok(());
            }
        }
        
        if self.json {
            // Output in JSON format
            println!("{{");
            println!("  \"version\": \"{}\",", version);
            println!("  \"core_version\": \"{}\",", squirrel_core::build_info::version());
            println!("  \"build_date\": \"{}\"", squirrel_core::build_info::build_date());
            println!("}}");
        } else {
            // Output in plain text format
            println!("Squirrel CLI v{}", version);
            println!("Core: v{}", squirrel_core::build_info::version());
            println!("Build Date: {}", squirrel_core::build_info::build_date());
        }
        
        Ok(())
    }
}
```

### 3. status

Shows the current status of the Squirrel system.

**Usage:**
```
squirrel status [--watch] [--interval <seconds>]
```

**Options:**
- `--watch`: Continuously monitor status
- `--interval <seconds>`: Update interval in seconds when watching (default: 5)

**Examples:**
```bash
# Show current status
squirrel status

# Monitor status with 2-second updates
squirrel status --watch --interval 2
```

**Implementation:**
```rust
#[derive(Parser, Debug)]
struct StatusCommand {
    /// Continuously monitor status
    #[arg(long)]
    watch: bool,
    
    /// Update interval in seconds when watching
    #[arg(long, default_value = "5")]
    interval: u64,
}

impl StatusCommand {
    async fn execute(&self, core: &Core) -> Result<(), Box<dyn Error>> {
        if self.watch {
            // Continuously monitor status
            loop {
                self.display_status(core)?;
                tokio::time::sleep(tokio::time::Duration::from_secs(self.interval)).await;
                // Clear screen
                print!("\x1B[2J\x1B[1;1H");
            }
        } else {
            // Show current status once
            self.display_status(core)?;
        }
        
        Ok(())
    }
    
    fn display_status(&self, core: &Core) -> Result<(), Box<dyn Error>> {
        // Get system status
        let status = core.get_status()?;
        
        println!("=== Squirrel System Status ===");
        println!("Status: {}", status.status);
        println!("Uptime: {} seconds", status.uptime);
        println!("Memory Usage: {} MB", status.memory_usage);
        println!("Active Commands: {}", status.active_commands);
        println!("Connected Clients: {}", status.connected_clients);
        
        Ok(())
    }
}
```

### 4. config

Manages the Squirrel CLI configuration.

**Usage:**
```
squirrel config [get|set|list|import|export] [key] [value]
```

**Subcommands:**
- `get <key>`: Get a configuration value
- `set <key> <value>`: Set a configuration value
- `list`: List all configuration settings
- `import <file>`: Import configuration from file
- `export <file>`: Export configuration to file

**Examples:**
```bash
# List all configuration settings
squirrel config list

# Get a specific configuration value
squirrel config get log.level

# Set a configuration value
squirrel config set log.level debug

# Export configuration to a file
squirrel config export ~/squirrel-config.toml

# Import configuration from a file
squirrel config import ~/squirrel-config.toml
```

**Implementation:**
```rust
#[derive(Parser, Debug)]
enum ConfigCommand {
    /// Get a configuration value
    Get {
        /// Configuration key
        key: String,
    },
    
    /// Set a configuration value
    Set {
        /// Configuration key
        key: String,
        
        /// Configuration value
        value: String,
    },
    
    /// List all configuration settings
    List,
    
    /// Import configuration from file
    Import {
        /// Input file path
        file: String,
    },
    
    /// Export configuration to file
    Export {
        /// Output file path
        file: String,
    },
}
```

### 5. run

Executes a command or script.

**Usage:**
```
squirrel run <command_or_script> [args...]
```

**Arguments:**
- `command_or_script`: Command or script to execute
- `args`: Arguments to pass to the command or script

**Examples:**
```bash
# Run a command with arguments
squirrel run analyze project_dir --deep

# Run a script
squirrel run scripts/backup.sq
```

**Implementation:**
```rust
#[derive(Parser, Debug)]
struct RunCommand {
    /// Command or script to execute
    command_or_script: String,
    
    /// Arguments to pass to the command or script
    args: Vec<String>,
}

impl RunCommand {
    fn execute(&self, registry: &CommandRegistry) -> Result<(), Box<dyn Error>> {
        // Check if it's a file
        if self.command_or_script.ends_with(".sq") {
            // Run as script
            return self.run_script()?;
        } else {
            // Run as command
            return registry.execute(&self.command_or_script, &self.args)?;
        }
    }
    
    fn run_script(&self) -> Result<(), Box<dyn Error>> {
        // Script execution logic
        println!("Running script: {}", self.command_or_script);
        // ... implementation ...
        Ok(())
    }
}
```

### 5. secrets

Manages secret values for secure storage and retrieval.

**Usage:**
```
squirrel secrets [get|set|list|delete] [key] [value]
```

**Subcommands:**
- `get <key>`: Get a secret value
- `set <key> <value>`: Set a secret value
- `list`: List all secret keys (values are not displayed)
- `delete <key>`: Delete a secret value

**Examples:**
```bash
# Set a secret value
squirrel secrets set api_token "my-secret-token"

# Get a secret value
squirrel secrets get api_token

# List all secret keys
squirrel secrets list

# Delete a secret
squirrel secrets delete api_token
```

**Implementation:**
```rust
#[derive(Parser, Debug)]
struct SecretsCommand {
    /// Subcommand for secrets management
    #[command(subcommand)]
    subcommand: SecretsSubcommand,
}

#[derive(Subcommand, Debug)]
enum SecretsSubcommand {
    /// Get a secret value
    Get {
        /// Key to get
        key: String,
    },
    
    /// Set a secret value
    Set {
        /// Key to set
        key: String,
        
        /// Value to set
        value: String,
    },
    
    /// List all secret keys
    List,
    
    /// Delete a secret value
    Delete {
        /// Key to delete
        key: String,
    },
}

impl SecretsCommand {
    fn execute(&self, secrets_manager: &SecretsManager) -> Result<String, Box<dyn Error>> {
        match &self.subcommand {
            SecretsSubcommand::Get { key } => {
                // Get secret
                let value = secrets_manager.get(key)?;
                Ok(value)
            },
            SecretsSubcommand::Set { key, value } => {
                // Set secret
                secrets_manager.set(key, value)?;
                Ok(format!("Secret '{}' set successfully", key))
            },
            SecretsSubcommand::List => {
                // List secret keys
                let keys = secrets_manager.list_keys()?;
                Ok(format!("Secret keys: {}", keys.join(", ")))
            },
            SecretsSubcommand::Delete { key } => {
                // Delete secret
                secrets_manager.delete(key)?;
                Ok(format!("Secret '{}' deleted successfully", key))
            },
        }
    }
}
```

### 6. mcp

Manages Machine Context Protocol (MCP) operations including server control, client connections, and protocol operations.

**Usage:**
```
squirrel mcp [server|client|status|protocol] [options]
```

**Subcommands:**
- `server`: Start an MCP server
  - `--host <host>`: Host to bind to (default: 127.0.0.1)
  - `--port <port>`: Port to listen on (default: 7777)
  
- `client`: Connect to an MCP server
  - `--host <host>`: Host to connect to (default: 127.0.0.1)
  - `--port <port>`: Port to connect to (default: 7777)
  - `--timeout <seconds>`: Connection timeout (default: 30)
  - `--interactive`: Start in interactive mode

- `status`: Check MCP server status
  - `--host <host>`: Host to check (default: 127.0.0.1)
  - `--port <port>`: Port to check (default: 7777)

- `protocol`: Manage MCP protocol operations
  - `validate <file>`: Validate a protocol message
  - `generate <type>`: Generate a protocol message template
  - `convert <file> --from <version> --to <version>`: Convert between protocol versions

**Examples:**
```bash
# Start an MCP server
squirrel mcp server --port 8080

# Connect to an MCP server in interactive mode
squirrel mcp client --host localhost --port 8080 --interactive

# Check server status
squirrel mcp status

# Validate a protocol message
squirrel mcp protocol validate message.json

# Generate a protocol message template
squirrel mcp protocol generate tool_call

# Convert a protocol message between versions
squirrel mcp protocol convert message.json --from 1.0 --to 1.1
```

**Implementation:**
```rust
#[derive(Parser, Debug)]
struct MCPCommand {
    /// Subcommand for MCP operations
    #[command(subcommand)]
    subcommand: MCPSubcommand,
}

#[derive(Subcommand, Debug)]
enum MCPSubcommand {
    /// Start an MCP server
    Server {
        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        
        /// Port to listen on
        #[arg(long, default_value = "7777")]
        port: u16,
    },
    
    /// Connect to an MCP server
    Client {
        /// Host to connect to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        
        /// Port to connect to
        #[arg(long, default_value = "7777")]
        port: u16,
        
        /// Connection timeout in seconds
        #[arg(long, default_value = "30")]
        timeout: u64,
        
        /// Start in interactive mode
        #[arg(long)]
        interactive: bool,
    },
    
    /// Check MCP server status
    Status {
        /// Host to check
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        
        /// Port to check
        #[arg(long, default_value = "7777")]
        port: u16,
    },
    
    /// Manage MCP protocol operations
    Protocol {
        /// Protocol subcommand
        #[command(subcommand)]
        protocol_cmd: ProtocolSubcommand,
    },
}

#[derive(Subcommand, Debug)]
enum ProtocolSubcommand {
    /// Validate a protocol message
    Validate {
        /// File containing the message
        file: PathBuf,
    },
    
    /// Generate a protocol message template
    Generate {
        /// Type of message to generate
        message_type: String,
    },
    
    /// Convert between protocol versions
    Convert {
        /// File containing the message
        file: PathBuf,
        
        /// Original protocol version
        #[arg(long)]
        from: String,
        
        /// Target protocol version
        #[arg(long)]
        to: String,
    },
}

impl MCPCommand {
    async fn execute(&self, mcp_client: &MCPClient) -> Result<String, Box<dyn Error>> {
        match &self.subcommand {
            MCPSubcommand::Server { host, port } => {
                // Start MCP server
                let server = MCPServer::new(host, *port);
                server.start().await?;
                Ok(format!("MCP server started on {}:{}", host, port))
            },
            MCPSubcommand::Client { host, port, timeout, interactive } => {
                // Connect to MCP server
                mcp_client.connect(host, *port, Duration::from_secs(*timeout)).await?;
                
                if *interactive {
                    // Start interactive mode
                    self.interactive_mode(mcp_client).await?;
                    Ok("Interactive mode exited".to_string())
                } else {
                    Ok(format!("Connected to MCP server at {}:{}", host, port))
                }
            },
            // Other subcommands...
        }
    }
    
    async fn interactive_mode(&self, client: &MCPClient) -> Result<(), Box<dyn Error>> {
        // Implement interactive mode
        // ...
        Ok(())
    }
}
```

### 7. plugin

Manages Squirrel plugins for extending CLI functionality.

**Usage:**
```
squirrel plugin [list|install|uninstall|info|enable|disable] [name]
```

**Subcommands:**
- `list`: List all installed plugins
- `install <name>`: Install a plugin
- `uninstall <name>`: Uninstall a plugin
- `info <name>`: Show plugin information
- `enable <name>`: Enable a plugin
- `disable <name>`: Disable a plugin

**Examples:**
```bash
# List all installed plugins
squirrel plugin list

# Install a plugin
squirrel plugin install my-plugin

# Get plugin information
squirrel plugin info my-plugin

# Enable a plugin
squirrel plugin enable my-plugin

# Disable a plugin
squirrel plugin disable my-plugin

# Uninstall a plugin
squirrel plugin uninstall my-plugin
```

**Implementation:**
```rust
#[derive(Parser, Debug)]
struct PluginCommand {
    /// Subcommand for plugin management
    #[command(subcommand)]
    subcommand: PluginSubcommand,
}

#[derive(Subcommand, Debug)]
enum PluginSubcommand {
    /// List all installed plugins
    List,
    
    /// Install a plugin
    Install {
        /// Plugin name or path
        name: String,
    },
    
    /// Uninstall a plugin
    Uninstall {
        /// Plugin name
        name: String,
    },
    
    /// Show plugin information
    Info {
        /// Plugin name
        name: String,
    },
    
    /// Enable a plugin
    Enable {
        /// Plugin name
        name: String,
    },
    
    /// Disable a plugin
    Disable {
        /// Plugin name
        name: String,
    },
}

impl PluginCommand {
    fn execute(&self, plugin_manager: &PluginManager) -> Result<String, Box<dyn Error>> {
        match &self.subcommand {
            PluginSubcommand::List => {
                // List plugins
                let plugins = plugin_manager.list_plugins()?;
                let mut result = String::from("Installed plugins:\n");
                
                for plugin in plugins {
                    result.push_str(&format!("- {} v{} ({})\n", 
                        plugin.name, 
                        plugin.version,
                        if plugin.enabled { "enabled" } else { "disabled" }
                    ));
                }
                
                Ok(result)
            },
            // Other subcommands...
        }
    }
}
```

## Command Registration

All commands must be registered with the command registry:

```rust
// Create the command registry
let mut registry = CommandRegistry::new();

// Register core commands
registry.register_command("help", Box::new(HelpCommand::default()))?;
registry.register_command("version", Box::new(VersionCommand::default()))?;
registry.register_command("status", Box::new(StatusCommand::default()))?;
registry.register_command("config", Box::new(ConfigCommand::default()))?;
registry.register_command("run", Box::new(RunCommand::default()))?;

// Register MCP commands
registry.register_command("connect", Box::new(ConnectCommand::default()))?;
registry.register_command("send", Box::new(SendCommand::default()))?;

// Register management commands
registry.register_command("plugin", Box::new(PluginCommand::default()))?;
registry.register_command("log", Box::new(LogCommand::default()))?;
```

## Command Development Guidelines

1. **Follow the trait-based pattern**: Implement the `Command` trait for all commands
2. **Use structured arguments**: Use `clap`'s derive feature for argument parsing
3. **Provide detailed help**: Include comprehensive help text for each command
4. **Return structured output**: Support both human-readable and machine-readable output
5. **Handle errors gracefully**: Provide clear error messages with context
6. **Support dry-run mode**: Add `--dry-run` option for commands that modify state
7. **Add verbose output**: Use the global verbosity level to adjust output detail
8. **Implement testing**: Add comprehensive unit and integration tests

## Testing Requirements

For each command, the following tests should be implemented:

1. **Argument parsing**: Test that arguments are correctly parsed
2. **Help display**: Test that help information is correctly displayed
3. **Error handling**: Test error cases and error output
4. **Output format**: Test different output formats
5. **Exit codes**: Test that exit codes match expectations

Example test:
```rust
#[test]
fn test_version_command() {
    let cmd = Command::cargo_bin("squirrel").unwrap();
    cmd.arg("version")
       .assert()
       .success()
       .stdout(predicate::str::contains("Squirrel CLI v"));

    cmd.arg("version")
       .arg("--json")
       .assert()
       .success()
       .stdout(predicate::str::contains("\"version\":"));
}
``` 