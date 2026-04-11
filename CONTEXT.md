<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Context — Squirrel

## What This Is

Squirrel is the **Universal AI Coordination Primal** for the [ecoPrimals](https://github.com/ecoPrimals) ecosystem: a pure Rust service that routes AI workloads, speaks Model Context Protocol (MCP), manages context windows, and coordinates multiple MCP servers with vendor-agnostic model selection via runtime capability discovery. See [README.md](README.md) for build, sockets, and usage.

## Role in the Ecosystem

- **AI model routing:** Selects providers and routes inference by cost, quality, latency, and declared capabilities.
- **MCP protocol:** Implements MCP coordination and multi-server orchestration.
- **IPC:** Primary control plane is **JSON-RPC 2.0**; **tarpc** is used for high-performance/binary paths with protocol negotiation. Transports default to **Unix domain sockets**; named pipes and TCP exist as fallbacks. Discovery service (`discovery.register`, heartbeats) participates in service discovery alongside biomeOS lifecycle patterns.

## Technical Facts

| Item | Value |
|------|--------|
| Language | Rust, **edition 2024** |
| Workspace | **22** crates (see `Cargo.toml` `members`) |
| Scale | ~**331k** lines across **993** `.rs` files |
| Native deps | **Pure Rust** default build — no C dependencies in the standard ecoBin path |
| Code license | **AGPL-3.0-or-later** (workspace `license`; see License section for full public framing) |
| Version | **v0.1.0-alpha.48** (workspace); status **pre-alpha** |
| Unsafe code | 0 — `unsafe_code = "forbid"` in workspace `[lints.rust]` |

## Capabilities (domains)

- AI routing and provider abstraction (cloud APIs, local OpenAI-compatible servers, hubs).
- Context management (`context.create` / `context.update` / `context.summarize` and related flows).
- MCP protocol implementation and coordination.
- Plugin system (unified plugin manager under `crates/core/plugins`).
- CLI and developer tools (`squirrel-cli`, `squirrel-ai-tools`).
- Rule system (`squirrel-rule-system`).
- Ecosystem integration (capability discovery, `ecosystem-api` client/types).

## Key crates (names)

| Crate / area | Role |
|--------------|------|
| `squirrel` | Main binary and library (`crates/main`) |
| `squirrel-mcp` | MCP protocol and AI coordinator |
| `squirrel-core` | Core types and shared infrastructure |
| `squirrel-ai-tools` | AI tooling and routing-related tooling |
| `squirrel-cli` | Command-line interface |
| `universal-patterns` | Transport, security, federation-style traits |
| `universal-constants` | Shared constants, identity, sys_info |
| `ecosystem-api` | Ecosystem API types and client |

Other workspace crates include auth, context, interfaces, plugins, config, commands, SDK, integration adapters, `universal-error`, and adapter-pattern examples/tests.

## IPC and discovery (summary)

- **JSON-RPC 2.0** — primary RPC surface (e.g. `squirrel client --method …`).
- **tarpc** — binary RPC where negotiated; complements JSON-RPC for performance-sensitive paths.
- **Unix domain sockets** — default transport; paths under `$XDG_RUNTIME_DIR/biomeos/` (see README).
- **Discovery service** — registration/heartbeat with ecosystem discovery (paired with biomeOS lifecycle concepts in docs).

## Architecture

- **Capability-based discovery:** Services located by capability (e.g. `find_services_by_capability`); avoids hardcoded primal names where the design uses `CapabilityIdentifier`-style indirection.
- **Infant primal pattern:** Minimal fixed self-knowledge; peers and dependencies resolved at runtime.
- **Sovereignty-first:** No compile-time coupling to specific sibling primals; delegates auth, storage, network, compute to whatever offers matching capabilities.

## License (public / documentation)

**scyBorg** triple (see [LICENSE](LICENSE) and README):

| Layer | License | Scope |
|-------|---------|--------|
| Software | AGPL-3.0-or-later | Code and binaries |
| Mechanics | ORC | Protocols, deployment niches, topology |
| Creative | CC-BY-SA 4.0 | Docs, specs, diagrams (this file uses CC-BY-SA-4.0 SPDX in header) |

## Test suite

- **6,881** tests passing, 0 failures, **107** ignored.
- **Zero `.unwrap()`** and **zero `panic!()`** in production code — all error handling is typed.
- **Chaos** and integration tests (e.g. under `crates/main/tests/chaos`).
- **Property-based** tests (e.g. `proptest` for serialization invariants).
- **Coverage** ~**86%** line coverage (all features) with **90%** as stated target (re-verify with `just coverage`).

## What this does NOT do

Does not own long-term authoritative identity/crypto policy alone (delegates via capabilities); does not replace dedicated storage or GPU primals; HTTP serving is feature-gated and off by default.

## Related

- [README.md](README.md) — full quick start, socket paths, code standards.
- [wateringHole](https://github.com/ecoPrimals/wateringHole) — ecosystem standards and registry.
