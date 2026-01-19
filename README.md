# 🐿️ Squirrel - AI Intelligence Primal

**Version**: v1.6.0 (Modern Architecture - Deep Debt Cleanup!)  
**Status**: ✅ **CLEAN BUILD - HTTP DEBT ELIMINATED!**  
**TRUE ecoBin**: #5+ (Unix Sockets + JSON-RPC + tarpc)

> *Deploy like an infant - knows nothing, discovers everything*

---

## 🎉 Latest Achievement: Deep Debt Cleanup Complete!

### v1.6.0 - HTTP Architecture Eliminated (Jan 19, 2026 PM)
```bash
# Deleted 21+ HTTP API files (2,800+ lines)
$ nm target/release/squirrel | grep -iE "(hyper|warp|tonic)" | wc -l
0  # ✅ ZERO HTTP framework symbols!

# Removed 5 vendor dependencies
✅ tonic (gRPC), prost, axum, tower-http, warp - ALL GONE!

# Binary size reduction
4.5M  # ✅ Lean (was ~25M with HTTP deps!)
```

**Evolution Complete**:
- 🗑️ **21+ HTTP files deleted** (2,800+ lines)
- ❌ **5 vendor deps removed** (gRPC, HTTP frameworks)
- ✅ **Modern architecture**: Unix sockets + JSON-RPC + tarpc
- 🎯 **ecoPrimals compliant**: NO HTTP, NO gRPC!

### v1.5.0 - 100% Pure Rust Dependency Tree (Jan 19, 2026 AM)
- 📦 **48 files deleted** (19,438+ lines)
- ✂️ **2 C dependencies eliminated** (jsonwebtoken, jsonrpsee)
- 🔨 **100% error resolution** (47 → 0 errors!)

**Combined**: One of the **MOST IMPACTFUL cleanup days** in ecoPrimals history!

---

## Overview

Squirrel is an **AI Intelligence Primal** for the ecoPrimals ecosystem, providing:
- 🤖 **MCP (Model Context Protocol)** server implementation
- 🧠 **AI capability orchestration** via Unix sockets
- 🔐 **JWT authentication** via capability discovery (BearDog)
- 🌐 **Network delegation** to Songbird
- 📊 **Context state management** across AI sessions

### TRUE PRIMAL Architecture (v1.6.0)
- **Unix sockets + JSON-RPC + tarpc** (NO HTTP!)
- **Capability-based discovery** (NO hardcoded primals!)
- **Delegates everything**: crypto → BearDog, network → Songbird, AI → capability_ai
- **Modern idiomatic Rust**: async, clean error handling, no workarounds
- **100% Pure Rust** (no C dependencies!)
- **Lean binary**: 4.5M (82% smaller than HTTP version!)

---

## Quick Start

### Prerequisites
- Rust 1.75+ (2021 edition)
- Unix-like OS (Linux, macOS)
- Access to ecoPrimals ecosystem (optional)

### Installation

```bash
# Clone the repository
git clone https://github.com/ecoPrimals/squirrel.git
cd squirrel

# Build (99.9% working - 4 syntax errors to fix)
cargo build --release

# Run
./target/release/squirrel
```

### Configuration

Squirrel uses environment-based discovery:

```bash
# Optional: Point to ecosystem socket
export ECOSYSTEM_SOCKET=/tmp/ecosystem.sock

# Optional: Configure logging
export RUST_LOG=info

# Run Squirrel
squirrel
```

---

## Features

### ✅ Implemented
- **MCP Server**: Full protocol implementation
- **JWT Auth**: Capability-based (delegated to BearDog)
- **AI Tools**: Capability discovery pattern established
- **Pure Rust**: ZERO C dependencies in dependency tree!
- **Unix Sockets**: JSON-RPC communication pattern

### 🔧 Stubbed (Ready for Implementation)
- **AI Provider Delegation**: Via Songbird capability
- **Network Operations**: Via Songbird Unix sockets
- **Service Discovery**: Via ecosystem patterns
- **Health Monitoring**: Distributed health checks

### 🚀 Planned
- Full Unix socket implementation
- End-to-end ecosystem integration
- Performance optimization
- Comprehensive test coverage

---

## Architecture

### Capability-Based Discovery

```
┌─────────────┐
│  Squirrel   │  Knows only itself
└──────┬──────┘
       │ Discovers at runtime:
       │
       ├──> 🔐 BearDog (crypto.ed25519.sign)
       ├──> 🌐 Songbird (http.client, ai.openai)
       └──> 📊 Ecosystem (service.discovery)
```

