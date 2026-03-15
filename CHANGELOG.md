<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Changelog

All notable changes to Squirrel will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Pre-alpha history is preserved as fossil record in
`ecoPrimals/archive/squirrel-pre-alpha-fossil-mar15-2026/docs/CHANGELOG.pre-alpha.md`.

## [0.1.0-alpha.2] - 2026-03-15

Comprehensive audit and standards alignment session. All wateringHole quality
gates now pass.

### Changed

- **Edition 2024**: All 22 crates upgraded from edition 2021 to 2024
  - Fixed `gen` reserved keyword (7 files)
  - Wrapped `std::env::set_var`/`remove_var` in `unsafe {}` for test code
  - `#![forbid(unsafe_code)]` → `#![cfg_attr(not(test), forbid(unsafe_code))]`
  - Collapsed nested `if` statements using let-chains (~50+ instances)
- **License**: `AGPL-3.0-or-later` → `AGPL-3.0-only` in all 23 Cargo.toml and 1,280 SPDX headers
- **Documentation**: Added `#![warn(missing_docs)]` to all 22 library crates; ~1,600 doc comments added
- **Clippy**: All code quality lints resolved — workspace passes `clippy -- -D warnings` clean

### Fixed

- 8 formatting violations (`cargo fmt`)
- 3 doc warnings (HTML tags in doc comments, unresolved links)
- 5 TODO/FIXME comments removed from committed code
- 8 failing plugin loading tests (implemented `load_plugins_from_directory`)
- 5 broken doctests (wrong crate name, uncompilable examples)
- Journal clone bug in `squirrel-commands` (invalid JSON in `SerializationError` clone)
- Redundant closure in capability registry
- Dead code in websocket transport (removed unused stubs)

### Removed

- Stale `run_examples.sh` script from adapter-pattern-examples

### Metrics

| Metric | Before | After |
|--------|--------|-------|
| Edition | 2021 | 2024 |
| License | AGPL-3.0-or-later | AGPL-3.0-only |
| `cargo fmt` | FAIL (8 files) | PASS |
| `cargo clippy -D warnings` | FAIL (couldn't run) | PASS (0 errors) |
| `cargo doc` warnings | 3 | 0 |
| Missing docs enforcement | 5/22 crates | 22/22 crates |
| Tests | 8 failing | All pass |
| Coverage (llvm-cov) | Unmeasured | ~66% |
| TODO/FIXME in code | 5 | 0 |

## [0.1.0-alpha] - 2026-03-15

First public alpha release. Squirrel is the AI Coordination Primal of the
ecoPrimals ecosystem — a sovereign MCP service for routing AI requests,
managing context, and coordinating multiple model providers.

### Highlights

- **3,749+ tests** passing across 22 crates, 0 failures
- **Zero C dependencies** in default build (pure Rust)
- **Zero unsafe code** (`#![forbid(unsafe_code)]` on all crates)
- **scyBorg license** — AGPL-3.0-only + CC-BY-SA 4.0
- **Capability registry** — `capability_registry.toml` as single source of truth
- **biomeOS lifecycle** — `lifecycle.register` + 30s heartbeat + SIGTERM cleanup
- **Context RPC methods** — `context.create`, `context.update`, `context.summarize`

### Architecture

- TRUE PRIMAL design: self-knowledge only, runtime capability discovery
- JSON-RPC 2.0 over Unix sockets (default IPC)
- tarpc binary protocol with automatic negotiation
- Transport hierarchy: Unix sockets → named pipes → TCP
- HTTP/WebSocket feature-gated OFF by default
- Vendor-agnostic AI: OpenAI, Anthropic, Gemini, local models (Ollama, llama.cpp, vLLM)
- Capability-based tool definitions with JSON Schema (`input_schema`) — McpToolDef pattern
- Deploy graph (`squirrel_deploy.toml`) for BYOB biomeOS deployment

### Feature Gates

| Feature | Purpose | Default |
|---------|---------|---------|
| `capability-ai` | Capability-based AI routing | ON |
| `ecosystem` | Ecosystem integration | ON |
| `tarpc-rpc` | High-performance binary RPC | ON |
| `delegated-jwt` | Capability-based JWT delegation | ON |
| `system-metrics` | sysinfo C dependency | OFF |
| `monitoring` | Prometheus metrics | OFF |
| `gpu-detection` | ToadStool GPU detection | OFF |
| `local-jwt` | Local JWT (brings ring C dep) | OFF |

### Spring Absorption & Primal Integration

- Added `capability_registry.toml` (wetSpring pattern) — replaces hardcoded capability lists
- Added `squirrel_deploy.toml` (airSpring pattern) — BYOB deploy graph with germination order
- Registry loader (`capabilities/registry.rs`) — TOML→JSON schema conversion, compiled fallback
- `handle_discover_capabilities` reads from registry instead of hardcoded vec
- `handle_list_tools` enriched with `input_schema` from registry (neuralSpring McpToolDef pattern)
- `capability.announce` treats `capabilities` as tool routing fallback (neuralSpring adapter fix)
- biomeOS lifecycle module: `lifecycle.register`, `lifecycle.status` heartbeat, signal handlers
- Context RPC methods wired: `context.create`, `context.update`, `context.summarize`
- BearDog crypto discovery aligned to biomeOS socket scan
- ToadStool AI provider auto-discovered via biomeOS socket scan
- SIGTERM/SIGINT signal handlers with socket file cleanup

### Cleanup from pre-alpha

- Reduced unique dependencies from 314 to 272
- Eliminated HTTP stack from default build
- Feature-gated all cross-primal code (Songbird, ToadStool, BearDog, NestGate)
- Replaced deprecated crates (`serde_yaml` → `serde_yml`, `log` → `tracing`)
- Purged PII, large artifacts, and stale code from git history
- Fixed deadlock in ExperienceReplay (RwLock re-entrance)
- Fixed all MCPError Display formatting (missing `#[error]` attributes)
- Fixed squirrel-mcp-auth feature interaction (delegated-jwt vs local-jwt)
- Resolved all build warnings across workspace
- Archived 420+ stale docs, scripts, and showcase files
