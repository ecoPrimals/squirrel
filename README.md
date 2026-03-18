<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Squirrel

**AI Coordination Primal** for the [ecoPrimals](https://github.com/ecoPrimals) ecosystem.

**License**: [scyBorg](LICENSE) (AGPL-3.0-only + ORC + CC-BY-SA 4.0) | **Build**: GREEN | **Tests**: 4,730 passing | **Edition**: 2024 | **Rust**: 1.93+ | **Coverage**: 71%

---

## What is Squirrel?

Squirrel is a sovereign AI Model Context Protocol (MCP) service. It routes AI requests, manages context windows, coordinates multiple MCP servers, and provides vendor-agnostic model selection through runtime capability discovery.

Any OpenAI-compatible server, cloud API, or local model can plug in through the same interface. Squirrel discovers services at runtime — no hardcoded names, no compile-time coupling. Every port and endpoint is overridable via environment variables.

See [ORIGIN.md](ORIGIN.md) for the full story of how Squirrel was built using constrained evolution.

### Owns

- AI task routing and provider selection (cost, quality, latency)
- MCP protocol coordination
- Context window management (`context.create` / `context.update` / `context.summarize`)
- Human dignity evaluation on AI operations (discrimination, manipulation, oversight)
- Session management and configuration
- Capability registry ([`capability_registry.toml`](capability_registry.toml))
- Deploy graph ([`squirrel_deploy.toml`](squirrel_deploy.toml))

### Delegates (via capability discovery — no hardcoded primal knowledge)

- Auth and crypto to any primal providing `security.*` capabilities
- Data storage to any primal providing `storage.*` capabilities
- Service mesh / HTTP proxy to any primal providing `network.*` capabilities
- GPU compute to any primal providing `compute.*` capabilities

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

# Full CI gate (fmt + clippy + test + doc)
just ci

# Lint (zero warnings required)
just clippy

# Coverage
just coverage
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
ecoBin:    Pure Rust — zero C dependencies in default build
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
│   │   ├── auth/             # Auth delegation (capability-based client)
│   │   ├── context/          # Context management + learning
│   │   ├── core/             # Core types (mesh feature-gated)
│   │   ├── interfaces/       # Core trait definitions
│   │   └── plugins/          # Plugin system (unified manager)
│   ├── config/               # Unified configuration
│   ├── tools/                # CLI, AI tools, rule system
│   ├── services/             # Command services
│   ├── sdk/                  # SDK for integration
│   ├── integration/          # Context adapter, ecosystem integration
│   ├── ecosystem-api/        # Ecosystem API types and client
│   ├── universal-constants/  # Shared constants, primal identity, sys_info
│   ├── universal-error/      # Unified error types
│   └── universal-patterns/   # Transport, security, federation traits
├── specs/                    # Specifications
└── justfile                  # Build automation (just ci/test/clippy/coverage)
```

---

## Code Standards

- `#![forbid(unsafe_code)]` unconditional on all 22 crate roots
- `#![deny(clippy::expect_used, clippy::unwrap_used)]` in production code (test-only `cfg_attr` allows)
- `#![warn(missing_docs)]` on all library crates
- `cargo clippy` with `pedantic` + `nursery` lints — zero errors on `--all-features --all-targets`
- `cargo fmt` — zero formatting violations
- Pure Rust: zero C dependencies in default build (ecoBin v3.0 compliant — `sysinfo` removed)
- All source files under 1,000 lines
- SPDX `AGPL-3.0-only` license header on all 1,241 `.rs` files
- Edition 2024 across all 22 workspace crates
- `tracing` for structured logging (no `println!` in production code)
- Typed errors via `thiserror`; `.context()` on all key error paths
- Zero-copy patterns: `Arc<str>`, `bytes::Bytes`, `Cow<str>` on hot paths
- Capability-based discovery (no hardcoded primal names — `CapabilityIdentifier` replaces enum)
- Human dignity evaluation on AI operations (discrimination, manipulation, oversight checks)
- Property-based testing via `proptest` for serialization invariants
- Dev credentials env-only (no hardcoded secrets in source)

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
