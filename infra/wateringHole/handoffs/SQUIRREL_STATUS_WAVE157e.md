# Squirrel Status — Wave 157e: G68 Platform Substrate Abstraction

**Date**: Aug 7, 2026
**From**: eastGate
**Commit**: (pending)

## G68 Implementation

Squirrel now has a `universal_patterns::platform` module implementing G68 Layers 1 and 2.
All raw `PermissionsExt` and `std::os::unix::fs::symlink` calls have been migrated to the
platform abstraction. No `PermissionsExt` imports remain in business or test code outside
the platform module itself.

### L1: Platform Links

| API | Unix | Windows |
|-----|------|---------|
| `create_capability_alias(path, name)` | `symlink(target, dir/name.sock)` | Write `dir/name.pipe` with pipe path |
| `cleanup_capability_alias(path, name)` | Remove symlink if same-directory | Remove `.pipe` discovery file |

Migrated: `unix_socket.rs` — `try_create_capability_domain_symlink()` and `cleanup_capability_domain_symlink()` now delegate to the platform module. Removed 3 unix-only helper functions (`capability_symlink_points_to_same_directory`, `remove_stale_capability_domain_symlink`, `CAPABILITY_DOMAIN_SYMLINK_NAME`).

### L2: Platform Access

| API | Unix | Windows |
|-----|------|---------|
| `set_access(path, level)` | `chmod` via `PermissionsExt` | `readonly` attribute |
| `set_access_async(path, level)` | Same, via `spawn_blocking` | Same |
| `check_world_accessible(path)` | Check `0o002` bit | Returns `false` (DACL deferred) |

`AccessLevel` enum: `OwnerExclusive` (0o700), `OwnerReadWrite` (0o600), `GroupReadable` (0o750), `Mode(u32)` (passthrough).

Migrated 4 call sites:
- `unix_socket.rs:ensure_biomeos_directory()` → `OwnerExclusive`
- `secret_store.rs:save()` → `OwnerReadWrite` (async)
- `listener.rs:try_bind()` → `Mode(perms)` (socket permissions)
- `cli/security.rs:perform_security_checks()` → `check_world_accessible()`

### L3: System Info

- `hostname()` now resolves on Windows via `COMPUTERNAME` env var
- Memory, disk, process RSS: documented as G68-L3-SYSINFO (requires `windows-sys`)

### Deferred (requires `windows-sys`)

| Item | What | Why deferred |
|------|------|--------------|
| G68-L2-DACL | Full Windows ACL manipulation | Needs `windows-sys` crate |
| G68-L3-SYSINFO | `GlobalMemoryStatusEx`, `GetDiskFreeSpaceEx` | Needs `windows-sys` crate |

## Verification

| Check | Result |
|-------|--------|
| `cargo check --workspace --tests --examples` | 0 errors |
| `cargo check --target x86_64-pc-windows-gnu --workspace --tests --examples` | **0 errors** |
| `cargo test --workspace --lib --tests` | **4,100 passed, 0 failed** |

## Codebase Snapshot

| Metric | Value |
|--------|-------|
| Workspace crates | 12 |
| Tests | 4,100 (+10 platform module tests) |
| Cross-arch (Windows) | **PASS** |
| `PermissionsExt` in business code | **0** (confined to platform module) |
| `std::os::unix::fs::symlink` in business code | **0** (confined to platform module) |
