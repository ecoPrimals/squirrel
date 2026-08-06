<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Squirrel Status Handoff — Wave 155p

**Date**: Aug 3, 2026 | **Wave**: 155p | **From**: squirrel team on eastGate
**To**: overwatch + upstream primal teams

## Current State

| Metric | Value |
|--------|-------|
| Tests | **6,986** passing (`--all-features`), 0 failures |
| Clippy | CLEAN (non-deprecated) |
| Formatting | CLEAN |
| Health | GREEN |
| IPC Methods | **44** registered (signal.plan + signal.dispatch new) |
| Capabilities | **39** in `niche::CAPABILITIES` |

## G18 Signal Graph Dispatch — WIRED

The primary deliverable this wave: **end-to-end `signal.dispatch`** through biomeOS signal graphs.

### What was done

1. **`signal.dispatch` JSON-RPC method** — New handler with four-strategy resolution cascade:
   - Local squirrel methods (system.ping, health.check, etc.)
   - Registered providers (`provider.register`'d springs)
   - Spring tool discovery (`mcp.tools.list`)
   - Capability-based socket scanning (provenance proxy pattern)

2. **Full niche registration** — signal.plan + signal.dispatch added to:
   - `CAPABILITIES`, `COST_ESTIMATES`, `SEMANTIC_MAPPINGS`
   - `operation_dependencies()`, `cost_estimates_json()`, `semantic_mappings_json()`
   - `capability_registry.toml`, JSON-RPC dispatch table

3. **Niche completeness sweep** — Added 12 previously missing capabilities

4. **Provenance methods promoted** — `find_provider_socket`, `forward_jsonrpc`, `discover_capability_socket` now `pub(crate)` for cross-handler reuse

### Signal dispatch flow

```
signal.plan (AI decomposes intent)
  → [{ signal: "neural.validate", params: {...} }, ...]

signal.dispatch (routes each step)
  → Strategy 1: local method? → dispatch_jsonrpc_method
  → Strategy 2: registered provider? → find_provider_socket → forward_jsonrpc
  → Strategy 3: spring tool? → SpringToolDiscovery → forward_jsonrpc
  → Strategy 4: socket scan? → discover_capability_socket → forward_jsonrpc
  → Error: no route found
```

### What's needed for full E2E

- **Live spring registration**: A spring (e.g., wetSpring, neuralSpring) must call `provider.register` to be discoverable
- **biomeOS signal graph orchestration**: biomeOS can now call `signal.plan` → iterate steps → call `signal.dispatch` for each
- **Cross-gate dispatch**: Currently UDS-local; cross-gate routing depends on inter-gate content.get E2E (#4)

## Pre-existing Debt Fixed

- `REQUIRED_CAPABILITIES` → `CONSUMED_CAPABILITIES` rename in discovery test
- `capability_registry.toml` path fix (stale `include_str!` path)
- TOML type inference annotations
- Bare `"health"` method exclusion from domain.method validation
- `cost_estimates_json()` macro recursion limit (refactored to programmatic builder)

## Upstream Dependencies (unchanged)

- **bearDog**: secrets.* JSON-RPC + BTSP strict mode — INTEGRATED
- **songBird**: UDS transport layer — INTEGRATED
- **biomeOS**: Signal graph orchestration — READY for integration (squirrel dispatch wired)
