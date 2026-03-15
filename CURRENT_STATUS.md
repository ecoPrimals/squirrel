<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Squirrel Current Status

**Last Updated**: March 15, 2026
**License**: AGPL-3.0-only

---

## Build Health

| Metric | Value |
|--------|-------|
| Build | GREEN (0 errors) |
| Tests | 1,622 passing / 0 failed (main crate) |
| Clippy | CLEAN |
| Formatting | CLEAN |
| Unsafe Code | 0 (`#![forbid(unsafe_code)]` all crates) |
| Pure Rust | 100% (zero C deps in default features) |
| SPDX Headers | All source files |
| License | AGPL-3.0-only on all crates |
| Unique Deps (non-dev) | 272 |
| HTTP Crates Compiled | 0 (feature-gated) |
| Deprecated Crates | 0 |

---

## Architecture

| Principle | Status |
|-----------|--------|
| TRUE PRIMAL (self-knowledge only) | Complete |
| Capability-based discovery | Complete |
| Vendor-agnostic AI providers | Complete |
| Isomorphic IPC | Complete |
| Multi-protocol RPC (JSON-RPC 2.0 + tarpc) | Complete |
| gRPC/tonic | Fully removed |
| HTTP stack (axum/tower/hyper) | Feature-gated OFF by default |
| Zero unsafe code | Enforced (`#![forbid(unsafe_code)]`) |
| sysinfo (C dependency) | Feature-gated behind `system-metrics` |
| serde_yaml (deprecated) | Replaced with `serde_yml` |
| log crate | Replaced with `tracing` (log bridge enabled) |

---

## Feature Gates

Code that belongs to other primals or optional subsystems is feature-gated OFF by default:

| Feature | What it gates | Primal owner |
|---------|---------------|--------------|
| `mesh` | Federation, routing, load balancing, service discovery, swarm | Songbird |
| `http-api` | Axum/Tower/Hyper HTTP stack | Legacy |
| `gpu-detection` | NVML/ROCm/nvidia-smi GPU detection | ToadStool |
| `system-metrics` | sysinfo crate (C dependency) | — |
| `local-jwt` | Local JWT signing (jsonwebtoken/ring) | BearDog |
| `websocket` | WebSocket transport (tokio-tungstenite) | — |

---

## Recent Evolution (March 15, 2026)

### Dependency Cleanup & Build Streamlining

- Removed 42 unique dependencies (314 → 272)
- Eliminated entire HTTP stack from default build (axum, tower, hyper = 0 crates)
- Replaced deprecated `serde_yaml` with `serde_yml` across 13 source files
- Migrated all `log::` macros to `tracing::` across 14 files
- Feature-gated `sysinfo` behind `system-metrics`
- Removed unused deps: `sled`, `argon2`, `rayon`, `crossbeam-channel`, `eyre`, `num_cpus`, `simple_logger`, etc.
- Replaced external `url`/`hex`/`urlencoding` with pure Rust implementations
- Clean build time reduced ~50% (from ~5 min to ~2.5 min)

### Primal Responsibility Cleanup

- Feature-gated Songbird code (federation/routing/swarm/ecosystem/service_discovery) behind `mesh`
- Feature-gated ToadStool code (GPU detection) behind `gpu-detection`
- Confirmed BearDog code (local JWT) already gated behind `local-jwt`
- Deleted 8 orphaned files never compiled
- Deleted empty MCP security module
- Removed deprecated CLI binary

---

## Crate Structure

| Crate | Purpose |
|-------|---------|
| `squirrel` (main) | Main library + binary |
| `squirrel-mcp` | MCP protocol + enhanced AI coordinator |
| `squirrel-mcp-auth` | Authentication delegation (BearDog client) |
| `squirrel-context` | Context management + learning |
| `squirrel-core` | Core types (mesh modules feature-gated) |
| `squirrel-interfaces` | Core trait definitions |
| `squirrel-plugins` | Plugin system |
| `squirrel-mcp-config` | Unified configuration |
| `squirrel-ai-tools` | AI tools + provider routing |
| `squirrel-cli` | CLI tools |
| `squirrel-commands` | Command services |
| `squirrel-sdk` | SDK for integration |
| `universal-constants` | Shared constants |
| `universal-error` | Unified error types |
| `universal-patterns` | Transport + traits |
| `ecosystem-api` | Ecosystem API types |

---

## Socket Path

```bash
/run/user/<uid>/biomeos/squirrel.sock
```
