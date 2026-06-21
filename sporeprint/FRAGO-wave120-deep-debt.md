+++
title = "FRAGO — Wave 120 Deep Debt AAR → Overwatch"
description = "After Action Review: identity consolidation, feature gating, orchestrator refactor, deep scan results"
date = 2026-06-21

[taxonomies]
primals = ["squirrel"]
springs = ["primalSpring"]
+++

# FRAGO — Squirrel Wave 120 Deep Debt AAR

**From**: eastGate local (squirrel workstation)
**To**: primalSpring overwatch
**DTG**: 2026-06-21T12:41Z
**Classification**: UNCLASSIFIED // ecoPrimals internal
**Commits**: `1d256bb2`, `5f7e29b5`

---

## 1. SITUATION

Squirrel entered Wave 120 at 7,524 tests, 90%+ coverage, zero clippy warnings,
zero unsafe, zero TODOs, zero banned C deps. Deep scan of all 1,033 `.rs` files
across 22 workspace crates executed to identify remaining structural debt.

## 2. MISSION

Execute all remaining deep debt: identity consolidation, hardcoded string
elimination, feature gating of unwired subsystems, smart file refactoring,
production mock evolution.

## 3. EXECUTION — COMPLETED

### 3a. Identity Consolidation (P0)

| Before | After |
|--------|-------|
| 3 separate `"squirrel"` literals (`niche::PRIMAL_ID`, `core::PRIMAL_TYPE`, `capabilities::SELF_PRIMAL_NAME`) | Single canonical source: `universal_constants::capabilities::SELF_PRIMAL_NAME`; other two are re-exports |
| `deploy_graph.rs` hardcoded `"squirrel"` in `includes_squirrel()` | Uses `niche::PRIMAL_ID` + `niche::DOMAIN` |
| `jsonrpc_server.rs` fallback `"squirrel.sock"` | `concat!(env!("CARGO_PKG_NAME"), ".sock")` |
| `arc_str.rs` string cache `"squirrel"` literal | Uses `niche::PRIMAL_ID` |

**Result**: Zero hardcoded self-identity string literals in production code.

### 3b. Feature Gating — Context Learning (P0)

- ~14,600 lines (14 production modules + 16 test modules) gated behind `context-learning` feature
- Default build: **6,899 tests** (faster compile, smaller binary surface)
- `--all-features` build: **7,524 tests** (unchanged, all learning tests run)
- Subsystem is internally coherent and heavily tested but not runtime-wired
- External integration test file (`learning_types_tests.rs`) also gated

### 3c. Smart Refactor — Security Orchestrator (P1)

- `security/orchestrator/mod.rs`: 797 → **661 lines**
- Extracted `response.rs`: **148 lines** — violation tracking, policy determination, response execution
- `mod.rs` retains: orchestration pipeline (`check_security`), initialization, statistics, shutdown
- Architecture improved: enforcement logic separable from check pipeline

### 3d. Lint Hygiene (verified clean)

- Zero `#[allow(` anywhere in codebase — all evolved to `#[expect(reason)]`
- Zero `unwrap()`/`expect()` in production paths (workspace-wide `deny`)
- Zero `unsafe` blocks in executable code
- Zero `TODO`/`FIXME`/`HACK` comments

## 4. DEEP SCAN FINDINGS — RESIDUAL DEBT

### P1 — Actionable next wave

| Item | Location | Effort | Notes |
|------|----------|--------|-------|
| Nuclear Lineage riboCipher (`0xEE`) | `jsonrpc_connection_handler.rs:91–93` | Medium | Connection closed on `0xEE` preamble; needs BearDog key material or JSON-RPC error |
| DNS-SD stub | `discovery/mechanisms/dnssd.rs:84–86` | Medium | Falls back to socket registry; not true DNS-SD. Needs hickory-dns or disable |
| Near-800L watch list | `routing/agent.rs` (795), `universal_executor.rs` (795) | Low | Single-concern files; split when adding 2nd platform or routing strategy |

### P2 — Blocked / Phase 2

| Item | Scope | Blocked on |
|------|-------|------------|
| Context learning wiring | ~14.6k lines | Product decision: when to integrate learning engine into runtime context |
| Federation Phase 2 | `universal_executor.rs`, `service.rs`, mesh types | Federation protocol maturity |
| Plugin system Phase 2 | `FilePluginDiscovery`, web V2, default_manager | Web plugin architecture |
| OpenAI/Anthropic deprecated adapters | Behind `deprecated-adapters` feature | Decision: keep or remove for v0.2 |
| `integration/ecosystem` placeholder | 141 lines, no consumers | Merge into `ecosystem-api` or implement Phase 2 |
| tarpc/bincode RUSTSEC-2025-0141 | Transitive via tarpc | tarpc migration or fork; acknowledged in `deny.toml` |

## 5. METRICS

| Metric | Value |
|--------|-------|
| Tests (all features) | 7,524 |
| Tests (default) | 6,899 |
| Clippy warnings | 0 |
| `#[allow(` instances | 0 |
| Production unsafe | 0 |
| Files >800L (prod) | 0 |
| Banned C deps | 0 |
| Coverage | 90.14% region / 89.67% line |

## 6. UPSTREAM AUDIT REQUESTS

For overwatch review:

1. **Identity pattern**: Confirm `universal_constants::capabilities::SELF_PRIMAL_NAME` as ecosystem-wide canonical self-identity pattern for all primals
2. **Learning subsystem**: Product decision on `context-learning` — wire into runtime or prune?
3. **Nuclear Lineage**: BearDog team — key material spec for `0xEE` riboCipher tier?
4. **DNS-SD**: Architecture decision — real mDNS/DNS-SD via hickory-dns, or document socket-registry as the canonical discovery mechanism?
5. **tarpc RUSTSEC**: Ecosystem-wide decision on tarpc/bincode migration timeline

## 7. SIGNAL

Squirrel is **GREEN**. All deep debt that can be resolved without external
dependencies or product decisions has been resolved. Remaining items are
Phase 2 / cross-primal coordination.

Ready for next cascade directive.

---

*Filed from eastGate workstation, Wave 120 deep debt session.*
