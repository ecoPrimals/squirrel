<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Squirrel

**AI Coordination Primal** for the [ecoPrimals](https://github.com/syntheticChemistry) ecosystem.

**License**: [scyBorg](LICENSE) (AGPL-3.0 + ORC + CC-BY-SA 4.0) | **Build**: GREEN | **Tests**: 1,622 passing | **Rust**: 1.81+

---

## What is Squirrel?

Squirrel is a sovereign AI Model Context Protocol (MCP) service. It routes AI requests, manages context windows, coordinates multiple MCP servers, and provides vendor-agnostic model selection through runtime capability discovery.

Any OpenAI-compatible server, cloud API, or local model can plug in through the same interface. Squirrel discovers services at runtime — no hardcoded names, no compile-time coupling.

See [ORIGIN.md](ORIGIN.md) for the full story of how Squirrel was built using constrained evolution.

### Owns

- AI task routing and provider selection (cost, quality, latency)
- MCP protocol coordination
- Context window management and memory optimization
- Session management and configuration

### Delegates

- Auth and crypto to **BearDog** (via capability discovery)
- Data storage to **NestGate**
- Service mesh to **Songbird**
- GPU compute to **ToadStool**

---

## Quick Start

```bash
# Build
cargo build --release

# Run
./target/release/squirrel standalone

# Test
cargo test -p squirrel --lib

# Lint
cargo clippy --workspace -- -D warnings
```

### Socket Path

```
/run/user/<uid>/biomeos/squirrel.sock
```

---

## Architecture

```
TRUE PRIMAL: Self-knowledge only, discovers everything else at runtime.

IPC:       JSON-RPC 2.0 over Unix sockets (default)
Binary:    tarpc with automatic protocol negotiation
Transport: Unix sockets -> Named pipes -> TCP (automatic fallback)
HTTP:      Feature-gated OFF by default (axum/tower behind http-api)
```

### Capability-Based Discovery

```rust
let ai_services = ecosystem
    .find_services_by_capability(PrimalCapability::ModelInference)
    .await?;
```

### Vendor-Agnostic AI

- **Cloud**: OpenAI, Anthropic, Gemini via API keys
- **Local**: Any OpenAI-compatible server (Ollama, llama.cpp, vLLM) via `LOCAL_AI_ENDPOINT`
- **Hubs**: HuggingFace, ModelScope via `MODEL_HUB_CACHE_DIR`
- **Custom**: Universal provider interface

---

## Project Structure

```
squirrel/
├── crates/
│   ├── main/                  # Main library and binary
│   ├── core/
│   │   ├── mcp/              # MCP protocol + AI coordinator
│   │   ├── auth/             # Auth delegation (BearDog client)
│   │   ├── context/          # Context management + learning
│   │   ├── core/             # Core types (mesh feature-gated)
│   │   ├── interfaces/       # Core trait definitions
│   │   └── plugins/          # Plugin system
│   ├── config/               # Unified configuration
│   ├── tools/                # CLI, AI tools, rule system
│   ├── services/             # Command services
│   ├── sdk/                  # SDK for integration
│   ├── universal-constants/  # Shared constants
│   ├── universal-error/      # Unified error types
│   └── universal-patterns/   # Transport and traits
├── tests/                    # Integration + chaos tests
├── specs/                    # Specifications
├── examples/                 # Code examples
└── config/                   # Environment configs
```

---

## Code Standards

- `#![forbid(unsafe_code)]` on all crates
- Pure Rust: zero C dependencies in default build
- All source files under 1,000 lines
- SPDX license headers on every file
- `tracing` for logging (no `log` crate)
- `serde_yml` for YAML (no deprecated `serde_yaml`)
- `std::sync::LazyLock` / `OnceLock` for statics
- Capability-based discovery (no hardcoded primal names)

---

## License

**[scyBorg](LICENSE)** — the ecoPrimals triple-copyleft framework:

| Layer | License | Covers |
|-------|---------|--------|
| Software | AGPL-3.0-or-later | All code, binaries, tools, infrastructure |
| Mechanics | ORC | Primal interaction protocols, spring deployment niches, ecosystem topology, constrained evolution methodology |
| Creative | CC-BY-SA 4.0 | Documentation, papers, diagrams, specifications |
| Reserved | ORC Reserved Material | ecoPrimals branding, primal names, logos |

Governed by three independent nonprofits. No single entity can revoke any layer.

Copyright (C) 2026 ecoPrimals Contributors
