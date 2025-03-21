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

## MCP Commands

### 1. connect

Connects to an MCP server.

**Usage:**
```
squirrel connect [--host <host>] [--port <port>] [--timeout <seconds>]
```

**Options:**
- `--host <host>`: MCP server host (default: localhost)
- `--port <port>`: MCP server port (default: 9000)
- `--timeout <seconds>`: Connection timeout in seconds (default: 30)

**Examples:**
```bash
# Connect to localhost
squirrel connect

# Connect to a remote server
squirrel connect --host example.com --port 9001
```

### 2. send

Sends a command to an MCP server.

**Usage:**
```
squirrel send <command> [args...] [--format <json|text>]
```

**Arguments:**
- `command`: Command to send
- `args`: Arguments for the command

**Options:**
- `--format <json|text>`: Output format (default: text)

**Examples:**
```bash
# Send a command to the MCP server
squirrel send status

# Send a command with JSON output
squirrel send analyze project_dir --format json
```

## Management Commands

### 1. plugin

Manages Squirrel plugins.

**Usage:**
```
squirrel plugin [list|install|remove|update|info] [plugin_name] [options]
```

**Subcommands:**
- `list`: List installed plugins
- `install <plugin_name>`: Install a plugin
- `remove <plugin_name>`: Remove a plugin
- `update [plugin_name]`: Update plugins
- `info <plugin_name>`: Show plugin information

**Examples:**
```bash
# List installed plugins
squirrel plugin list

# Install a plugin
squirrel plugin install code-analyzer

# Update all plugins
squirrel plugin update
```

### 2. log

Manages and displays logs.

**Usage:**
```
squirrel log [show|clear|export] [options]
```

**Subcommands:**
- `show`: Show logs
- `clear`: Clear logs
- `export <file>`: Export logs to file

**Options for 'show':**
- `--level <level>`: Minimum log level to display
- `--limit <count>`: Maximum number of logs to display
- `--follow`: Continuously show new logs

**Examples:**
```bash
# Show recent logs
squirrel log show --limit 50

# Show only error logs
squirrel log show --level error

# Follow logs in real-time
squirrel log show --follow
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