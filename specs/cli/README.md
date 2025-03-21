# CLI Specification

## Overview

The Command-Line Interface (CLI) for the Squirrel platform provides a user-friendly way to interact with the core system functionality through terminal commands. This document outlines the specifications, architecture, and design patterns for the CLI component.

## Current Status

The CLI implementation is currently in active development with basic functionality established:
- Basic command execution framework
- Integration with the Command Registry system
- Core command set implementation
- Lock performance optimization

## Core Components

### CLI Architecture

```
┌─────────────────────────────────────┐
│             CLI Interface           │
│                                     │
│  ┌─────────────┐   ┌─────────────┐  │
│  │ Command     │   │ Argument    │  │
│  │ Processor   │   │ Parser      │  │
│  └─────┬───────┘   └──────┬──────┘  │
│        │                  │         │
│        ▼                  ▼         │
│  ┌─────────────────────────────────┐│
│  │       Command Registry          ││
│  └─────────────────┬───────────────┘│
│                    │                │
│                    ▼                │
│  ┌─────────────────────────────────┐│
│  │        Command Executor         ││
│  └─────────────────────────────────┘│
└─────────────────────────────────────┘
```

### Primary Components

1. **CLI Interface**: Entry point for user commands
2. **Command Processor**: Interprets and validates commands
3. **Argument Parser**: Parses command-line arguments
4. **Command Registry**: Manages available commands
5. **Command Executor**: Executes commands with proper context

## Technical Requirements

### Command Structure

Commands should follow a consistent pattern:
```rust
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Suppress logging output
    #[arg(short, long)]
    quiet: bool,

    /// Set logging level (error, warn, info, debug, trace)
    #[arg(long, default_value = "info")]
    log_level: String,

    /// The command to execute
    #[command(subcommand)]
    command: Commands,
}
```

### Standard Global Options

All commands should support these standard options:
- `-v, --verbose`: Enable verbose logging
- `-q, --quiet`: Suppress logging output
- `--log-level`: Set logging level
- `--help`: Show help information
- `--version`: Show version information

### Error Handling

The CLI should use a consistent error handling approach:
```rust
fn execute(&self) -> std::result::Result<(), Box<dyn Error>> {
    // Handle errors with proper context
    if let Err(e) = do_something() {
        log_command_error(self.name(), &e);
        return Err(e.into());
    }
    Ok(())
}
```

### Performance Requirements

- Command initialization: < 100ms
- Command execution (simple): < 50ms
- Memory usage: < 50MB per CLI session
- Lock contention: Avoid holding locks during command execution

## Command Specifications

### Core Commands

1. **help**: Display help information
   - Usage: `squirrel help [command]`
   - Description: Shows help for all commands or a specific command

2. **version**: Display version information
   - Usage: `squirrel version`
   - Description: Shows version information for the CLI and core

3. **status**: Check system status
   - Usage: `squirrel status`
   - Description: Shows the current status of the Squirrel system

### Command Development Guidelines

1. Follow the `006-cli-standards` rule for consistent command implementation
2. Use clap's derive feature for argument parsing
3. Separate stdout (for data) and stderr (for logs)
4. Implement proper error handling with context
5. Add comprehensive integration tests for each command

## Integration Points

### Core Integration

The CLI integrates with the Core system through the following components:
- `squirrel-core`: For core functionality access
- `squirrel-commands`: For command registry and execution

```rust
// Create a core instance
let core = Core::new();

// Create and use command registry
let registry = squirrel_commands::create_command_registry()?;
let result = registry.lock()?.execute(&command_name, &command_args)?;
```

### MCP Integration

The CLI should support interacting with remote MCP services:
- Establish connection to MCP servers
- Execute commands remotely when applicable
- Manage authentication and session state

## Implementation Considerations

### Lock Management

The current implementation includes optimizations for lock management:
- Batch operations that require locks to minimize contention
- Use a `LockTimer` to track and log lock acquisition times
- Warn when locks are held for too long (potential contention)

```rust
struct LockTimer {
    operation: String,
    start_time: Instant,
    warn_threshold: Duration,
}
```

### Logging and Output

The CLI follows these principles for logging and output:
- Use the `log` crate for consistent logging
- Write logs to stderr, command output to stdout
- Support different verbosity levels

## Testing Strategy

1. **Unit Tests**: Test individual command implementations
2. **Integration Tests**: Test end-to-end command execution
3. **Performance Tests**: Ensure commands meet performance requirements

Example integration test:
```rust
#[test]
fn test_help_command() {
    let mut cmd = Command::cargo_bin("squirrel").unwrap();
    cmd.arg("help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Available commands"));
}
```

## Future Enhancements

1. **Shell Completion**: Add support for shell completion scripts
2. **Interactive Mode**: Implement an interactive shell mode
3. **Plugin Commands**: Support for plugin-provided commands
4. **Richer Output Formats**: Support for JSON, YAML, and other output formats
5. **Remote Command Execution**: Execute commands on remote squirrel instances

## Development Roadmap

1. **Phase 1**: Basic command infrastructure (Complete)
2. **Phase 2**: Core command implementation (In Progress)
3. **Phase 3**: Advanced features and performance optimization (Planned)
4. **Phase 4**: Plugin integration and extensibility (Future)

## Documentation Requirements

1. Each command should have comprehensive help text
2. Man pages should be generated for all commands
3. Examples should be provided for common use cases
4. Error messages should be clear and actionable 