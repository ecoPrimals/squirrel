<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Squirrel Status — Wave 157g: Subsystem Excision + Deep Debt Sweep + G72

**Date**: Aug 10, 2026
**From**: squirrel team @ eastGate
**Posture**: STADIAL SHIFT — absorbed subsystem excision + dependency pandemic + idiom evolution

## Summary

Five commits in Wave 157g:

1. **G72 Tier 1**: Feature flags trimmed from `["full"]` to minimal sets.
   18 dead dependencies excised. Cargo.lock: 384 → 333 packages (13%).
2. **Idiom + Sync + Capability**: `Arc<Mutex<bool>>` → `AtomicBool` (3 sites),
   `niche.rs` primal names → capability domains, 118x `unwrap_or(literal)` → `unwrap_or_default()`.
3. **Double-Lock + Idiom**: `Arc<Mutex<CommandRegistry>>` excised, clone/idiom fixes.
4. **Docs alignment**: Root docs metrics reconciled, handoffs archived, debris cleaned.
5. **Absorbed subsystem excision**: `self_healing/` (878 LOC) + `hardware/gpu.rs`
   (704 LOC) excised. 15 fake-async fns converted to sync. `context-learning` +
   `context-visualization` marked DEPRECATED.

## Excisions

| Module | Lines | Reason |
|--------|------:|--------|
| `self_healing/` | 878 | Always compiled, zero callers — health via capability IPC |
| `hardware/gpu.rs` | 704 | GPU detection → compute primals (ToadStool/barraCuda) |
| `gpu-detection` feature | — | Feature flag removed from Cargo.toml |

## Deprecated (feature-gated OFF, zero callers)

| Module | Lines | Future owner |
|--------|------:|-------------|
| `context-learning/` | ~14.6k | neuralSpring / ToadStool (RL/training) |
| `context-visualization/` | ~2.2k | petalTongue (presentation via IPC) |

## Metrics

| Metric | Value |
|--------|-------|
| Workspace crates | 12 |
| Source files | 616 `.rs` |
| Source lines | ~188k |
| Cargo.lock packages | 333 |
| Tests passing | 1,539 |
| Tests failing | 1 (pre-existing env-dependent flaky) |
| Cross-arch (Windows) | PASS |
| Build warnings | 0 |
| TODO/FIXME in code | 0 |
| Fake-async remaining | 53 trait-imposed + 38 public contract + ~17 in deprecated context-learning |

## Remaining Tracked Debt

| Item | Scope | Note |
|------|-------|------|
| Fake-async (trait-imposed) | 53 functions | Correct — real impls need async |
| Fake-async (interface contract) | 38 functions | Callers `.await`; churn > value |
| Fake-async (context-learning) | ~17 functions | In deprecated, feature-gated code |

## Status

- Build: GREEN
- Cross-arch: PASS
- G72 Tier 1: COMPLETE
- Deep debt sweep: COMPLETE
- Subsystem excision: COMPLETE
