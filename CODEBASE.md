# DataScienceBioLab Codebase Guide

> ℹ️ **Start Here**: First read [specs/integration/SPECS.md](specs/integration/SPECS.md) for the complete integration specification overview.

## Directory Structure

```
groundhog/
├── .cursor/              # Cursor IDE configuration
├── .github/              # GitHub configuration
├── analysis/             # Analysis tools and data
├── benches/              # Performance benchmarks
├── crates/              # Additional crates
├── docs/                # Documentation
├── examples/            # Example code and usage
├── income/              # Income analysis tools
├── reporting/           # Reporting functionality
├── specs/               # Specifications
│   ├── MVP/            # MVP requirements
│   ├── core/           # Core specifications
│   ├── income/         # Income specs
│   ├── integration/    # Integration specs
│   │   ├── SPECS.md
│   │   ├── AGENT.md
│   │   └── plugin-mcp-integration.md
│   ├── mcp/            # MCP specifications
│   ├── plugins/        # Plugin specifications
│   └── ui/            # UI specifications
├── src/                # Source code
│   ├── ai/mcp-tools/            # AI-related functionality
│   ├── bin/           # Binary executables
│   ├── core/          # Core functionality
│   ├── mcp/           # Machine Context Protocol
│   └── ui/            # User interface components
├── target/             # Build output directory
├── templates/          # Template files
├── tempRules/          # Temporary rules storage
├── tests/              # Test files
├── .gitignore         # Git ignore rules
├── AGENTS.md          # Agent configuration
├── Cargo.lock         # Dependency lock file
├── Cargo.toml         # Project manifest
├── CODEBASE.md        # This file
├── README.md          # Project overview
├── SPECS.md          # System specifications
├── TEAMCHAT.md       # Team communication
└── config.example.toml # Example configuration
```

## Core Components

### 1. System Core (`src/core/`)
```rust
pub mod protocol {
    // Protocol definitions and handlers
    pub mod v1;
    pub mod messaging;
    pub mod transport;
}

pub mod state {
    // State management system
    pub mod manager;
    pub mod sync;
    pub mod persistence;
}

pub mod events {
    // Event system
    pub mod dispatcher;
    pub mod handlers;
    pub mod queue;
}
```

### 2. Machine Context Protocol (`src/mcp/`)
```rust
pub mod protocol {
    // MCP core protocol implementation
    pub mod context;
    pub mod messaging;
    pub mod handlers;
}

pub mod tools {
    // MCP tool management
    pub mod registry;
    pub mod execution;
    pub mod validation;
}

pub mod state {
    // MCP state management
    pub mod context;
    pub mod persistence;
}
```

### 3. AI MCP Tools (`src/ai/mcp-tools/`)
```rust
pub mod tools {
    // AI-powered tool implementations
    pub mod codebase;
    pub mod context;
    pub mod execution;
}

pub mod agents {
    // AI agent implementations
    pub mod review;
    pub mod implementation;
    pub mod architecture;
    pub mod monitoring;
    pub mod testing;
}

pub mod integration {
    // AI tool integration
    pub mod hooks;
    pub mod plugins;
    pub mod handlers;
}
```

### 4. Integration Layer (`src/integration/`)
```rust
pub mod hooks {
    // Agent hook system
    pub mod preprocessor;
    pub mod postprocessor;
    pub mod error_handler;
}

pub mod tools {
    // Tool implementations
    pub mod registry;
    pub mod executor;
    pub mod validator;
}

pub mod plugins {
    // Plugin system
    pub mod loader;
    pub mod manager;
    pub mod sandbox;
}
```

### 5. User Interface (`src/ui/`)
```rust
pub mod components {
    // UI components
    pub mod dialog;
    pub mod progress;
    pub mod notification;
}

pub mod state {
    // UI state management
    pub mod store;
    pub mod reducers;
    pub mod actions;
}
```

## Version Control

This guide is version controlled alongside the codebase.
Updates are tagged with corresponding software releases.

---

Last Updated: [Current Date]
Version: 1.0.0 