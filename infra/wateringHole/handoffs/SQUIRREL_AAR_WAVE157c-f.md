# Squirrel AAR — Waves 157c–157f

**Date**: Aug 9, 2026
**From**: eastGate overwatch
**Scope**: Waves 157c through 157f (cross-arch compliance → G68 platform substrate → debris cleanup)
**Commits**: `c0980100` → `9ef3ca3e` (5 commits, 40 files, +781 / −1,083)

---

## MISSION

Bring squirrel to full cross-architecture compliance (G66 silicon deism elimination, G68 platform substrate abstraction) and clean remaining technical debt.

## DELIVERED

| Wave | What | Net Lines |
|------|------|-----------|
| **157c** | G66: Eliminate silicon deism — wrapped ~60 Unix-only test imports across 17 files with `#[cfg(unix)]` | ~+80 |
| **157d** | Cross-arch compliance — fixed all Windows-specific warnings (`get_socket_path` gating, `PathBuf` imports, dead code, unused imports). Added `reason=` to 12 `#![allow]` blocks. | +158 / −53 |
| **157e** | G68 Platform Substrate — created `universal_patterns::platform` module (L1 links, L2 access). Migrated 4 prod call sites from raw `PermissionsExt`/`symlink`. Improved `sys_info` Windows hostname. | +634 / −146 |
| **157f** | Orphan cleanup — removed 5 dead `.rs` files (895 lines). Fixed last unreasoned `#[allow]`. | +1 / −896 |

**Total**: −302 net lines. 10 new platform tests. 0 regressions.

## VERIFICATION

| Check | Result |
|-------|--------|
| `cargo check --workspace --tests --examples` | **0 errors, 0 warnings** |
| `cargo check --target x86_64-pc-windows-gnu --workspace --tests --examples` | **0 errors** |
| `cargo test --workspace --lib --tests` | **4,100 passed, 0 failed** |
| sourDough G68 scanner | **COMPLIANT** (0 violations) |

## CODEBASE SNAPSHOT

| Metric | Before (157c) | After (157f) |
|--------|---------------|--------------|
| Tests | 4,090 | 4,100 |
| Files (.rs) | 625 | 620 |
| Lines (.rs) | ~191K | ~190K |
| Crates | 12 | 12 |
| `PermissionsExt` in business code | 6 sites / 4 files | **0** (confined to `platform::access`) |
| `std::os::unix::fs::symlink` in business code | 1 site | **0** (confined to `platform::link`) |
| `#[allow]` without `reason=` | 13 | **0** |
| Orphan .rs files | 5 | **0** |
| Windows cross-arch | Warnings | **0 errors, 1 pre-existing docs warning** |

## ARCHITECTURE DECISIONS

1. **Platform module in `universal-patterns`** — colocated with transport (G66), same crate consumers. `AccessLevel` enum uses semantic names (`OwnerExclusive`, `OwnerReadWrite`) not mode bits. Unix-specific `Mode(u32)` variant for transport listener passthrough.

2. **Windows capability alias via discovery file** — Unix uses symlink (`ai.sock → squirrel.sock`); Windows uses a `.pipe` discovery file containing the named pipe path. Same semantic (capability-domain alias for IPC discovery), different mechanism.

3. **Windows ACL deferred** — Full DACL manipulation requires `windows-sys` crate. Current Windows `set_access` uses `readonly` attribute. Tracked as G68-L2-DACL.

4. **sys_info Windows: `COMPUTERNAME` env var** — Always set by Windows OS. No new crate dependency needed. Other sys_info stubs (memory, disk) documented as G68-L3-SYSINFO for `windows-sys` adoption.

5. **39 remaining `#[cfg(unix)]` blocks are intentional** — These are in UDS transport paths and security provider socket scanning. They're properly gated transport code, not platform abstraction violations. The G68 "platform substrate" spec targets permissions (L2) and links (L1), not the existence of Unix-specific transport.

## REMAINING WORK (NON-CODE)

| Item | Owner | Status |
|------|-------|--------|
| E2: squirrel systemd on ironGate | ironGate gate team | Deploy from golgi depot |
| footPrint agent panel | petalTongue → squirrel TCP JSON-RPC | Squirrel ready (TCP listener operational) |
| N2-N5 verification | primalSpring | squirrel is receiver, ready |

## DEBT STATUS

**Zero P0. Zero P1. No code debt remaining.**

- 0 `todo!()` / `unimplemented!()` / `unsafe` / `FIXME` / `HACK`
- 0 production `unwrap()`
- 0 `#[deprecated]` items
- 0 `#[allow]` without `reason=`
- 0 production files >800 lines
- 0 orphan files
- 0 `Box<dyn Error>` in return types (only 2 `From` boundary impls)

Squirrel is clean and ready for depot rebuild + gate deployment.
