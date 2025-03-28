# Squirrel MCP Examples

This directory contains examples that demonstrate various aspects of the Squirrel MCP project.

## Running the Examples

You can run any example with Cargo:

```bash
cargo run --example <example_name>
```

For specific examples with command-line arguments:

```bash
cargo run --example <example_name> -- <arguments>
```

## Available Examples

### Custom Dashboard Example

The `custom_dashboard` example demonstrates a custom TUI dashboard with simulated metrics and MCP protocol integration:

```bash
cargo run --example custom_dashboard -- --help
```

Options:
- `-i, --interval <INTERVAL>`: Update interval in seconds (default: 3)
- `-a, --alerts`: Enable dynamic alerts simulation
- `-m, --mcp`: Show MCP protocol simulation
- `-p, --pattern <PATTERN>`: Show CPU simulation pattern (sine, spike, random) (default: sine)

Example usages:

```bash
# Basic dashboard with default settings
cargo run --example custom_dashboard

# Dashboard with alerts, fast updates, and MCP integration
cargo run --example custom_dashboard -- --alerts --interval 1 --mcp

# Dashboard with random CPU pattern and alerts
cargo run --example custom_dashboard -- --alerts --pattern random
```

### Simple Journal Example

```bash
cargo run --example simple_journal
```

### Plugin Usage Example

```bash
cargo run --example plugin_usage
```

### Transaction Example

```bash
cargo run --example transaction_example
```

### Observability Example

```bash
cargo run --example observability_example
```

### Phase 1 Demo

```bash
cargo run --example phase1_demo
```

### Journal Example

```bash
cargo run --example journal_example
```

## Running With Specific Features

Some examples may require specific features to be enabled:

```bash
cargo run --example <example_name> --features <feature_name>
```

## Notes

- Make sure you have all the necessary dependencies installed.
- Some examples might require specific environment setup or configuration files.
- The Terminal UI examples work best in a terminal with color support and sufficient size. 