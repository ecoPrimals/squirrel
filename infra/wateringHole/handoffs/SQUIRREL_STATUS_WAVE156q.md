<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Squirrel — Wave 156q: Port Constants + Lint Hygiene + Dead Code

**Date**: Aug 6, 2026
**Host**: eastGate
**Tests**: 6,371 passing / 0 failures / 0 warnings (default features)
**Covers**: Wave 156q

---

## Port Literals → Named Constants

6 inline port numbers across 4 crates wired to `universal_constants::network` constants:

| File | Old | New Constant |
|------|-----|-------------|
| `core/core/src/federation/types.rs` | `port: 8080` | `DEFAULT_JSON_RPC_PORT` |
| `core/auth/src/lib.rs` | `8443` | `DEFAULT_SECURITY_PORT` |
| `universal-patterns/src/config/builder_presets.rs` | `.port(8080)` | `DEFAULT_JSON_RPC_PORT` |
| `universal-patterns/src/config/builder_presets.rs` | `.port(8081)` | `DEFAULT_HTTP_SERVICE_PORT` |
| `universal-patterns/src/config/builder_presets.rs` | `.port(8082)` | `DEFAULT_ADMIN_PORT` |
| `tools/cli/src/config_types.rs` | `unwrap_or(9000)` | `DEFAULT_MCP_TCP_PORT` |

3 new constants added to `universal-constants/src/network.rs`:
- `DEFAULT_HTTP_SERVICE_PORT: u16 = 8081`
- `DEFAULT_ADMIN_PORT: u16 = 8082`
- `DEFAULT_MCP_TCP_PORT: u16 = 9000`

## Lint Hygiene

- 5 `#[allow]`/`#[expect]` attributes converted to include `reason = "..."`:
  - `universal-patterns/src/security/providers/types.rs` — `#[allow(dead_code)]` + reason
  - `ecosystem-api/src/types/registration.rs` — `#![allow(deprecated)]` → `#![expect(deprecated, reason)]`
  - `ecosystem-api/src/traits/discovery.rs` — same
  - `tools/ai-tools/src/error.rs` — `#[expect(deprecated)]` + reason
- 1 unfulfilled `#![allow(deprecated)]` removed from `main/src/ecosystem/registry/discovery.rs` (module had zero deprecated item usage)

## Dead Code Eliminated

- `infer_primal_type_from_capability()` — deprecated function with zero callers across entire workspace. Definition removed from `ecosystem/types.rs`, re-export removed from `ecosystem/mod.rs`.

## Remaining Deprecated Surface

All fossils properly quarantined:

| Fossil | Location | Status |
|--------|----------|--------|
| `PluginError` (SDK) | `crates/sdk/src/infrastructure/error/core.rs` | `pub(crate)`, serde compat only |
| `EcosystemPrimalType` | `crates/main/src/ecosystem/types.rs` | `#[deprecated]`, test + serde compat |
| `AIError` (ai-tools) | `crates/tools/ai-tools/src/error.rs` | `#[deprecated]`, confined to file |
| `PrimalType` (ecosystem-api) | `crates/ecosystem-api/src/types/primal.rs` | `#[deprecated]`, migration compat |

## Debt Survey Post-156q

| Category | Status |
|----------|--------|
| Production `.unwrap()` | 0 |
| Production files >800 lines | 0 |
| TODO/FIXME markers | 0 |
| SDK `PluginError` leakage | 0 |
| Inline port literals (production) | 0 remaining |
| `#[allow]` without reason | 0 remaining (1 intentional `#[allow]` has `reason`) |
