<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Crypto Migration Guide

**Status**: Complete (as of alpha.46)
**Last Updated**: April 13, 2026

## Summary

Squirrel's default build is **pure Rust** with zero C dependencies. All
cryptographic operations use pure-Rust implementations:

| Operation | Library | Notes |
|-----------|---------|-------|
| TLS | `rustls` | Pure Rust TLS via `rustls-tls` feature |
| Hashing | `blake3` | Pure Rust BLAKE3 (no C `blake3` backend) |
| JWT signing | `ed25519-dalek` | **Feature-gated** behind `local-crypto` |
| Compression | `miniz_oxide` | Pure Rust deflate (flate2 backend) |

## ecoBin Compliance

The `deny.toml` bans 14 C-dependency crates to enforce pure-Rust builds:

- `ring`, `openssl`, `openssl-sys`, `native-tls` — use `rustls` instead
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

1. `openssl` / `native-tls` → `rustls-tls` (all HTTP clients)
2. `ring` → removed (no direct dependency)
3. `reqwest` → removed (IPC-first architecture)
4. `pprof` → removed (was pulling `libunwind`)
5. `libloading` → removed (secure plugin stub)
6. `flate2` C backend → `miniz_oxide` pure Rust backend
7. `sysinfo` → pure Rust `/proc` parsing (`sys_info` module)

## Verification

```bash
cargo deny check bans    # Verifies no banned C crates
cargo tree | grep ring    # Should return nothing
just doctor               # Full ecoBin compliance check
```
