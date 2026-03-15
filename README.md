<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Squirrel AI Primal

**Universal AI Coordination Primal** for the ecoPrimals ecosystem.

**License**: AGPL-3.0-only | **Build**: GREEN | **Tests**: 1,622 passing | **Deps**: 272 (zero HTTP) | **Rust**: 1.81+

---

## Overview

Squirrel is the AI Coordination Primal of the [ecoPrimals](https://github.com/syntheticChemistry) ecosystem — a sovereign AI Model Context Protocol (MCP) service providing vendor-agnostic model routing, multi-MCP coordination, and context management through a TRUE PRIMAL architecture: zero compile-time coupling, runtime capability-based discovery, and isomorphic IPC. The codebase follows **Spring absorption** — evolving patterns from the ecosystem (e.g., SLO/tolerance registry, provenance tracking, XDG socket conventions) into shared primitives.

Squirrel originated from Geoffrey Huntley's [stdlib thesis](https://ghuntley.com/stdlib/) on treating AI coding assistants as autonomous agents. That insight evolved into a formal methodology — **constrained evolution** — where the AI proposes code variants, Rust's type system and borrow checker act as natural selection, and validation suites serve as the fitness function. See [ORIGIN.md](ORIGIN.md) for the full story.

### Core Capabilities

- **AI Processing**: Model inference, task routing, multi-MCP coordination
- **Context Management**: Advanced context window management, memory optimization
- **Intelligent Routing**: Cost, quality, and latency-based provider selection
- **Ecosystem Coordination**: Sovereign operation with optional ecosystem integration

### Architecture Principles

- **TRUE PRIMAL**: Self-knowledge only, discovers other primals at runtime
- **Vendor-Agnostic AI**: No compile-time coupling to any AI vendor -- supports any OpenAI-compatible server, cloud API, or model hub
- **Isomorphic IPC**: Same binary adapts to all platforms automatically (Linux, Android, Windows, macOS, BSD, WASM)
- **Capability-Based Discovery**: Runtime service discovery via capabilities, not hardcoded names
- **Pure Rust**: 100% Rust dependencies, zero C deps, zero unsafe code in production

---

## Quick Start

### Prerequisites

- Rust 1.81+ (stable) -- required for `std::sync::LazyLock` and `#[expect]`
- Cargo

### Build

```bash
# Build the project
cargo build --release

# Run all workspace tests
cargo test --workspace

# Check code quality (pedantic + nursery)
cargo clippy --workspace -- -D warnings -W clippy::pedantic -W clippy::nursery
```

### Run

```bash
# Start in standalone mode
./target/release/squirrel standalone

# Start with ecosystem coordination
./target/release/squirrel coordinate

# Show help
./target/release/squirrel --help
```

### Socket Path

```bash
# NUCLEUS-compliant path
/run/user/<uid>/biomeos/squirrel.sock
```

---

## Project Structure

```
squirrel/
├── crates/
│   ├── main/                  # Main library and binary
│   ├── core/                  # Core functionality
│   │   ├── mcp/              # MCP protocol + enhanced AI coordinator
│   │   ├── auth/             # Authentication and JWT
│   │   ├── context/          # Context management + learning
│   │   ├── core/             # Service discovery + federation
│   │   ├── interfaces/       # Core trait definitions
│   │   └── plugins/          # Plugin system
│   ├── config/               # Unified configuration
│   ├── tools/                # CLI, AI tools, rule system
│   ├── services/             # Command services
│   ├── integration/          # Integration libraries
│   ├── sdk/                  # SDK for Squirrel integration
│   ├── universal-constants/  # Shared constants
│   ├── universal-error/      # Unified error types
│   └── universal-patterns/   # Transport and traits
├── tests/                    # Integration + chaos test suites
├── specs/                    # Specifications
├── docs/                     # Additional documentation
└── archive/                  # Historical records
```

---

## Architecture

### TRUE PRIMAL Pattern

```
┌─────────────┐
│  Squirrel   │  Self-knowledge only
└──────┬──────┘
       │ Runtime Discovery:
       │
       ├──> AI Providers (via capability discovery, vendor-agnostic)
       ├──> Neural API (via socket scanning)
       ├──> Security (via capability discovery)
       └──> Peers (via registry)
```

### Capability-Based Discovery

```rust
// Discovers services by capability at runtime
let ai_services = ecosystem
    .find_services_by_capability(PrimalCapability::ModelInference)
    .await?;
```

### Vendor-Agnostic AI

All AI provider interactions are capability-based, not vendor-specific:

- **Cloud APIs**: OpenAI, Anthropic, Gemini, etc. via API key configuration
- **Local Servers**: Any OpenAI-compatible server (Ollama, llama.cpp, vLLM, LocalAI, etc.) via `LOCAL_AI_ENDPOINT`
- **Model Hubs**: HuggingFace, ModelScope, etc. via `MODEL_HUB_CACHE_DIR`
- **Custom Providers**: Register any provider via the universal provider interface

### Multi-Protocol RPC

- **JSON-RPC 2.0** over Unix sockets (default)
- **tarpc** with automatic protocol negotiation
- gRPC/tonic fully removed — 100% JSON-RPC + tarpc
- Universal transport: automatic fallback (Unix sockets -> Named pipes -> TCP)

---

## Testing

```bash
# Run workspace tests
cargo test --workspace

# Run main crate tests only
cargo test -p squirrel

# Run with coverage
cargo llvm-cov --workspace --html
```

### Test Coverage

- **1,622 tests** passing in the main crate (0 failures)
- Unit, integration, E2E, chaos, fault injection, and property-based testing
- Additional tests in `squirrel-mcp`, `squirrel-context`, `squirrel-commands`, etc.

---

## Configuration

### Environment Variables

Squirrel uses environment-first configuration with multi-tier resolution:

| Variable | Purpose | Default |
|----------|---------|---------|
| `LOCAL_AI_ENDPOINT` | Local AI server URL | `http://localhost:11434` |
| `LOCAL_AI_PORT` | Local AI server port | `11434` |
| `OPENAI_API_KEY` | OpenAI API key | -- |
| `ANTHROPIC_API_KEY` | Anthropic API key | -- |
| `MCP_DEFAULT_MODEL` | Default model | `gpt-3.5-turbo` |
| `SQUIRREL_SOCKET` | Custom socket path | auto-detected |

Legacy vendor-specific env vars (`OLLAMA_ENDPOINT`, `LLAMACPP_ENDPOINT`, etc.) are still supported as fallbacks.

See `squirrel.toml.example` for file-based configuration.

---

## Development

### Code Standards

- Zero unsafe code in production (`#![forbid(unsafe_code)]`)
- Production mocks isolated behind `#[cfg(test)]` / `#[cfg(testing)]`
- All files under 1,000 lines (smart refactoring)
- Capability-based discovery (not hardcoded primal types)
- `std::sync::LazyLock` / `OnceLock` for statics (no `lazy_static` / `once_cell`)
- Proper error handling throughout (no `unwrap()` in production paths)
- SPDX license headers on all source files

### Pre-Push Workflow

```bash
cargo build --release
cargo test --workspace
cargo clippy --workspace
```

### Contributing

1. Read [READ_ME_FIRST.md](READ_ME_FIRST.md) for current status
2. Follow TRUE PRIMAL architecture principles
3. Add tests for new functionality
4. Ensure all workspace tests pass before submitting

---

## License

**AGPL-3.0-only**

Copyright (C) 2026 DataScienceBioLab

This program is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, version 3 of the License.

See [LICENSE-AGPL3](LICENSE-AGPL3) for the complete license text.

### Network Service Requirement

Under AGPL Section 13, if you modify Squirrel and run it as a network service, you must offer users interacting with it remotely the opportunity to receive the Corresponding Source code.
