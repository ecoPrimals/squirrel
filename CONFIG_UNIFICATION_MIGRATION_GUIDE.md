# Config System Unification Migration Guide

**Date**: November 7, 2025  
**Status**: Phase 1 Complete - Foundation Ready  
**Next Steps**: Begin timeout migration

---

## 📋 What's Been Built

### ✅ Phase 1 Complete: Foundation (4 hours)

We've created the **unified configuration system** that will replace 2,498 hardcoded timeout values and consolidate 737 config structs:

#### New Modules Created

1. **`crates/config/src/unified/mod.rs`** - Main unified config module
2. **`crates/config/src/unified/timeouts.rs`** - TimeoutConfig with full environment awareness
3. **`crates/config/src/unified/types.rs`** - SquirrelUnifiedConfig hierarchy
4. **`crates/config/src/unified/loader.rs`** - ConfigLoader with precedence

#### Key Features

**TimeoutConfig**: Environment-aware timeout configuration
```rust
use squirrel_mcp_config::unified::TimeoutConfig;

let config = TimeoutConfig::from_env();
let timeout = config.connection_timeout(); // Returns Duration
```

**Environment Variables Supported**:
- `SQUIRREL_CONNECTION_TIMEOUT_SECS` (default: 30)
- `SQUIRREL_REQUEST_TIMEOUT_SECS` (default: 60)
- `SQUIRREL_HEALTH_CHECK_TIMEOUT_SECS` (default: 5)
- `SQUIRREL_OPERATION_TIMEOUT_SECS` (default: 10)
- `SQUIRREL_DATABASE_TIMEOUT_SECS` (default: 30)
- `SQUIRREL_HEARTBEAT_INTERVAL_SECS` (default: 30)
- `SQUIRREL_DISCOVERY_TIMEOUT_SECS` (default: 10)
- `SQUIRREL_AI_INFERENCE_TIMEOUT_SECS` (default: 120)
- `SQUIRREL_PLUGIN_LOAD_TIMEOUT_SECS` (default: 15)
- `SQUIRREL_SESSION_TIMEOUT_SECS` (default: 3600)
- Custom timeouts: `SQUIRREL_CUSTOM_TIMEOUT_<NAME>_SECS`

**SquirrelUnifiedConfig**: Hierarchical configuration structure
```rust
use squirrel_mcp_config::unified::{ConfigLoader, SquirrelUnifiedConfig};

// Load with full precedence hierarchy
let config = ConfigLoader::load()?;

// Access configuration
let timeout = config.timeouts.connection_timeout();
let port = config.network.http_port;
```

**ConfigLoader**: Precedence hierarchy
1. Environment variables (highest)
2. Configuration file (squirrel.toml)
3. Platform-specific defaults
4. Secure fallback defaults (lowest)

---

## 🚀 Phase 2: Timeout Migration (Next Steps)

### Target: Replace 2,498 Hardcoded Timeouts

#### Migration Pattern

**BEFORE** (hardcoded):
```rust
// ❌ Hardcoded timeout scattered throughout codebase
use tokio::time::timeout;
use std::time::Duration;

timeout(Duration::from_secs(30), operation()).await?
```

**AFTER** (unified config):
```rust
// ✅ Environment-aware timeout from unified config
use squirrel_mcp_config::unified::ConfigLoader;

let config = ConfigLoader::load()?;
timeout(config.timeouts.connection_timeout(), operation()).await?
```

#### Step-by-Step Migration Process

##### 1. Identify Hardcoded Timeouts

**Common Patterns to Find**:
```bash
# Search for hardcoded Duration values
rg "Duration::from_secs\(" crates/
rg "Duration::from_millis\(" crates/
rg "timeout\(Duration" crates/
```

**Categories Found** (from analysis):
- Connection timeouts (~500 instances)
- Request timeouts (~400 instances)
- Health check timeouts (~300 instances)
- Operation timeouts (~600 instances)
- Database timeouts (~200 instances)
- Discovery timeouts (~150 instances)
- AI inference timeouts (~100 instances)
- Plugin timeouts (~100 instances)
- Other (~148 instances)

##### 2. Choose Appropriate Timeout

Map each hardcoded timeout to the appropriate unified config method:

