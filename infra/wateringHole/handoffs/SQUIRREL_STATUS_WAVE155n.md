<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Squirrel Status Handoff — Wave 155n

**Date**: Jul 30, 2026 | **Wave**: 155n | **From**: squirrel team on eastGate
**To**: overwatch + upstream primal teams

## Current State

| Metric | Value |
|--------|-------|
| Tests | **7,138** passing / 0 failures (16 crates, `--all-features`) |
| Tests (default) | 6,453 passing |
| Tests (main crate) | 763 passing |
| Clippy | 0 non-deprecated warnings (`pedantic + nursery + cargo`) |
| Formatting | `cargo fmt --check` clean |
| Unsafe | 0 blocks (`unsafe_code = "forbid"`) |
| Files >800L (prod) | 0 (largest: 777L) |
| `.rs` files | 986 |
| Lines | ~306k |
| Edition | 2024 (Rust 1.94+) |
| ecoBin | 4.4 MB static-pie musl |
| Coverage | 90.1% region / 89.6% line |
| Mocks in prod | 0 (all `#[cfg(test)]`) |
| Hardcoded hosts/ports | 0 in prod (all via `universal-constants`) |
| TODO/FIXME/HACK | 0 |
| `#[expect(dead_code)]` | 32 (all Phase 2 placeholders with documented reasons) |
| Deprecated items | 44 (PrimalType migration-period + renamed config; zero non-test callers) |

## Completed Since Wave 155g

### Wave 155n — Hardcode Evolution + Lockfile Purge
- Extracted hardcoded timeouts/sizes from `storage_client`, `universal_primal_ecosystem`, `compute_adapter` into `universal_constants::timeouts` named constants.
- Last caller of deprecated `DEFAULT_BIND_ADDRESS` evolved to `get_bind_address()`.
- Regenerated `Cargo.lock` — purged stale entries (449→418 packages).

### Wave 155m — Clippy Deep Debt Sweep
- 150+ Clippy warnings fixed (auto + manual).
- Dead code elimination: `cleanup_if_needed`, unused struct fields → unit structs or `_`-prefixed.
- Idiomatic Rust: `let...else`, `#[must_use]`, consolidated match arms, significant-Drop extraction.
- Config struct refinement: `#[allow(clippy::struct_excessive_bools)]` on legitimate config types.

## Active IPC Integrations

| Upstream Capability | Method | Status |
|---------------------|--------|--------|
| `security.*` | `secrets.store/retrieve/list/delete` | WIRED — `SecurityProviderSecretStore` |
| `security.*` | BTSP `ClientHello` handshake | WIRED — `btsp_client.rs` |
| `network.*` | `http.request` delegation | WIRED — capability endpoint discovery |
| `compute.*` | `compute.execute` | WIRED — `UniversalComputeAdapter` IPC |
| `storage.*` | `storage.*` operations | WIRED — `StorageAdapter` IPC |
| `defense.*` | `defense.detect_anomaly`, `defense.classify_threat` | WIRED — delegation |
| any | `capabilities.list` / `primal.announce` | WIRED — JSON-RPC handlers |

## Dependency Audit Summary

| Category | Status |
|----------|--------|
| C deps | 0 direct — `deny.toml` bans 14 C-dep crates |
| `lazy_static` | Purely transitive (colored, prettytable, sharded-slab) |
| `tungstenite` | Transitive via axum (feature-gated behind `http-api`) |
| `ring`/`openssl`/`reqwest` | Confirmed absent from lockfile |
| `unsafe` blocks | 0 — `unsafe_code = "forbid"` |

## Gaps for Upstream Review

1. **Adapter E2E validation**: All universal adapters wired for IPC but need integration testing with live capability providers.
2. **`send_to_primal` runtime registry**: Currently depends on env vars for socket paths. Runtime discovery registry would eliminate this.
3. **Feature-gated test coverage**: 7,138 tests with `--all-features`; CI should run with `--all-features` to cover all paths.
4. **`EcosystemPrimalType` deprecation**: ~20 use sites with `#[allow(deprecated)]`; removal blocked on ecosystem-wide migration to `CapabilityIdentifier`.
