<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Squirrel

**AI Coordination Primal** for the [ecoPrimals](https://github.com/ecoPrimals) ecosystem.

**License**: [scyBorg](LICENSE) (AGPL-3.0-or-later + ORC + CC-BY-SA 4.0) | **Build**: GREEN | **Tests**: 6,809 passing | **Edition**: 2024 | **Coverage**: 90.1% region | **ecoBin**: 3.5 MB | **Methods**: 42+ IPC (42 registered + provenance proxy)

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
- Capability registry ([`config/capability_registry.toml`](config/capability_registry.toml))
- Deploy graph ([`squirrel_deploy.toml`](squirrel_deploy.toml))

### Delegates (via capability discovery — no hardcoded primal knowledge)

- Auth and crypto to any primal providing `security.*` capabilities
- Data storage to any primal providing `storage.*` capabilities
- Service mesh / HTTP proxy to any primal providing `network.*` capabilities
- GPU compute to any primal providing `compute.*` capabilities

---

## Quick Start

```bash
# Build (static musl binary — default target)
just build-ecobin

# Run (server mode — listens on Unix socket)
cargo run -p squirrel -- server

# Client (send a JSON-RPC call)
cargo run -p squirrel -- client --method health.liveness --params '{}'

# Test
cargo test --workspace --lib --tests

# Full CI gate (fmt + clippy + test + deny)
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

Capability symlink: `ai.sock` → `squirrel.sock` (auto-created for capability-based discovery)

### Auth Model

Squirrel does **not** expose `auth.mode` — it delegates all auth to the security capability provider (any primal advertising `security.*` capabilities). This is intentional: Squirrel is the AI coordination primal, not an auth server. TCP and UDS transports share the same JSON-RPC method surface; neither implements auth methods locally.

### Method Gate (JH-0)

Pre-dispatch capability gate at `crates/main/src/rpc/method_gate.rs`. Ships in **`GateMode::Permissive`** (no behavioral change). Classifies every JSON-RPC method as `Public` (health, identity, capabilities, discovery, auth, provenance) or `Protected` (AI inference, tool execution, context management). Prepares `CallerContext` and `ResourceEnvelope` structures for JH-2 enforcement when BearDog ionic token verification ships.

### Compute Delegation

Squirrel delegates compute workloads to the ecosystem compute primal (toadStool) via JSON-RPC IPC. Detection order: `COMPUTE_SERVICE_ENDPOINT` → `COMPUTE_ENDPOINT` → `TOADSTOOL_ENDPOINT` → local dev fallback. The `RemoteComputeProvider` translates `WorkloadExecutionSpec` into toadStool's `compute.execute` wire format and speaks JSON-RPC 2.0 over Unix socket or TCP.

### Inference Provider Discovery

At startup, `AiRouter` discovers inference providers from multiple sources:

1. **HTTP providers**: `AI_HTTP_PROVIDERS` env + vendor API keys
2. **Local AI**: `LOCAL_AI_ENDPOINT` / `OLLAMA_ENDPOINT` / `OLLAMA_URL` → Ollama-compatible HTTP
3. **Inference endpoints**: `INFERENCE_ENDPOINT` / `AI_INFERENCE_ENDPOINT` → auto-registers a `RemoteInferenceAdapter` for neuralSpring or any inference primal (UDS or HTTP)
4. **Socket hints**: `AI_PROVIDER_SOCKETS` → comma-separated Unix socket paths
5. **Socket scan**: `COMPUTE_SOCKET` → tiered capability discovery

Runtime registration: any primal can call `inference.register_provider` to dynamically add itself. UDS inference calls use a **120-second** read timeout by default (override via `SQUIRREL_INFERENCE_TIMEOUT_SECS`).

---

## Architecture

```
TRUE PRIMAL: Self-knowledge only, discovers everything else at runtime.

Fitness:   6,809 tests passing (0 failures) | ~1,023 `.rs` files | ~321k lines | zero Box<dyn Error> in prod

IPC:       JSON-RPC 2.0 over Unix sockets (default)
Binary:    tarpc with automatic protocol negotiation
TCP:       JSON-RPC 2.0 over TCP via `--port` + `--bind` (newline-delimited)
Transport: Unix sockets → Named pipes → TCP (automatic fallback)
Provider:  provider.register / provider.list / provider.deregister (spring registration)
Lifecycle: ecosystem lifecycle.register + ipc.register + 30s heartbeat
Niche:     niche.rs self-knowledge (capabilities, costs, dependencies, consumed)
Edition:   Rust 2024
ecoBin:    Pure Rust — zero C dependencies in default build
```

**JSON-RPC health (ecosystem standard):** `health.check`, `health.liveness`, and `health.readiness` are the **canonical** method names. The `system.*` names (for example `system.ping`) remain as **backward-compatibility aliases** only.

### Capability-Based Discovery

```rust
let ai_services = ecosystem
    .find_services_by_capability("ai.inference")
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

## Degradation Behavior

When Squirrel is unavailable, downstream consumers degrade as follows:

| Domain | Degradation | Severity |
|--------|-------------|----------|
| `ai.*` / `inference.*` | AI queries fail; consumers fall back to offline heuristics or cached responses | HIGH |
| `tool.*` | MCP tool routing unavailable; local tools still execute if consumer has them | MEDIUM |
| `context.*` | Context sessions unavailable; consumers operate stateless | LOW |
| `capabilities.*` / `identity.get` | Capability discovery fails; static configurations or cached responses used | LOW |
| `graph.*` | BYOB graph parsing unavailable; pre-validated graphs still deploy | LOW |
| `provider.*` | Spring registration queued; springs retry on reconnect | LOW |

**Standalone mode**: Squirrel operates fully without other primals. AI routing degrades
to local-only providers. Compute delegation falls back to `LocalProcessProvider`.
Storage endpoint resolution uses defaults. No primal dependency is hard-gated.

## Stadial Pairing

| Downstream Partner | Integration Surface | Validation |
|-------------------|---------------------|------------|
| esotericWebb | `ai.query`, `tool.execute`, `context.*` — agentic AI for game narratives | AI provider availability, tool routing |
| projectFOUNDATION | `ai.query`, `inference.*` — AI-assisted thread analysis | Inference endpoint discovery, model selection |
| neuralSpring | `inference.register_provider` — inference backend registration | Provider lifecycle, UDS timeout (120s) |
| all springs | `capabilities.list`, `identity.get` — discovery substrate | Canonical envelope shape compliance |

---

## Code Standards

- `unsafe_code = "forbid"` in workspace `[lints.rust]` — enforced across all 22 crates
- `clippy::expect_used` + `clippy::unwrap_used` = `deny` workspace-wide (test-only `cfg_attr` allows)
- `#![warn(missing_docs)]` on all library crates
- `cargo clippy` with `pedantic` + `nursery` + `cargo` lints — zero errors under `-D warnings`
- `#[expect(reason)]` over `#[allow]` — dead suppressions caught automatically
- `cargo fmt` — zero formatting violations
- Pure Rust: zero C dependencies in default build (ecoBin v3.0 compliant — `sysinfo` removed)
- Production files under 800 lines (test-only files may be larger)
- SPDX `AGPL-3.0-or-later` license header on all `.rs` files
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
| Software | AGPL-3.0-or-later | All code, binaries, tools, infrastructure |
| Mechanics | ORC | Primal interaction protocols, spring deployment niches, ecosystem topology, constrained evolution methodology |
| Creative | CC-BY-SA 4.0 | Documentation, papers, diagrams, specifications |
| Reserved | ORC Reserved Material | ecoPrimals branding, primal names, logos |

Governed by three independent nonprofits. No single entity can revoke any layer.

Copyright (C) 2026 ecoPrimals Contributors
