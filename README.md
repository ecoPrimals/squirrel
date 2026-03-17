<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Squirrel

**AI Coordination Primal** for the [ecoPrimals](https://github.com/ecoPrimals) ecosystem.

**License**: [scyBorg](LICENSE) (AGPL-3.0-only + ORC + CC-BY-SA 4.0) | **Build**: GREEN | **Tests**: 4,925 passing | **Edition**: 2024 | **Rust**: 1.85+ | **Coverage**: 69%

---

## What is Squirrel?

Squirrel is a sovereign AI Model Context Protocol (MCP) service. It routes AI requests, manages context windows, coordinates multiple MCP servers, and provides vendor-agnostic model selection through runtime capability discovery.

Any OpenAI-compatible server, cloud API, or local model can plug in through the same interface. Squirrel discovers services at runtime — no hardcoded names, no compile-time coupling.

See [ORIGIN.md](ORIGIN.md) for the full story of how Squirrel was built using constrained evolution.

### Owns

- AI task routing and provider selection (cost, quality, latency)
- MCP protocol coordination
- Context window management (`context.create` / `context.update` / `context.summarize`)
- Session management and configuration
- Capability registry ([`capability_registry.toml`](capability_registry.toml))
- Deploy graph ([`squirrel_deploy.toml`](squirrel_deploy.toml))

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

# Run (server mode — listens on Unix socket)
./target/release/squirrel server

# Client (send a JSON-RPC call)
./target/release/squirrel client --method system.ping --params '{}'

# Test
cargo test --workspace

# Lint (zero warnings required)
cargo clippy --workspace -- -D warnings

# Coverage
cargo llvm-cov --workspace
```

### Socket Path

```
$XDG_RUNTIME_DIR/biomeos/squirrel-${FAMILY_ID}.sock
```

Fallback: `/run/user/<uid>/biomeos/squirrel.sock` or `/tmp/squirrel.sock`.

---

## Architecture

```
TRUE PRIMAL: Self-knowledge only, discovers everything else at runtime.

IPC:       JSON-RPC 2.0 over Unix sockets (default)
Binary:    tarpc with automatic protocol negotiation
Transport: Unix sockets → Named pipes → TCP (automatic fallback)
HTTP:      Feature-gated OFF by default (optional dev/test only)
Lifecycle: biomeOS lifecycle.register + Songbird discovery.register + 30s heartbeat
Niche:     niche.rs self-knowledge (capabilities, costs, dependencies, consumed)
Edition:   Rust 2024
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
│   ├── integration/           # Context adapter, ecosystem integration
│   ├── ecosystem-api/         # Ecosystem API types and client
│   ├── universal-constants/   # Shared constants + primal identity
│   ├── universal-error/       # Unified error types
│   └── universal-patterns/    # Transport, security, federation traits
└── specs/                     # Specifications
```

---

## Code Standards

- `#![forbid(unsafe_code)]` unconditional on all 21 crates
- `#![deny(clippy::expect_used, clippy::unwrap_used)]` in production code
- `#![warn(missing_docs)]` on all library crates
- `cargo clippy` with `pedantic` + `nursery` lints — zero errors on `--all-features --lib`
- `cargo fmt` — zero formatting violations
- Pure Rust: zero C dependencies in default build (ecoBin compliant)
- All source files under 1,000 lines
- SPDX `AGPL-3.0-only` license on all crates (`[workspace.package]`)
- Edition 2024 across all 21 workspace crates
- `tracing` for structured logging (no `println!` in production)
- Typed errors via `thiserror` (no `Box<dyn Error>` in library code)
- Zero-copy patterns: `Arc<str>`, `bytes::Bytes`, `Arc<dyn Trait>` on hot paths
- Capability-based discovery (no hardcoded primal names)
- Property-based testing via `proptest` for serialization invariants

---

## License

**[scyBorg](LICENSE)** — the ecoPrimals triple-copyleft framework:

| Layer | License | Covers |
|-------|---------|--------|
| Software | AGPL-3.0-only | All code, binaries, tools, infrastructure |
| Mechanics | ORC | Primal interaction protocols, spring deployment niches, ecosystem topology, constrained evolution methodology |
| Creative | CC-BY-SA 4.0 | Documentation, papers, diagrams, specifications |
| Reserved | ORC Reserved Material | ecoPrimals branding, primal names, logos |

Governed by three independent nonprofits. No single entity can revoke any layer.

Copyright (C) 2026 ecoPrimals Contributors
