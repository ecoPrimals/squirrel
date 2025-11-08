# Timeout Migration Examples - MCP Transport Layer

**Date**: November 7, 2025  
**Module**: `crates/core/mcp/src/transport/`  
**Instances Found**: 11 files with hardcoded timeouts

---

## 📊 Timeout Instances Found

### Files with Hardcoded Timeouts
```
✓ memory/mod.rs           - Duration::from_secs(5) for receive
✓ tcp/connection.rs       - Duration::from_secs(30) in PortConfig
✓ tcp/mod.rs              - connection_timeout: 30 secs in config struct
✓ Tests (7 files)         - Multiple test timeouts
```

---

## 🎯 Migration Pattern

### Pattern 1: Simple Timeout in Function

**BEFORE** (memory/mod.rs:367):
```rust
async fn receive_message(&self) -> Result<MCPMessage> {
    let mut rx_guard = self.incoming_channel.lock().await;
    let receive_future = rx_guard.recv();
    
    // ❌ Hardcoded timeout
    match tokio::time::timeout(std::time::Duration::from_secs(5), receive_future).await {
        Ok(Some(message)) => Ok(message),
        Ok(None) => Err(/* channel closed */),
        Err(_) => Err(/* timeout */),
    }
}
```

**AFTER** (with unified config):
```rust
async fn receive_message(&self) -> Result<MCPMessage> {
    let mut rx_guard = self.incoming_channel.lock().await;
    let receive_future = rx_guard.recv();
    
    // ✅ Config-driven timeout
    let timeout = self.config.timeouts.operation_timeout();
    match tokio::time::timeout(timeout, receive_future).await {
        Ok(Some(message)) => Ok(message),
        Ok(None) => Err(/* channel closed */),
        Err(_) => Err(/* timeout */),
    }
}
```

**Changes Required**:
1. Add `config: Arc<SquirrelUnifiedConfig>` to MemoryTransport struct
2. Pass config to constructor
3. Use `self.config.timeouts.operation_timeout()`

---

### Pattern 2: Config Struct with Timeout Fields

**BEFORE** (tcp/connection.rs:16-46):
```rust
#[derive(Debug, Clone)]
pub struct PortConfig {
    pub port: u16,
    pub protocol: String,
    pub max_connections: u32,
    pub timeout: Duration,  // ❌ Hardcoded in Default impl
    pub keep_alive: bool,
    // ...
}

impl Default for PortConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            max_connections: 10,
            timeout: Duration::from_secs(30),  // ❌ Hardcoded
            keep_alive: true,
            // ...
        }
    }
}
```

**AFTER** (with unified config):
```rust
#[derive(Debug, Clone)]
pub struct PortConfig {
    pub port: u16,
    pub protocol: String,
    pub max_connections: u32,
    // Remove timeout field - get from unified config instead
    pub keep_alive: bool,
    // ...
}

impl PortConfig {
    // ✅ Constructor that takes unified config
    pub fn from_unified_config(config: &SquirrelUnifiedConfig) -> Self {
        Self {
            port: config.network.http_port,
            max_connections: config.network.max_connections,
            keep_alive: true,
            // Get timeout from config when needed via method
            // ...
        }
    }
    
    // ✅ Method to get timeout from config
    pub fn timeout(&self, config: &SquirrelUnifiedConfig) -> Duration {
        config.timeouts.connection_timeout()
    }
}

impl Default for PortConfig {
    fn default() -> Self {
        // ✅ Load unified config for defaults
        let config = ConfigLoader::load()
            .map(|c| c.into_config())
            .unwrap_or_default();
        Self::from_unified_config(&config)
    }
}
```

---

### Pattern 3: Config Struct Field (TcpTransportConfig)

**BEFORE** (tcp/mod.rs:46-124):
```rust
#[derive(Debug, Clone)]
pub struct TcpTransportConfig {
    pub remote_address: String,
    pub max_message_size: usize,
    pub connection_timeout: u64,  // ❌ In seconds, hardcoded default
    pub keep_alive_interval: Option<u64>,
    pub max_reconnect_attempts: u32,
    pub reconnect_delay_ms: u64,
    // ...
}

impl Default for TcpTransportConfig {
    fn default() -> Self {
        Self {
            remote_address: "localhost:8080".to_string(),
            max_message_size: 16 * 1024 * 1024,
            connection_timeout: 30,  // ❌ Hardcoded 30 seconds
            keep_alive_interval: Some(60),
            max_reconnect_attempts: 3,
            reconnect_delay_ms: 1000,  // ❌ Hardcoded 1 second
            // ...
        }
    }
}
```

**AFTER** (with unified config):
```rust
use squirrel_mcp_config::unified::SquirrelUnifiedConfig;

#[derive(Debug, Clone)]
pub struct TcpTransportConfig {
    pub remote_address: String,
    pub max_message_size: usize,
    // Remove timeout fields - get from unified config
    pub max_reconnect_attempts: u32,
    // Reference to unified config
    config: Arc<SquirrelUnifiedConfig>,
}

impl TcpTransportConfig {
    // ✅ Constructor with config
    pub fn new(config: Arc<SquirrelUnifiedConfig>) -> Self {
        Self {
            remote_address: "localhost:8080".to_string(),
            max_message_size: config.mcp.max_message_size,
            max_reconnect_attempts: 3,
            config,
        }
    }
    
    // ✅ Methods to get timeouts from config
    pub fn connection_timeout(&self) -> Duration {
        self.config.timeouts.connection_timeout()
    }
    
    pub fn keep_alive_interval(&self) -> Duration {
        self.config.timeouts.heartbeat_interval()
    }
    
    pub fn reconnect_delay(&self) -> Duration {
        self.config.timeouts.get_custom_timeout("tcp_reconnect")
    }
}

impl Default for TcpTransportConfig {
    fn default() -> Self {
        let config = Arc::new(ConfigLoader::load()
            .map(|c| c.into_config())
            .unwrap_or_default());
        Self::new(config)
    }
}
```

