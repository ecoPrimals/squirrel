---
description: Comprehensive documentation of Squirrel codebase structure
version: 1.0.0
last_updated: 2024-09-25
---

# Squirrel Codebase Structure

## Overview

This document provides a comprehensive overview of the Squirrel project's codebase structure. It explains the organization of directories, the relationships between components, and the flow of dependencies throughout the system.

## Directory Organization

The Squirrel project follows a logical directory structure organized by purpose:

```
squirrel/
├── code/               # All source code
│   ├── crates/         # Rust crates organized by purpose
│   │   ├── core/       # Core functionality
│   │   │   ├── context/    # Context management
│   │   │   ├── mcp/        # Machine Context Protocol
│   │   │   ├── plugins/    # Plugin system
│   │   │   ├── interfaces/ # Shared interfaces
│   │   │   └── core/       # Core utilities and shared code
│   │   ├── integration/    # Integration components
│   │   │   ├── api-clients/    # API client implementations
│   │   │   ├── context-adapter/ # Context adapter systems
│   │   │   ├── web/            # Web integration
│   │   │   └── mcp-pyo3-bindings/ # Python bindings
│   │   ├── services/      # Service implementations
│   │   │   ├── app/           # Application services
│   │   │   ├── commands/      # Command system
│   │   │   ├── dashboard-core/ # Dashboard core services
│   │   │   └── monitoring/    # Monitoring services
│   │   ├── tools/         # Development tools
│   │   │   ├── ai-tools/      # AI integration tools
│   │   │   ├── cli/           # Command-line tools
│   │   │   └── rule-system/   # Rule system tools
│   │   └── ui/            # User interface components
│   │       ├── ui-tauri-react/ # Tauri React UI
│   │       └── ui-terminal/    # Terminal UI
│   ├── src/            # Frontend source code
│   ├── src-tauri/      # Tauri integration
│   ├── plugins/        # Plugin implementations
│   ├── examples/       # Example code
│   ├── proto/          # Protocol definitions
│   ├── specs/          # Specifications
│   ├── data/           # Data files
│   ├── benches/        # Benchmarks
│   └── standalone_server/ # Standalone server implementation
│
├── docs/               # All documentation
│   ├── migration/      # Migration-related docs
│   ├── testing/        # Testing documentation
│   ├── implementation/ # Implementation plans & guides
│   ├── web/            # Web integration docs
│   ├── ui/             # UI-related docs
│   ├── miscellaneous/  # Miscellaneous documents
│   └── original/       # Original docs (reference)
│
├── scripts/            # All scripts
│   ├── build/          # Build and setup scripts
│   ├── test/           # Testing scripts
│   ├── ui/             # UI-related scripts
│   ├── windows/        # Windows-specific scripts
│   └── misc/           # Miscellaneous scripts
│
├── tests/              # Test files
│   ├── py/             # Python tests
│   ├── rs/             # Rust tests
│   └── bin/            # Test binaries
│
├── config/             # Configuration files
│   ├── docker/         # Docker configuration
│   └── misc/           # Miscellaneous configuration
│
├── logs/               # Log files
├── .meta/              # Metadata files for development tools
├── backups/            # Backup files (from migrations)
├── tools/              # Development tools
└── specs/              # Specifications and design documents
    ├── core/           # Core specifications
    ├── integration/    # Integration specifications
    ├── services/       # Service specifications
    ├── tools/          # Tool specifications
    ├── ui/             # UI specifications
    └── archived/       # Archived specifications
```

## Component Architecture

The codebase is organized around the following key components:

### Core Components

The `core` directory contains the foundational components of the system:

- **Context Management**: Manages state and application context
- **MCP (Machine Context Protocol)**: Defines the protocol for machine interactions
- **Plugins**: Plugin system architecture and management
- **Interfaces**: Shared interfaces and contracts
- **Core Utilities**: Fundamental utilities used across the system

### Integration Components

The `integration` directory contains components that bridge different parts of the system:

- **API Clients**: Implementations for various external APIs
- **Context Adapter**: Adapters for different context management systems
- **Web Integration**: Web-specific integration code
- **MCP PyO3 Bindings**: Python bindings for the MCP protocol

### Service Components

The `services` directory contains service implementations:

- **App**: Core application services
- **Commands**: Command system implementation
- **Dashboard Core**: Core functionality for dashboards
- **Monitoring**: Monitoring and observability services

### UI Components

The `ui` directory contains user interface implementations:

- **UI Tauri React**: React-based Tauri UI implementation
- **UI Terminal**: Terminal-based UI implementation

### Tool Components

The `tools` directory contains various development tools:

