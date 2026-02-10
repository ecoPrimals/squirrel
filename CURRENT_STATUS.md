# Squirrel Current Status

**Last Updated**: February 9, 2026
**License**: AGPL-3.0-only

---

## Build Health

| Metric | Value |
|--------|-------|
| Build | GREEN (0 errors) |
| Tests | 1,957 passing / 0 failed / 146 ignored |
| Test Suites | 85 |
| Test Duration | ~25 seconds (full workspace) |
| Compile Warnings | 214 (non-deprecation: scaffolding for future features) |
| Unsafe Code | 0 in production |
| Pure Rust | 100% (zero C dependencies) |
| SPDX Headers | All source files |

---

## Architecture

| Principle | Status |
|-----------|--------|
| TRUE PRIMAL (self-knowledge only) | Complete |
| Capability-based discovery | Complete |
| Vendor-agnostic AI providers | Complete (Feb 9 evolution) |
| Isomorphic IPC | Complete |
| Universal transport | Complete |
| Multi-protocol RPC (JSON-RPC + tarpc) | Complete |
| Zero unsafe code | Enforced (`#![deny(unsafe_code)]`) |

---

## Recent Evolution (February 9, 2026)

### Vendor-Agnostic Evolution

Evolved all vendor-specific AI provider types to capability-based patterns:

- `OllamaProvider` + `LlamaCppProvider` -> `LocalServerProvider`
- `HuggingFaceProvider` -> `ModelHubProvider`
- `OllamaConfig` + `LlamaCppConfig` -> `LocalServerConfig`
- `HuggingFaceConfig` -> `ModelHubConfig`
- Environment: `LOCAL_AI_ENDPOINT` (agnostic) with `OLLAMA_ENDPOINT` fallback

### Dependency Evolution

- `lazy_static!` -> `std::sync::LazyLock` (across all crates)
- `once_cell::sync::Lazy` -> `std::sync::LazyLock`
- `once_cell::sync::OnceCell` -> `std::sync::OnceLock`
- Removed `lazy_static` and `once_cell` from 9 Cargo.toml files

### Code Quality

- Fixed unused imports across 5 files
- Moved test-only imports to `cfg(test)` modules
- Fixed environment variable race conditions (named serial groups)
- Evolved deprecated `AIError::Generic` to proper error variants
- All BIOME OS FIX comments cleaned to standard documentation

---

## Test Suite

```
85 test suites:
  1,957 passed
  0 failed
  146 ignored (doc tests for transport requiring runtime)
```

### Coverage Areas

- Unit tests across all crates
- Integration tests
- Chaos tests (13/15): network resilience, resource exhaustion, concurrency
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

- Unix socket JSON-RPC 2.0 server
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