---

### Pattern 4: Test Timeouts

**BEFORE** (tests/transport_tests.rs):
```rust
#[tokio::test]
async fn test_connection_timeout() {
    let result = tokio::time::timeout(
        Duration::from_secs(5),  // ❌ Hardcoded test timeout
        establish_connection()
    ).await;
    
    assert!(result.is_ok());
}
```

**AFTER** (with unified config):
```rust
#[tokio::test]
async fn test_connection_timeout() {
    let config = ConfigLoader::load().unwrap().into_config();
    
    let result = tokio::time::timeout(
        config.timeouts.connection_timeout(),  // ✅ From config
        establish_connection(&config)
    ).await;
    
    assert!(result.is_ok());
}

// Or for tests that need custom timeouts:
#[tokio::test]
async fn test_with_custom_timeout() {
    std::env::set_var("SQUIRREL_CONNECTION_TIMEOUT_SECS", "2");
    let config = TimeoutConfig::from_env();
    
    let result = tokio::time::timeout(
        config.connection_timeout(),  // Uses test value
        establish_connection()
    ).await;
    
    assert!(result.is_err()); // Expect timeout with 2 seconds
}
```

---

## 🔄 Migration Steps for Each File

### Step 1: Add Config Import
```rust
use squirrel_mcp_config::unified::{SquirrelUnifiedConfig, ConfigLoader};
use std::sync::Arc;
```

### Step 2: Update Struct to Include Config
```rust
pub struct MemoryTransport {
    // ... existing fields ...
    config: Arc<SquirrelUnifiedConfig>,
}
```

### Step 3: Update Constructor
```rust
impl MemoryTransport {
    pub fn new(config: Arc<SquirrelUnifiedConfig>) -> Self {
        Self {
            // ... existing initialization ...
            config,
        }
    }
}
```

### Step 4: Replace Hardcoded Timeouts
```rust
// OLD: Duration::from_secs(5)
// NEW: self.config.timeouts.operation_timeout()

// OLD: Duration::from_secs(30)
// NEW: self.config.timeouts.connection_timeout()

// OLD: Duration::from_millis(1000)
// NEW: self.config.timeouts.get_custom_timeout("operation_name")
```

### Step 5: Update Call Sites
```rust
// OLD: let transport = MemoryTransport::new();
// NEW: 
let config = Arc::new(ConfigLoader::load()?.into_config());
let transport = MemoryTransport::new(config);
```

---

## 📝 Mapping Guide

### Timeout Duration → Config Method

| Hardcoded Value | Use Config Method | Notes |
|-----------------|-------------------|-------|
| `Duration::from_secs(5)` | `operation_timeout()` | Generic 5-10s operations |
| `Duration::from_secs(30)` | `connection_timeout()` | Connection establishment |
| `Duration::from_secs(60)` | `request_timeout()` | Request completion |
| `Duration::from_millis(500)` | `health_check_timeout()` | Quick health checks |
| `Duration::from_millis(1000)` | `get_custom_timeout("name")` | Custom operations |
| `Duration::from_secs(120)` | `ai_inference_timeout()` | AI operations |

---

## ✅ Checklist for Each File

When migrating a file:

- [ ] Add config imports
- [ ] Update struct to store `Arc<SquirrelUnifiedConfig>`
- [ ] Update constructor to accept config
- [ ] Replace all `Duration::from_secs()` with config methods
- [ ] Replace all `Duration::from_millis()` with config methods
- [ ] Update all call sites to pass config
- [ ] Update tests to use config
- [ ] Verify compilation
- [ ] Run tests
- [ ] Commit changes

---

## 🎯 Environment Variable Testing

After migration, test with environment variables:

```bash
# Test with custom connection timeout
export SQUIRREL_CONNECTION_TIMEOUT_SECS=45
cargo test --package squirrel-mcp-core

# Test with custom operation timeout
export SQUIRREL_OPERATION_TIMEOUT_SECS=15
cargo run

# Test with custom TCP reconnect delay
export SQUIRREL_CUSTOM_TIMEOUT_TCP_RECONNECT_SECS=2
cargo test tcp_reconnect
```

---

## 📊 Progress Tracking

### MCP Transport Module

- [ ] `memory/mod.rs` - 1 instance (receive timeout)
- [ ] `tcp/connection.rs` - 1 instance (PortConfig default)
- [ ] `tcp/mod.rs` - 3 instances (TcpTransportConfig)
- [ ] `websocket/mod.rs` - (need to check)
- [ ] `stdio/mod.rs` - (need to check)
- [ ] `tests/transport_tests.rs` - Multiple test timeouts
- [ ] `tests/integration_tests.rs` - Multiple test timeouts
- [ ] `tests/connection_tests.rs` - Multiple test timeouts
- [ ] `tests/frame_tests.rs` - Multiple test timeouts
- [ ] `memory/tests.rs` - Multiple test timeouts
- [ ] `memory/standalone_test.rs` - Multiple test timeouts

**Total**: ~15-20 timeout instances in transport module

---

## 🚀 Next Steps

1. **Start with `memory/mod.rs`** (simplest, 1 instance)
2. **Move to `tcp/connection.rs`** (PortConfig pattern)
3. **Then `tcp/mod.rs`** (TcpTransportConfig pattern)
4. **Update tests** (test timeout pattern)
5. **Verify all changes** compile and tests pass

---

**Created**: November 7, 2025  
**Status**: Ready for execution  
**Estimated Time**: 4-6 hours for transport module

