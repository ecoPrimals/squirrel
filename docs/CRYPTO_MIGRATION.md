# Crypto Dependency Migration: Toward Pure Rust

**ecoPrimals Standard**: Pure Rust crypto (RustCrypto), no openssl/ring.

**Current State**: When HTTP features are enabled, `reqwest` with `rustls-tls` pulls in `ring`, which contains C and assembly code. All reqwest usage is optional (dev/testing); production uses Unix sockets.

## Proof of Concept: ecosystem-api

ecosystem-api has been upgraded to reqwest 0.12 as a proof of concept. It compiles successfully with `--features http-api`. Other crates remain on reqwest 0.11 until full migration.

## Crates with Optional reqwest

| Crate | Feature | Purpose |
|-------|---------|---------|
| ecosystem-api | http-api, http-client | **reqwest 0.12** (proof of concept) |
| squirrel-plugins | marketplace | Plugin marketplace HTTP |
| squirrel-core | http-client | HTTP client utilities |
| squirrel-mcp-auth | http-auth | HTTP-based auth |
| ecosystem-api | http-api, http-client | Service mesh HTTP APIs |
| squirrel-mcp-config | http-config | HTTP config fetching |
| universal-patterns | http-patterns | HTTP orchestration patterns |
| squirrel-ai-tools | direct-http | Direct AI vendor HTTP |
| squirrel-mcp | direct-http | AI provider HTTP |
| squirrel-sdk | http | WASM HTTP client |
| squirrel-cli | http-commands | CLI HTTP commands |

## Dependency Chain

```
reqwest 0.11 → hyper 0.14 → hyper-rustls 0.24 → rustls 0.21 → ring (C/ASM)
```

## Upgrade Path: reqwest 0.12 + Pluggable Crypto

reqwest 0.12 uses hyper 1.0 + rustls 0.23. Rustls 0.23+ supports pluggable crypto backends:

| Backend | C/Rust | Status |
|---------|--------|--------|
| ring (default) | C/ASM | Current |
| aws-lc-rs | C (BoringSSL fork) | Replaces ring, not pure Rust |
| rustls-rustcrypto | 100% Rust (RustCrypto) | Alpha (0.0.2), not production-ready |

### reqwest 0.12 Feature Options

- `rustls-tls-webpki-roots` (default) — uses ring
- `rustls-tls-manual-roots-no-provider` — no built-in crypto; user installs provider at runtime

### Pure Rust Path (when rustls-rustcrypto matures)

```toml
[dependencies]
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls-manual-roots-no-provider"] }
rustls = { version = "0.23", default-features = false }
rustls-rustcrypto = "0.0"  # Alpha; install_default() before Client::new()
```

```rust
// Before creating any reqwest Client:
rustls::crypto::rustcrypto::default_provider()
    .install_default()
    .expect("crypto provider already installed");
```

### aws-lc-rs Path (replaces ring, still C)

```toml
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls-manual-roots-no-provider"] }
```

```rust
rustls::crypto::aws_lc_rs::default_provider()
    .install_default()
    .expect("crypto provider already installed");
```

## Migration Considerations

1. **reqwest 0.11 → 0.12**: API largely compatible (Client, RequestBuilder, Method). Some users report HTTP/2 timeouts or 400 errors; test before full rollout.
2. **No-provider setup**: Must call `install_default()` once before any reqwest Client creation. Best done at application startup.
3. **Transitive deps**: anthropic-sdk already pulls reqwest 0.12; workspace has both 0.11 and 0.12. Unifying to 0.12 reduces duplication.
4. **sqlx**: Workspace uses `runtime-tokio-rustls` which also brings ring. Separate migration path; sqlx may add aws-lc-rs support in future.

## Recommendation

- **Short term**: Keep reqwest 0.11. All HTTP features remain optional; default build is pure Rust.
- **Medium term**: Upgrade to reqwest 0.12 when ready; use `rustls-tls-manual-roots-no-provider` + aws-lc-rs to replace ring (reduces C surface; not pure Rust).
- **Long term**: When rustls-rustcrypto reaches stable, switch to 100% Rust crypto for HTTP features.

## Verification

Default build (no HTTP features) has zero ring:

```bash
cargo build --workspace  # No reqwest, no ring
```

With HTTP features:

```bash
cargo build --workspace --features "http-api"  # Brings reqwest → ring
```
