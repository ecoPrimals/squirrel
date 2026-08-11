<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Squirrel Status — Wave 157g: Deep Debt Sweep + G72 Dependency Pandemic

**Date**: Aug 10, 2026
**From**: squirrel team @ eastGate
**Posture**: STADIAL SHIFT — deep debt excision + dependency pandemic

## Summary

Three commits in Wave 157g:

1. **G72 Tier 1**: Feature flags trimmed from `["full"]` to explicit minimal sets.
   18 dead dependencies excised. Cargo.lock: 384 → 333 packages (13%).
2. **Idiom + Sync + Capability**: `Arc<Mutex<bool>>` → `AtomicBool` (3 sites),
   `niche.rs` primal names → capability domains, 118x `unwrap_or(literal)` → `unwrap_or_default()`.
3. **Double-Lock + Idiom**: `Arc<Mutex<CommandRegistry>>` double-lock excised (all
   `CommandRegistry` methods are `&self` with interior mutability). Clone-on-copy fix,
   buffer-clone elimination in experience sampling, VecDeque direct serialization,
   `match` → `if let` for poison handlers, `Vec<&String>` → `Vec<&str>`.

## Metrics

| Metric | Value |
|--------|-------|
| Workspace crates | 12 |
| Source files | ~620 `.rs` |
| Source lines | ~190k |
| Cargo.lock packages | 333 |
| Tests passing | 4,100+ |
| Tests failing | 1 (pre-existing env-dependent flaky: `coordinate_storage_fails_without_provider`) |
| Cross-arch (Windows) | PASS |
| Build warnings | 0 |
| TODO/FIXME in code | 0 |

## Remaining Tracked Debt

| Item | Scope | Blocker |
|------|-------|---------|
| 224 fake-async functions | Workspace-wide | Most are trait-imposed — trait evolution required |

## Docs Cleanup (this handoff)

- Root docs aligned: test counts, crate counts (12 not 13), file/line metrics
- Broken symlink `crates/universal-patterns/ai.sock` removed
- `specs/SOCKET_REGISTRY_SPEC.md` capability table: primal names → capability discovery
- `sporeprint/validation-summary.md` fossil Status section replaced with current metrics
- Handoffs 157b–157e + AAR archived

## Status

- Build: GREEN
- Cross-arch: PASS
- G72 Tier 1: COMPLETE
- Deep debt sweep: COMPLETE (double-lock, AtomicBool, unwrap_or_default, idioms)