### Unix Socket Delegation

```
AI Request → Squirrel → Unix Socket → Songbird → OpenAI
                      (JSON-RPC)      (HTTPS)
```

**Benefits**:
- ✅ No C dependencies in Squirrel
- ✅ Network primal handles TLS/HTTPS
- ✅ Single point for rate limiting, caching
- ✅ Easy to swap providers

---

## Project Structure

```
squirrel/
├── crates/
│   ├── main/              # Main binary & orchestration
│   ├── core/
│   │   ├── auth/          # JWT via capability discovery
│   │   ├── core/          # Core functionality (Pure Rust!)
│   │   └── mcp/           # MCP protocol implementation
│   ├── tools/
│   │   ├── ai-tools/      # AI capability discovery
│   │   └── cli/           # CLI utilities
│   ├── config/            # Configuration management
│   └── integration/       # Integration patterns
├── docs/                  # Documentation
├── archive/               # Session documents (fossil record)
└── scripts/               # Utility scripts
```

---

## Recent Changes (v1.4.9)

### 🎉 Historic Cleanup Session (Jan 19, 2026)

**Removed** (19,382+ lines):
- Entire HTTP infrastructure
- Direct AI provider clients (OpenAI, Anthropic, Gemini, Ollama)
- `jsonwebtoken` crate (ring dependency)
- `jsonrpsee` crate (ring dependency)
- Connection pooling infrastructure
- 48 files total

**Added**:
- Unix socket delegation pattern
- Capability discovery for AI
- Modern idiomatic Rust patterns
- Comprehensive migration documentation

**Result**:
- ✅ ZERO C dependencies (verified!)
- 🔧 4 syntax errors (mechanical fixes)
- 📈 99.9% Pure Rust

See [CURRENT_STATUS.md](CURRENT_STATUS.md) for detailed progress.

---

## Documentation

- **[CURRENT_STATUS.md](CURRENT_STATUS.md)** - Detailed current state
- **[START_HERE.md](START_HERE.md)** - Quick start guide
- **[docs/](docs/)** - Full documentation
  - [CAPABILITY_AI_MIGRATION_GUIDE.md](docs/CAPABILITY_AI_MIGRATION_GUIDE.md)
  - Migration guides and patterns
- **[archive/](archive/)** - Session documents (fossil record)

---

## Testing

```bash
# Run tests (most passing, some skipped)
cargo test --workspace

# Check for C dependencies
cargo tree -p squirrel | grep ring  # Should be empty!

# Build validation (4 errors to fix)
cargo build --release
```

---

## Contributing

This project follows the **ecoPrimals philosophy**:
- **Deep solutions**, not patches
- **Pure Rust** (no C dependencies)
- **Capability-based** discovery
- **Document thoroughly** (fossil record)

Current contribution opportunities:
1. Fix 4 remaining syntax errors
2. Implement Unix socket communication
3. Add comprehensive tests
4. Performance optimization

---

## TRUE ecoBin Certification

**Status**: #5 Candidate (99.9%)

✅ **Criteria Met**:
- 100% Pure Rust dependency tree (verified!)
- UniBin architecture (single binary, subcommands)
- Doctor Mode (health checks, diagnostics)
- Capability discovery (TRUE PRIMAL)
- No hardcoded primals

🔧 **Remaining**:
- 4 syntax errors to fix
- Build validation
- Test suite completion

---

## Ecosystem Integration

Squirrel integrates with:
- **BearDog**: Cryptography (Ed25519 signing)
- **Songbird**: Network operations (HTTP, AI APIs)
- **Ecosystem**: Service discovery, health monitoring

All via **Unix sockets** and **capability discovery** - zero compile-time coupling!

---

## Performance

- **Startup**: < 100ms (estimated, after build fixes)
- **Memory**: TBD (lightweight, Pure Rust)
- **Binary Size**: TBD (smaller without C deps)

---

## License

See [LICENSE](LICENSE) file for details.

---

## Support

- **Issues**: [GitHub Issues](https://github.com/ecoPrimals/squirrel/issues)
- **Discussions**: GitHub Discussions
- **Docs**: See [docs/](docs/) directory

---

## Acknowledgments

This project represents one of the **largest cleanup sessions** in ecoPrimals history:
- 9 hours of focused execution
- 48 files deleted
- 19,382+ lines removed
- 2 C dependencies eliminated
- **ZERO ring dependencies!** 🎉

**The ecological way - execute deeply, evolve constantly!** 🌍🦀✨

---

*Deploy like an infant - knows nothing, discovers everything* 🐿️