| Hardcoded Value | Unified Config Method | Notes |
|-----------------|----------------------|-------|
| `Duration::from_secs(30)` for connections | `config.timeouts.connection_timeout()` | Connection establishment |
| `Duration::from_secs(60)` for requests | `config.timeouts.request_timeout()` | Request completion |
| `Duration::from_secs(5)` for health | `config.timeouts.health_check_timeout()` | Health checks |
| `Duration::from_secs(10)` generic | `config.timeouts.operation_timeout()` | Generic operations |
| `Duration::from_secs(120)` for AI | `config.timeouts.ai_inference_timeout()` | AI model inference |
| `Duration::from_secs(15)` for plugins | `config.timeouts.plugin_load_timeout()` | Plugin loading |
| Custom values | `config.timeouts.get_custom_timeout("name")` | Named operations |

##### 3. Update Code

**Example 1: Simple Timeout**
```rust
// BEFORE
async fn connect_to_service(addr: &str) -> Result<Connection> {
    tokio::time::timeout(
        Duration::from_secs(30),  // ❌ Hardcoded
        TcpStream::connect(addr)
    ).await??
}

// AFTER
async fn connect_to_service(addr: &str, config: &SquirrelUnifiedConfig) -> Result<Connection> {
    tokio::time::timeout(
        config.timeouts.connection_timeout(),  // ✅ Environment-aware
        TcpStream::connect(addr)
    ).await??
}
```

**Example 2: Health Check**
```rust
// BEFORE
async fn health_check(endpoint: &str) -> Result<HealthStatus> {
    let timeout_duration = Duration::from_secs(5);  // ❌ Hardcoded
    
    tokio::time::timeout(timeout_duration, check_health(endpoint))
        .await
        .map_err(|_| Error::Timeout)?
}

// AFTER
async fn health_check(endpoint: &str, config: &SquirrelUnifiedConfig) -> Result<HealthStatus> {
    tokio::time::timeout(
        config.timeouts.health_check_timeout(),  // ✅ Environment-aware
        check_health(endpoint)
    ).await.map_err(|_| Error::Timeout)?
}
```

**Example 3: Custom Timeout**
```rust
// BEFORE
async fn special_operation() -> Result<()> {
    tokio::time::timeout(
        Duration::from_secs(42),  // ❌ Custom hardcoded value
        do_special_thing()
    ).await??
}

// AFTER
async fn special_operation(config: &SquirrelUnifiedConfig) -> Result<()> {
    tokio::time::timeout(
        config.timeouts.get_custom_timeout("special_operation"),  // ✅ Configurable
        do_special_thing()
    ).await??
}

// Can be configured via environment:
// export SQUIRREL_CUSTOM_TIMEOUT_SPECIAL_OPERATION_SECS=42
```

##### 4. Handle Struct/State Patterns

Many timeouts are in struct fields. Update those too:

**BEFORE**:
```rust
pub struct ServiceClient {
    endpoint: String,
    timeout: Duration,  // ❌ Hardcoded in new()
}

impl ServiceClient {
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            timeout: Duration::from_secs(30),  // ❌ Hardcoded
        }
    }
}
```

**AFTER**:
```rust
pub struct ServiceClient {
    endpoint: String,
    config: Arc<SquirrelUnifiedConfig>,  // ✅ Store config reference
}

impl ServiceClient {
    pub fn new(endpoint: String, config: Arc<SquirrelUnifiedConfig>) -> Self {
        Self {
            endpoint,
            config,
        }
    }
    
    fn timeout(&self) -> Duration {
        self.config.timeouts.request_timeout()  // ✅ Dynamic
    }
}
```

---

## 📊 Migration Tracking

### Progress Checklist

#### Module-by-Module Migration

**Core MCP Modules** (highest priority - 800+ instances):
- [ ] `crates/core/mcp/src/transport/` (~200 instances)
- [ ] `crates/core/mcp/src/session/` (~150 instances)
- [ ] `crates/core/mcp/src/client/` (~150 instances)
- [ ] `crates/core/mcp/src/resilience/` (~100 instances)
- [ ] `crates/core/mcp/src/protocol/` (~100 instances)
- [ ] `crates/core/mcp/src/enhanced/` (~100 instances)

**Main Application** (~400 instances):
- [ ] `crates/main/src/ecosystem/` (~100 instances)
- [ ] `crates/main/src/universal_adapters/` (~80 instances)
- [ ] `crates/main/src/monitoring/` (~60 instances)
- [ ] `crates/main/src/storage_client/` (~40 instances)
- [ ] `crates/main/src/security_client/` (~40 instances)
- [ ] `crates/main/src/compute_client/` (~40 instances)
- [ ] Other main modules (~40 instances)

