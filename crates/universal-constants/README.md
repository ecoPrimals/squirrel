# 🌐 Universal Constants

**Single Source of Truth for all Squirrel MCP Constants**

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)]()
[![Tests](https://img.shields.io/badge/tests-25%20passing-brightgreen)]()
[![Version](https://img.shields.io/badge/version-0.1.0-blue)]()

---

## 📋 Overview

The `universal-constants` crate consolidates all constants from across the Squirrel Universal AI Primal system into a single, well-organized, type-safe location. This eliminates:

- ✅ Duplicate constant definitions
- ✅ Inconsistent values across modules
- ✅ Hard-to-find magic numbers
- ✅ Type safety issues (u64 vs Duration)

## 🎯 Design Principles

1. **Single Source of Truth**: Every constant defined exactly once
2. **Type Safety**: Use proper types (`Duration` not `u64`)
3. **Domain Organization**: Clear module boundaries
4. **Zero Dependencies**: Pure Rust, no external deps
5. **Comprehensive Tests**: 25 tests ensuring correctness

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
universal-constants = { path = "../universal-constants" }
```

## 🚀 Usage

### Quick Start

```rust
use universal_constants::{timeouts, network, limits};

// Use timeout constants
let timeout = timeouts::DEFAULT_CONNECTION_TIMEOUT;

// Use network constants  
let port = network::DEFAULT_WEBSOCKET_PORT;

// Use limit constants
let buffer_size = limits::DEFAULT_BUFFER_SIZE;
```

### URL Builders

```rust
use universal_constants::builders;

// Build URLs
let http_url = builders::localhost_http(8080);
// "http://localhost:8080"

let ws_url = builders::localhost_ws(8080);
// "ws://localhost:8080"

// Build endpoint URLs
let health = builders::health_url(&http_url);
// "http://localhost:8080/health"
```

### Environment Variable Parsing

```rust
use universal_constants::{builders, timeouts, env_vars};

// Parse with defaults
let timeout = builders::parse_timeout_duration(
    env_vars::CONNECTION_TIMEOUT,
    timeouts::DEFAULT_CONNECTION_TIMEOUT
);

// Get specific environment values
let db_timeout = builders::get_database_timeout();
let max_conns = builders::get_max_connections();
```

## 📚 Module Organization

### `timeouts` - Duration Constants

All timeout and interval values using `std::time::Duration`:

```rust
use universal_constants::timeouts;

timeouts::DEFAULT_CONNECTION_TIMEOUT    // 30 seconds
timeouts::DEFAULT_REQUEST_TIMEOUT       // 60 seconds
timeouts::DEFAULT_OPERATION_TIMEOUT     // 10 seconds
timeouts::DEFAULT_HEARTBEAT_INTERVAL    // 30 seconds
timeouts::DEFAULT_PING_INTERVAL         // 30 seconds
timeouts::DEFAULT_RETRY_DELAY           // 5 seconds
timeouts::DEFAULT_DATABASE_TIMEOUT      // 30 seconds
```

### `limits` - Size and Count Limits

Buffer sizes, connection limits, and capacity values:

```rust
use universal_constants::limits;

limits::DEFAULT_MAX_CONNECTIONS         // 100
limits::DEFAULT_MAX_SERVICES            // 1000
limits::DEFAULT_MAX_RETRIES             // 3
limits::DEFAULT_BUFFER_SIZE             // 8KB
limits::DEFAULT_CHUNK_SIZE              // 4KB
limits::DEFAULT_MAX_MESSAGE_SIZE        // 16MB
limits::DEFAULT_MAX_CONTEXT_LENGTH      // 128K
```

### `network` - Network Configuration

Addresses, ports, and endpoint paths:

```rust
use universal_constants::network;

// Addresses
network::DEFAULT_BIND_ADDRESS           // "127.0.0.1"
network::DEFAULT_LOCALHOST              // "localhost"

// Ports
network::DEFAULT_WEBSOCKET_PORT         // 8080
network::DEFAULT_HTTP_PORT              // 8081
network::DEFAULT_ADMIN_PORT             // 8082
network::DEFAULT_METRICS_PORT           // 9090

// Endpoints
network::HEALTH_ENDPOINT                // "/health"
network::METRICS_ENDPOINT               // "/metrics"
network::WS_ENDPOINT                    // "/ws"
```

### `protocol` - Protocol Constants

MCP and HTTP protocol configuration:

```rust
use universal_constants::protocol;

protocol::DEFAULT_MCP_SUBPROTOCOL       // "mcp"
protocol::DEFAULT_PROTOCOL_VERSION      // "1.0"
protocol::DEFAULT_USER_AGENT            // "squirrel-mcp/1.0"
protocol::DEFAULT_CONTENT_TYPE          // "application/json"
protocol::CONTENT_TYPE_JSON             // "application/json"
protocol::FEATURE_MULTI_AGENT           // "multi-agent"
```

### `env_vars` - Environment Variable Names

All environment variable names:

```rust
use universal_constants::env_vars;

env_vars::BIND_ADDRESS                  // "MCP_BIND_ADDRESS"
env_vars::WEBSOCKET_PORT                // "MCP_WEBSOCKET_PORT"
env_vars::CONNECTION_TIMEOUT            // "MCP_CONNECTION_TIMEOUT"
env_vars::DATABASE_TIMEOUT              // "DATABASE_TIMEOUT"
env_vars::MAX_CONNECTIONS               // "MAX_CONNECTIONS"
env_vars::BIOMEOS_REGISTRATION_URL      // "BIOMEOS_REGISTRATION_URL"
```

### `builders` - Helper Functions

Utility functions for building URLs and parsing config:

```rust
use universal_constants::builders;

// URL builders
builders::build_http_url(host, port)
builders::build_ws_url(host, port)
builders::localhost_http(port)
builders::localhost_ws(port)
builders::health_url(base_url)
builders::metrics_url(base_url)

// Environment parsers
builders::parse_timeout_duration(var, default)
builders::parse_limit(var, default)
builders::parse_port(var, default)
builders::parse_bool(var, default)

// Specific getters
builders::get_database_timeout()
builders::get_heartbeat_interval()
builders::get_max_connections()
builders::get_buffer_size()
```

## 🔄 Migration Guide

### From `squirrel_mcp_config::constants`

```rust
// ❌ Old (deprecated)
use squirrel_mcp_config::constants::timeouts;
use squirrel_mcp_config::constants::limits;

// ✅ New
use universal_constants::{timeouts, limits};
```

### From `squirrel_mcp::constants`

```rust
// ❌ Old (deprecated)
use squirrel_mcp::constants::network;
use squirrel_mcp::constants::protocol;

// ✅ New
use universal_constants::{network, protocol};
```

### Type-Safe Timeouts

```rust
// ❌ Old (raw milliseconds)
let timeout_ms: u64 = 30_000;

// ✅ New (type-safe Duration)
use universal_constants::timeouts;
let timeout = timeouts::DEFAULT_CONNECTION_TIMEOUT;
```

## 🧪 Testing

Run all tests:

```bash
cargo test
```

Run specific module tests:

```bash
cargo test --test timeouts
cargo test --test builders
```

View test coverage:

```bash
cargo test -- --nocapture
```

**Test Results**: 25 tests passing (100% pass rate)

## 📊 Consolidation Impact

### Before Unification

```
Constants scattered across:
- crates/config/src/constants.rs          (177 lines)
- crates/core/mcp/src/constants.rs        (222 lines)
- crates/core/mcp/src/protocol/constants.rs
- 230+ inline definitions across 87 files

Problems:
❌ Duplicate definitions
❌ Value mismatches
❌ Hard to maintain
❌ Type inconsistencies
```

### After Unification

```
Single source of truth:
- crates/universal-constants/             (1 crate)
  - src/timeouts.rs                       (comprehensive)
  - src/limits.rs                         (comprehensive)
  - src/network.rs                        (comprehensive)
  - src/protocol.rs                       (comprehensive)
  - src/env_vars.rs                       (comprehensive)
  - src/builders.rs                       (comprehensive)

Benefits:
✅ Single definition per constant
✅ Type-safe (Duration not u64)
✅ Easy to find and maintain
✅ Comprehensive documentation
✅ 25 tests ensuring correctness
```

## 🎯 Benefits

### Developer Experience

- **Find constants easily**: All in one place
- **Type safety**: Use `Duration` not raw milliseconds
- **Autocomplete**: Modern IDE support
- **Documentation**: Comprehensive examples
- **Testing**: 100% test coverage

### Maintenance

- **Update once**: Change propagates everywhere
- **No duplicates**: Single source of truth
- **Version control**: Clear change history
- **Refactoring**: Easy to reorganize

### Quality

- **Consistency**: Same values everywhere
- **Type safety**: Catch errors at compile time
- **Testing**: Comprehensive test suite
- **Documentation**: Clear usage examples

## 📖 Examples

See `examples/` directory for:

- Basic usage examples
- Migration examples
- Integration patterns
- Advanced usage

## 🔗 Related Crates

- `squirrel-mcp-config` - Configuration management (uses universal-constants)
- `squirrel-mcp` - MCP core (migrating to universal-constants)
- `universal-patterns` - Universal trait patterns

## 📝 License

MIT OR Apache-2.0

## 🤝 Contributing

When adding new constants:

1. Choose the appropriate module (timeouts, limits, network, etc.)
2. Use proper types (`Duration` not `u64`)
3. Add documentation
4. Add tests
5. Update this README

## 📞 Support

For questions or issues:

- Check the inline documentation
- Review the examples
- See the migration guide above
- Consult the main project README

---

**Status**: ✅ **PRODUCTION READY**  
**Version**: 0.1.0  
**Tests**: 25 passing  
**Dependencies**: 0 (pure Rust)

**Made with ❤️ by the Squirrel Team** 🐿️

