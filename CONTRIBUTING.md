<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Contributing to Squirrel

Squirrel is the AI coordination primal of the ecoPrimals ecosystem.

## License

All contributions are accepted under the **scyBorg triple-copyleft** framework:

- **Code**: AGPL-3.0-only (see `LICENSE-AGPL3`)
- **Mechanics**: ORC (see `LICENSE-ORC`)
- **Creative**: CC-BY-SA 4.0 (see `LICENSE-CC-BY-SA`)

By submitting a pull request you agree that your contribution is licensed
under these terms.

## Standards

Every change must pass the wateringHole checklist before merge:

```
cargo fmt --all -- --check
cargo clippy --all-features --all-targets -- -D warnings
cargo test --all-features
cargo doc --all-features --no-deps
```

### Non-Negotiable

| Rule | Detail |
|------|--------|
| `#![forbid(unsafe_code)]` | Every crate root. No exceptions without hardware justification. |
| `clippy::pedantic + nursery + cargo` | Zero warnings under `-D warnings`. |
| `#[expect(reason)]` over `#[allow]` | Dead suppressions caught automatically. |
| `deny(unwrap_used, expect_used)` | Production code. Tests relax via `cfg_attr`. |
| No files > 1000 lines | Split into modules. |
| No TODO/FIXME/HACK | Track in wateringHole handoffs. |
| `domain.verb` method naming | All JSON-RPC methods. |
| SPDX headers | Every `.rs` file. |
| Pure Rust | No C dependencies in default features. `deny.toml` enforced. |

### Architecture

- **JSON-RPC 2.0 + tarpc** for IPC. No gRPC, no axum, no tower.
- **Capability-based discovery** at runtime. No hardcoded primal names for routing.
- **Self-knowledge only**: Squirrel knows its own capabilities; discovers others at runtime.
- **Zero-copy** where possible: `Arc<str>`, `bytes::Bytes`, `Cow<str>`, `&'static str`.
- **ecoBin compliant**: single binary, cross-compilation, platform-agnostic IPC.

### Testing

- Target: 90% line coverage via `cargo-llvm-cov`.
- Proptest for round-trip and fuzz.
- Chaos tests under `tests/chaos/`.
- E2E workflows under `tests/e2e/`.
- Doctests count as tests.

## Commit Messages

Use imperative mood. Focus on *why*, not *what*.

```
Evolve rate limiter whitelist to env-configurable

The loopback whitelist was hardcoded. Now reads SQUIRREL_RATE_LIMIT_WHITELIST
(comma-separated IPs) with loopback fallback, per capability-first standards.
```

## Handoffs

Session work is tracked via wateringHole handoffs. If your session is
incomplete, write a handoff to `wateringHole/handoffs/` before stopping.