**AI Tools** (~300 instances):
- [ ] `crates/tools/ai-tools/src/router/` (~80 instances)
- [ ] `crates/tools/ai-tools/src/local/` (~60 instances)
- [ ] `crates/tools/ai-tools/src/providers/` (~60 instances)
- [ ] `crates/tools/ai-tools/src/common/` (~50 instances)
- [ ] Other AI modules (~50 instances)

**Integration Crates** (~200 instances):
- [ ] `crates/integration/api-clients/` (~80 instances)
- [ ] `crates/integration/web/` (~40 instances)
- [ ] `crates/integration/toadstool/` (~40 instances)
- [ ] Other integration (~40 instances)

**Plugins & Services** (~300 instances):
- [ ] `crates/core/plugins/` (~100 instances)
- [ ] `crates/services/` (~80 instances)
- [ ] `crates/core/context/` (~60 instances)
- [ ] `crates/core/auth/` (~60 instances)

**Remaining** (~498 instances):
- [ ] `crates/tools/cli/` (~100 instances)
- [ ] `crates/sdk/` (~80 instances)
- [ ] `crates/universal-patterns/` (~60 instances)
- [ ] Other crates (~258 instances)

---

## 🧪 Testing Strategy

### Unit Tests

Test that config loading works:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_timeout_config_from_env() {
        std::env::set_var("SQUIRREL_CONNECTION_TIMEOUT_SECS", "45");
        
        let config = TimeoutConfig::from_env();
        assert_eq!(config.connection_timeout(), Duration::from_secs(45));
    }
    
    #[test]
    fn test_config_loader() {
        let loaded = ConfigLoader::load().unwrap();
        assert!(loaded.sources().len() > 0);
    }
}
```

### Integration Tests

Test that timeouts are actually used:
```rust
#[tokio::test]
async fn test_connection_uses_config_timeout() {
    let config = ConfigLoader::load().unwrap().into_config();
    
    // This should timeout according to config, not hardcoded value
    let result = connect_with_timeout(&config).await;
    
    // Verify behavior matches config
}
```

### Manual Testing

```bash
# Test with custom timeout
export SQUIRREL_CONNECTION_TIMEOUT_SECS=5
cargo run

# Test with custom timeout for specific operation
export SQUIRREL_CUSTOM_TIMEOUT_MY_OPERATION_SECS=100
cargo run

# Verify timeouts are loaded
cargo test test_timeout_config
```

---

## 🎯 Success Criteria

### Migration Complete When:

1. **All Hardcoded Timeouts Replaced**
   - [x] Foundation created
   - [ ] 2,498 instances migrated → 0 hardcoded timeouts
   - [ ] All modules use unified config

2. **Environment Awareness**
   - [x] All timeout defaults check environment variables
   - [ ] Custom timeouts supported
   - [ ] Documentation updated

3. **Testing**
   - [ ] Unit tests passing
   - [ ] Integration tests passing
   - [ ] Manual testing verified

4. **Documentation**
   - [x] Migration guide created
   - [ ] API documentation updated
   - [ ] Environment variable list published

---

## 📞 Getting Help

### Common Issues

**Q: How do I pass config to deeply nested functions?**
A: Store `Arc<SquirrelUnifiedConfig>` in your main structs and pass it down. Use `Arc::clone()` for cheap cloning.

**Q: What if I need a timeout that isn't in TimeoutConfig?**
A: Use `config.timeouts.get_custom_timeout("your_operation_name")` and set via `SQUIRREL_CUSTOM_TIMEOUT_YOUR_OPERATION_NAME_SECS`.

**Q: How do I test with different timeout values?**
A: Set environment variables in your test:
```rust
#[test]
fn test_with_custom_timeout() {
    std::env::set_var("SQUIRREL_CONNECTION_TIMEOUT_SECS", "10");
    let config = TimeoutConfig::from_env();
    // test with config
}
```

**Q: Can I still override timeouts per-operation?**
A: Yes! The config provides defaults, but you can always use `Duration::from_secs(X)` for specific cases. Just document why it's special.

---

## 🚀 Next Steps

1. **Start Migrating** (this week)
   - Begin with `crates/core/mcp/src/transport/`
   - Update ~200 instances
   - Verify tests pass

2. **Continue Systematically** (weeks 2-4)
   - Work through each module
   - Update tests as you go
   - Track progress in this document

3. **Verify Complete** (week 5)
   - Search for remaining hardcoded timeouts
   - Ensure all use unified config
   - Update documentation

**Let's eliminate all 2,498 hardcoded timeouts and achieve 100% environment-driven configuration!** 🎯

---

**Created**: November 7, 2025  
**Status**: Phase 1 Complete, Ready for Migration  
**Next Update**: After first module migration complete