- **AI Tools**: AI integration and tooling
- **CLI**: Command-line interface tools
- **Rule System**: Tools for the rule system

## Dependency Flow

The flow of dependencies in the Squirrel project follows these principles:

1. **Core Independence**: Core components don't depend on service, integration, or UI components
2. **Service Dependencies**: Services depend on core components but not on each other
3. **Integration Dependencies**: Integration components may depend on core and service components
4. **UI Dependencies**: UI components may depend on core, service, and integration components
5. **Tool Independence**: Tools should be as independent as possible

This diagram illustrates the high-level dependency flow:

```
┌────────────┐      ┌────────────┐      ┌────────────┐      ┌────────────┐
│    Core    │◄─────│  Services  │◄─────│Integration │◄─────│     UI     │
└────────────┘      └────────────┘      └────────────┘      └────────────┘
                         ▲                    ▲
                         │                    │
                         └────────────────────┘
                                  ▲
                                  │
                         ┌────────────┐
                         │   Tools    │
                         └────────────┘
```

### Specific Dependencies

Here are the specific dependency relationships between major components:

| Component | Dependencies |
|-----------|--------------|
| Core/Context | Core/Interfaces |
| Core/MCP | Core/Interfaces, Core/Context |
| Core/Plugins | Core/Interfaces, Core/Context, Core/MCP |
| Services/App | Core/Context, Core/MCP, Core/Plugins |
| Services/Commands | Core/Context, Core/MCP |
| Services/Monitoring | Core/Context, Core/MCP |
| Integration/Web | Services/App, Core/MCP |
| Integration/PyO3 | Core/MCP |
| UI/TauriReact | Services/App, Integration/Web |
| UI/Terminal | Services/Commands, Services/Monitoring |

## Build System

The project uses Cargo workspaces to manage the Rust components:

```rust
// Workspace Cargo.toml
[workspace]
members = [
    "crates/core/context",
    "crates/core/mcp",
    "crates/core/plugins",
    "crates/core/interfaces",
    "crates/core/core",
    "crates/integration/api-clients",
    "crates/integration/context-adapter",
    "crates/integration/web",
    "crates/integration/mcp-pyo3-bindings",
    "crates/services/app",
    "crates/services/commands",
    "crates/services/dashboard-core",
    "crates/services/monitoring",
    "crates/tools/ai-tools",
    "crates/tools/cli",
    "crates/tools/rule-system",
    "crates/ui/ui-tauri-react",
    "crates/ui/ui-terminal",
]

[workspace.dependencies]
# Shared dependencies with locked versions
tokio = { version = "1.36", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

Each crate has its own Cargo.toml file that specifies its dependencies, including cross-crate dependencies.

## Compilation Order

To build the entire project, components must be compiled in the following order:

1. Core/Interfaces
2. Core/Context, Core/Core
3. Core/MCP, Core/Plugins
4. Services/*
5. Integration/*
6. UI/*, Tools/*

## Standard Development Environment

To ensure consistency across development environments, the project includes:

- A Docker development environment
- VS Code configuration for consistent settings
- Rustfmt and Clippy configurations
- Standard test frameworks and configurations

## Testing Strategy

The codebase follows a multi-layered testing approach:

1. **Unit Tests**: Co-located with the code they test
2. **Integration Tests**: Located in `tests/rs/` and `tests/py/`
3. **End-to-End Tests**: Located in the UI crates
4. **Performance Tests**: Located in `benches/`

## Documentation Organization

Documentation follows a similar structure to the code:

- Specifications in `specs/` mirror the crate organization in `code/crates/`
- Implementation guides in `docs/implementation/`
- API documentation generated from code comments

## Version Control Strategy

The project uses a feature-branch workflow with pull requests:

- `main` branch is the source of truth
- Feature branches are created for new features
- Pull requests are required for all changes
- Continuous integration runs tests on all pull requests

## Best Practices

To maintain the codebase structure:

1. **Follow the Directory Organization**: Place new code in the appropriate directory
2. **Respect Dependency Flow**: Don't create circular dependencies
3. **Maintain Documentation**: Update specs when code changes
4. **Write Tests**: Include tests for all new code
5. **Use Standard Patterns**: Follow established patterns for error handling, logging, etc.

## Next Steps and Evolution

The codebase structure is designed to be extensible:

1. New core capabilities should be added as new crates under `core/`
2. New integrations should be added as new crates under `integration/`
3. New services should be added as new crates under `services/`
4. New UI implementations should be added as new crates under `ui/`

This document should be updated whenever significant structural changes are made to the codebase.

---

*This document is maintained by the Core Team. Last revision: September 25, 2024.* 