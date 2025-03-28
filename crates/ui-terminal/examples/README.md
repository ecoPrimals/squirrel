# Terminal UI Examples

This directory contains examples that demonstrate the Terminal UI capabilities of the Squirrel MCP project.

## Running Examples

You can run any example directly from the crate directory:

```bash
# From the project root
cargo run --package ui-terminal --example <example_name>

# Or from the ui-terminal crate directory
cd crates/ui-terminal
cargo run --example <example_name>
```

For specific examples with command-line arguments:

```bash
cargo run --package ui-terminal --example <example_name> -- <arguments>
```

## Available Examples

### Custom Dashboard Example

The `custom_dashboard` example demonstrates a custom TUI dashboard with simulated metrics and MCP protocol integration:

```bash
cargo run --package ui-terminal --example custom_dashboard -- --help
```

Options:
- `-i, --interval <INTERVAL>`: Update interval in seconds (default: 3)
- `-a, --alerts`: Enable dynamic alerts simulation
- `-m, --mcp`: Show MCP protocol simulation
- `-p, --pattern <PATTERN>`: Show CPU simulation pattern (sine, spike, random) (default: sine)

Example usages:

```bash
# Basic dashboard with default settings
cargo run --package ui-terminal --example custom_dashboard

# Dashboard with alerts, fast updates, and MCP integration
cargo run --package ui-terminal --example custom_dashboard -- --alerts --interval 1 --mcp

# Dashboard with random CPU pattern and alerts
cargo run --package ui-terminal --example custom_dashboard -- --alerts --pattern random
```

## Notes

- The Terminal UI examples work best in a terminal with color support and sufficient size
- For the best experience, use a terminal with TrueColor support and a size of at least 120x30
- Press '?' in any example to show the help overlay with keyboard shortcuts
- Press 'q' to quit the application 