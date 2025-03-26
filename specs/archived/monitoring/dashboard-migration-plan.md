# Dashboard UI Migration Plan

**Version**: 1.0.0  
**Last Updated**: 2024-06-22  
**Status**: Proposed  
**Priority**: High

## Overview

This document outlines the plan for migrating the existing Terminal UI implementation from `crates/squirrel-dashboard-tui` to `crates/ui-terminal` to align with project-wide naming conventions. This migration is necessary to ensure consistency across all UI implementations (Terminal, Web, Desktop).

## Current State

Currently, we have:
- `crates/squirrel-dashboard-core`: Core dashboard functionality
- `crates/squirrel-dashboard-tui`: Terminal UI implementation
- `crates/ui-web`: Web UI implementation

The goal is to standardize the naming pattern to:
- `crates/squirrel-dashboard-core`: Core dashboard functionality (unchanged)
- `crates/ui-terminal`: Terminal UI implementation (migrated from squirrel-dashboard-tui)
- `crates/ui-web`: Web UI implementation (unchanged)
- `crates/ui-desktop`: Desktop UI implementation (future)

## Migration Steps

### 1. Create New Directory Structure

```bash
# Create the new directory
mkdir -p crates/ui-terminal/src
mkdir -p crates/ui-terminal/src/widgets
mkdir -p crates/ui-terminal/src/bin
```

### 2. Create Cargo.toml for ui-terminal

Create a new `crates/ui-terminal/Cargo.toml` with the following content:

```toml
[package]
name = "ui-terminal"
version = "0.1.0"
edition = "2021"
description = "Terminal UI implementation for the Squirrel monitoring dashboard"

[dependencies]
# Dashboard core
squirrel-dashboard-core = { path = "../squirrel-dashboard-core" }

# TUI dependencies
ratatui = "0.24.0"
crossterm = "0.27.0"

# Async runtime
tokio = { version = "1.0", features = ["full"] }
async-trait = "0.1"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Time handling
chrono = { version = "0.4", features = ["serde"] }

# Command-line arguments
clap = { version = "4.5", features = ["derive"] }

# Utilities
tui-input = "0.8.0"

[dev-dependencies]
tokio-test = "0.4"
```

### 3. Migrate Source Files

Copy and adjust all source files from `crates/squirrel-dashboard-tui` to `crates/ui-terminal`:

```bash
# Copy source files
cp -r crates/squirrel-dashboard-tui/src/* crates/ui-terminal/src/
```

### 4. Update Import Paths

Update import paths in all files to reflect the new crate name:

1. In all files, update:
   - `use squirrel_dashboard_tui::` to `use ui_terminal::`
   - Any other references to the old crate name

### 5. Update Workspace Cargo.toml

Update the root `Cargo.toml` to include the new crate and eventually remove the old one:

```toml
[workspace]
members = [
    # ...existing members
    "crates/squirrel-dashboard-core",
    "crates/ui-terminal",
    # Keep temporarily for backward compatibility
    "crates/squirrel-dashboard-tui",
    # ...
]
```

### 6. Create README.md

Create a new README.md for `crates/ui-terminal`:

```markdown
# Terminal UI for Squirrel Dashboard

A terminal user interface implementation for the Squirrel Dashboard system.

## Features

- Interactive terminal-based dashboard for system monitoring
- Real-time metrics display with visual indicators
- Alert management and notification system
- System health check monitoring
- Network connection monitoring
- Tab-based navigation for easy access to different dashboard views

## Usage

### Running the Terminal UI Dashboard

```bash
cargo run --bin ui-terminal -- [OPTIONS]
```

### Command Line Options

```
OPTIONS:
    -c, --config <FILE>        Path to configuration file
    -r, --refresh <SECONDS>    Set refresh interval in seconds (overrides config)
    -d, --debug                Enable debug mode
    -h, --help                 Print help information
    -V, --version              Print version information
```

## Integration with Dashboard Core

This terminal UI implementation connects to the `squirrel-dashboard-core` service to retrieve metrics and other data. It subscribes to updates from the core service and automatically refreshes the display when new data is available.
```

### 7. Testing

Test the new crate:

```bash
# Build the new crate
cargo build -p ui-terminal

# Run tests
cargo test -p ui-terminal

# Run the binary
cargo run -p ui-terminal
```

### 8. Update Dependencies

If any other crates depend on `squirrel-dashboard-tui`, update them to use `ui-terminal` instead.

### 9. Deprecation Strategy

1. Add deprecation notice to `crates/squirrel-dashboard-tui/README.md`:

```markdown
# DEPRECATED: Squirrel Dashboard TUI

**This crate is deprecated and has been replaced by `ui-terminal`. Please update your dependencies.**

The crate has been moved to follow project-wide naming conventions. All functionality is now available in the `ui-terminal` crate.

## Migration

To migrate from `squirrel-dashboard-tui` to `ui-terminal`:

1. Update your Cargo.toml dependencies:
   ```toml
   # Old
   squirrel-dashboard-tui = { path = "../squirrel-dashboard-tui" }
   
   # New
   ui-terminal = { path = "../ui-terminal" }
   ```

2. Update your imports:
   ```rust
   // Old
   use squirrel_dashboard_tui::TuiDashboard;
   
   // New
   use ui_terminal::TuiDashboard;
   ```
```

2. After a reasonable transition period, remove `crates/squirrel-dashboard-tui` from the workspace and codebase.

## Timeline

- Directory creation and Cargo.toml setup: 1 hour
- File migration and path updates: 2-3 hours
- Testing and verification: 2 hours
- Dependency updates: 1-2 hours
- Total: 1 day

## Risks and Mitigation

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Breaking changes to dependent code | High | Medium | Provide clear migration instructions and temporarily keep old crate |
| Import path issues | Medium | High | Thorough testing and grep for all instances of the old crate name |
| Build failures | Medium | Medium | Incremental approach with testing at each step |
| Missing files or functionality | High | Low | Verify feature parity and run all tests |

## Success Criteria

- `crates/ui-terminal` builds successfully
- All tests pass
- Terminal UI functionality works exactly as before
- README.md provides clear documentation
- Import paths are consistent with project conventions

## Next Steps

1. Create `crates/ui-terminal` directory structure
2. Set up Cargo.toml for the new crate
3. Migrate source files
4. Update import paths
5. Update workspace configuration
6. Test functionality
7. Add deprecation notice to old crate
8. After transition period, remove old crate 