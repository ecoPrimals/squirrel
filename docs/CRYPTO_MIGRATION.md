<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Crypto Migration Guide

**Status**: Complete — pure-Rust default build current as of April 2026 (workspace v0.1.0)
**Last Updated**: April 20, 2026

## Summary

Squirrel's default build is **pure Rust** with zero C dependencies. All
cryptographic operations use pure-Rust implementations:

| Operation | Library | Notes |
|-----------|---------|-------|
| Hashing | `blake3` | Pure Rust BLAKE3 — workspace uses `default-features = false` with `features = ["pure"]` (no bundled C/SIMD assembly backend) |
| JWT signing | `ed25519-dalek` | **Feature-gated** behind `local-crypto` (squirrel-mcp crate) |
| Compression | `miniz_oxide` | Pure Rust deflate (flate2 backend) |

TLS (`rustls`) was removed during the stadial gate — Squirrel is IPC-first
(Unix sockets), so TLS is delegated to the security capability provider.

## ecoBin Compliance

The `deny.toml` bans 14 C-dependency crates to enforce pure-Rust builds:

- `ring`, `openssl`, `openssl-sys`, `native-tls`, `rustls` — all eliminated
- `libloading`, `pprof` — removed entirely
- `reqwest` — banned (Tower Atomic: IPC-first)
- `tokio-tungstenite` — banned (Tower Atomic)

## Feature-Gated Crypto

The `local-crypto` feature enables `ed25519-dalek` for local JWT signing
without network access. This is the only path that pulls in a crypto library:

```toml
[features]
local-crypto = ["ed25519-dalek"]
```

In production, crypto operations are delegated to the security capability
provider via `crypto.signing` capability discovery over Unix socket IPC.

## Migration Path (completed)

1. `openssl` / `native-tls` → removed (IPC-first, no HTTP clients)
2. `ring` / `rustls` → removed (eliminated from Cargo.lock during stadial gate)
3. `reqwest` → removed (IPC-first architecture; banned in `deny.toml`)
4. `pprof` → removed (was pulling `libunwind`)
5. `libloading` → removed (secure plugin stub)
6. `flate2` C backend → `miniz_oxide` pure Rust backend
7. `sysinfo` → pure Rust `/proc` parsing (`sys_info` module)
8. `nix` → `rustix` (pure Rust Linux syscalls, no libc FFI)
9. `nvml-wrapper` → removed (GPU monitoring delegated to compute capability provider)

## Verification

```bash
cargo deny check bans    # Verifies no banned C crates
cargo tree | grep ring    # Should return nothing
just doctor               # Full ecoBin compliance check
```
