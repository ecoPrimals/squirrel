# Squirrel Current Status

**Last Updated**: March 14, 2026
**License**: AGPL-3.0-only

---

## Build Health

| Metric | Value |
|--------|-------|
| Build | GREEN (0 errors) |
| Tests | 4,240 passing / 0 failed |
| Test Duration | Single-threaded (full workspace) |
| Clippy | CLEAN (-D warnings, 0 warnings) |
| Formatting | CLEAN |
| Doc Warnings | CLEAN (0 rustdoc warnings) |
| Test Coverage | 66% line, 68% region (workspace, via cargo llvm-cov; target: 90%) |
| Unsafe Code | 0 (all crates use `#![forbid(unsafe_code)]`) |
| Pure Rust | 100% (zero C deps, sqlx uses rustls) |
| SPDX Headers | All source files |
| File Sizes | All .rs files under 1,000 lines |
| License | AGPL-3.0-only on all crates |

---

## Architecture

| Principle | Status |
|-----------|--------|
| TRUE PRIMAL (self-knowledge only) | Complete |
| Capability-based discovery | Complete |
| Vendor-agnostic AI providers | Complete |
| Isomorphic IPC | Complete |
| Universal transport | Complete |
| Multi-protocol RPC (JSON-RPC 2.0 + tarpc) | Complete |
| gRPC/tonic | Fully removed |
| Zero unsafe code | Enforced (`#![forbid(unsafe_code)]` all crates) |

---

## Recent Evolution (March 14, 2026)

### Comprehensive Audit & Deep Debt Resolution

- **Tests**: 4,240 passing / 0 failed
- **Clippy**: CLEAN (-D warnings, 0 warnings)
- **Coverage**: 66% line, 68% region (target: 90%, via cargo llvm-cov)
- **File sizes**: All .rs files under 1,000 lines
- **License**: AGPL-3.0-only on all crates
- **gRPC/tonic**: Fully removed — JSON-RPC 2.0 + tarpc only
- **Capability-based discovery**: Complete (TRUE PRIMAL)
- **Unsafe code**: `#![forbid(unsafe_code)]` on all crates
- **SPDX headers**: All source files
- **Dependencies**: 100% Pure Rust (sqlx uses rustls, zero C deps)

---

## Test Suite

```
4,240 passed
0 failed
```

### Coverage Areas

- Unit tests across all crates
- Integration tests
- Chaos tests: network resilience, resource exhaustion, concurrency
- Config validation tests
- Environment variable tests (with named serial groups)

---

## Crate Structure

| Crate | Purpose |
|-------|---------|
| `squirrel` (main) | Main library + binary |
| `squirrel-mcp` | MCP protocol + enhanced AI coordinator |
| `squirrel-auth` | Authentication + JWT |
| `squirrel-context` | Context management + learning |
| `squirrel-core` | Service discovery + federation |
| `squirrel-interfaces` | Core trait definitions |
| `squirrel-plugins` | Plugin system |
| `squirrel-config` | Unified configuration |
| `squirrel-ai-tools` | AI tools + provider routing |
| `squirrel-cli` | CLI tools |
| `universal-constants` | Shared constants |
| `universal-error` | Unified error types |
| `universal-patterns` | Transport + traits |

---

## Production Features

- Unix socket JSON-RPC 2.0 server (gRPC/tonic fully removed)
- tarpc binary protocol with automatic negotiation
- Capability-based AI provider routing
- Vendor-agnostic local AI server support
- Graceful shutdown (Ctrl+C)
- Tracing spans for observability
- Environment variable configuration with multi-tier resolution
- Input validation, rate limiting, threat monitoring

---

## Deployment

### Binary

```bash
cargo build --release
# Output: target/release/squirrel (~4.5 MB, statically linked)
```

### Socket Path

```bash
/run/user/<uid>/biomeos/squirrel.sock
```

### Environment

Key environment variables (all optional, sensible defaults):

```bash
LOCAL_AI_ENDPOINT=http://localhost:11434
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
MCP_DEFAULT_MODEL=gpt-3.5-turbo
SQUIRREL_SOCKET=/custom/path.sock
```
