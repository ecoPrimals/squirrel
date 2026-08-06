<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Squirrel — Wave 156e: Deployment + Safety + Dead Code Elimination

**Date**: Aug 5, 2026
**Host**: eastGate
**Tests**: 7,241 passing / 0 failures / 0 warnings (`--all-features`)

## Changes

### E2: systemd Service Unit (`infra/squirrel.service`)

User-mode systemd service for ironGate deployment:
- `ExecStart=%h/.local/bin/squirrel server` (UDS-only, auto-detects socket path)
- `ExecStartPre=/bin/mkdir -p /run/user/%U/biomeos` (ensure socket dir)
- Hardening: `NoNewPrivileges`, `ProtectSystem=strict`, `ProtectHome=read-only`, `PrivateTmp`
- Socket cleanup on stop: removes `squirrel.sock` + `.pid`
- Unblocks petal-bridge `agent.*` → squirrel UDS routing

### RPC Entry Point Signature Evolution

`JsonRpcServer::new()` and `with_ai_router()`:
- **Before**: `pub fn new(socket_path: String)`
- **After**: `pub fn new(socket_path: impl Into<String>)`
- Eliminates forced `.clone()` at call sites; accepts `&str`, `PathBuf`, `Arc<str>`

### SDK WASM Safety

- 6 `Reflect::set(...).expect(...)` → `let _ = Reflect::set(...)` in `error/conversions.rs`
- `StringPool::get_or_create` → `entry().or_insert_with()` (eliminates `expect` on invariant)
- Production unwrap count: 8 → 1 (only `Semaphore::acquire` documented invariant remains)

### Dead Code Elimination

| Item | File | Action |
|------|------|--------|
| `queue_message` + `process_queued_messages` | `federation/network/messaging.rs` | Deleted (duplicated `tasks.rs` logic) |
| `ConnectionState` struct | `federation/network/types.rs` | Deleted (zero consumers) |
| `transition_to` method | `cli/plugins/example_plugin.rs` | Deleted (unused state machine method) |
| `node_info` field | `federation/network/core.rs` | Renamed `_node_info` |
| `keys` field | `federation/sovereign_data/encryption.rs` | Renamed `_keys` |
| `endpoints` / `components` | `plugins/web/adapter.rs` | Renamed `_endpoints` / `_components` |
| `manager` field | `plugins/web/marketplace.rs` | Renamed `_manager` |
| `handlers` / `security` | `mcp/protocol/handler/message_router.rs` | Renamed `_handlers` / `_security` |

## Remaining Debt (tracked, multi-wave)

| Item | Scope | Priority |
|------|-------|----------|
| `EcosystemPrimalType` / `PrimalType` | 42 files, ~500 refs | P1 (multi-wave) |
| `RegistryType` enum | 3 production files | P2 (only `Biomeos` variant works) |
| `PluginError` in SDK | 600+ refs | P3 |
| `AIError` in ai-tools | 3 consumer lines | P3 |
| Clone-heavy context/MCP hot paths | 5 files, ~75 clones | P2 (Arc evolution) |
| Rate limiter dead code (4 fields) | `rate_limiter/types.rs` | P3 |

## Cascade Notes

Ready for golgiBody cascade. squirrel.service ready for ironGate deployment.
