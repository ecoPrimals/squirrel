# 🎯 Hardcoding Migration Guide - TRUE PRIMAL Evolution

**Date**: January 30, 2026 (Evening)  
**Purpose**: Migrate from hardcoded endpoints to capability-based runtime discovery  
**Philosophy**: TRUE PRIMAL - Self-knowledge only, discover everything at runtime

---

## 🌟 **EXECUTIVE SUMMARY**

This guide shows how to evolve hardcoded endpoints to capability-based discovery, aligning with TRUE PRIMAL principles:
- ✅ Self-knowledge only (no hardcoded primal names/endpoints)
- ✅ Runtime discovery (find what you need when you need it)
- ✅ Environment-driven configuration (explicit overrides)
- ✅ Graceful fallbacks (sensible defaults with warnings)

---

## 🎯 **NEW INFRASTRUCTURE**

### **1. EndpointResolver** (Comprehensive Solution)

**Location**: `crates/universal-patterns/src/config/endpoint_resolver.rs`

**Features**:
- ✅ Multi-protocol support (Unix sockets, HTTP, WebSocket)
- ✅ Intelligent strategy selection (PreferSocket, PreferNetwork, SocketOnly, NetworkOnly)
- ✅ Environment variable overrides
- ✅ Endpoint caching for performance
- ✅ Integrated with biomeOS socket standardization

**Usage**:
```rust
use universal_patterns::config::{EndpointResolver, Endpoint, ResolutionStrategy};

// Create resolver (prefers Unix sockets for local primals)
let resolver = EndpointResolver::new();

// Resolve endpoint for a primal
let songbird_endpoint = resolver.resolve("songbird").await?;

match songbird_endpoint {
    Endpoint::UnixSocket(path) => {
        // Best case: Local primal via Unix socket (fast, secure)
        let stream = UnixStream::connect(path).await?;
    }
    Endpoint::Http(url) => {
        // Fallback: Remote primal or legacy HTTP
        let response = client.get(url).send().await?;
    }
    Endpoint::WebSocket(url) => {
        // WebSocket connection
        let ws = connect_async(url).await?;
    }
}
```

### **2. Socket Discovery Helpers** (Convenience)

**Location**: `crates/main/src/capabilities/discovery.rs`

**New Functions**:
```rust
// Discover specific primals by name (with Unix socket support)
let songbird = discover_songbird().await?;
let beardog = discover_beardog().await?;
let toadstool = discover_toadstool().await?;
let nestgate = discover_nestgate().await?;

// Use the discovered socket
let stream = UnixStream::connect(&songbird.socket).await?;
```

### **3. Port Resolution** (Backward Compatibility)

**Location**: `crates/universal-patterns/src/config/port_resolver.rs`  
**When to Use**: Legacy HTTP/network communication

```rust
use universal_patterns::config::PortResolver;

let resolver = PortResolver::new();
let port = resolver.resolve_port("http").await?;
let endpoint = resolver.resolve_endpoint("http").await?;
```

---

## 📋 **MIGRATION PATTERNS**

### **Pattern 1: Hardcoded Endpoint → EndpointResolver**

**Before** (Hardcoded):
```rust
// ❌ BAD: Hardcoded endpoint
let songbird_url = "http://localhost:8443";
let response = client.get(songbird_url).send().await?;
```

**After** (Discovery-based):
```rust
// ✅ GOOD: Runtime discovery
let resolver = EndpointResolver::new();
let endpoint = resolver.resolve("songbird").await?;

match endpoint {
    Endpoint::UnixSocket(path) => {
        // Prefer Unix socket (local primal)
        let stream = UnixStream::connect(path).await?;
        // Use JSON-RPC over Unix socket
    }
    Endpoint::Http(url) => {
        // Fallback to HTTP (remote primal)
        let response = client.get(&url).send().await?;
    }
    _ => { /* Handle other types */ }
}
```

**Environment Configuration**:
```bash
# Explicit endpoint (highest priority)
export SONGBIRD_ENDPOINT=http://remote-host:8443

# Or Unix socket
export SONGBIRD_SOCKET=/run/user/1000/biomeos/songbird.sock

# Or let it auto-discover (preferred for NUCLEUS deployment)
# No env vars needed - will find at /run/user/<uid>/biomeos/songbird.sock
```

---

### **Pattern 2: Hardcoded Port → Port Discovery**

**Before** (Hardcoded):
```rust
// ❌ BAD: Hardcoded port
let port = 8080;
let url = format!("http://localhost:{}", port);
```

**After** (Discovery-based):
```rust
// ✅ GOOD: Runtime discovery
use universal_constants::network::get_service_port;

let port = get_service_port("websocket");
let url = format!("http://localhost:{}", port);
```

**Environment Configuration**:
```bash
# Override port via environment
export WEBSOCKET_PORT=9090

# Otherwise uses sensible defaults (8080 for websocket)
```

---

### **Pattern 3: Multiple Endpoints → Unified Resolver**

**Before** (Multiple hardcoded):
```rust
// ❌ BAD: Multiple hardcoded endpoints
const SONGBIRD_URL: &str = "http://localhost:8443";
const BEARDOG_URL: &str = "http://localhost:8444";
const TOADSTOOL_URL: &str = "http://localhost:8445";
const NESTGATE_URL: &str = "http://localhost:8446";
```

**After** (Unified discovery):
```rust
// ✅ GOOD: Single resolver, multiple primals
let resolver = Arc::new(EndpointResolver::new());

// Discover all primals you need
let songbird = resolver.resolve("songbird").await?;
let beardog = resolver.resolve("beardog").await?;
let toadstool = resolver.resolve("toadstool").await?;
let nestgate = resolver.resolve("nestgate").await?;

// Use them as needed (prefer Unix sockets automatically!)
```

---

### **Pattern 4: Test Hardcoding → Environment-Based**

**Before** (Hardcoded test):
```rust
#[tokio::test]
async fn test_primal_connection() {
    // ❌ BAD: Hardcoded in test
    let url = "http://localhost:8080";
    let client = connect(url).await.unwrap();
}
```

**After** (Environment-based test):
```rust
#[tokio::test]
async fn test_primal_connection() {
    // ✅ GOOD: Use resolver in test
    let resolver = EndpointResolver::new().warn_on_fallback(false);
    let endpoint = resolver.resolve("test_service").await.unwrap();
    
    // Test can configure via env var if needed
    // export TEST_SERVICE_ENDPOINT=http://test-host:9999
}
```

---

## 🔍 **MIGRATION CHECKLIST**

### **Phase 1: Add Infrastructure** ✅ COMPLETE

- ✅ Create `EndpointResolver` with multi-protocol support
- ✅ Update socket discovery to use `/biomeos/` directory
- ✅ Add standard primal discovery helpers
- ✅ Wire into universal-patterns module
- ✅ Comprehensive testing (207/207 tests passing)

### **Phase 2: Update High-Priority Usages** (In Progress)

**Priority Areas**:
1. Security coordinator (BearDog integration)
2. Ecosystem manager (primal discovery)
3. MCP transport (WebSocket/TCP connections)
4. Test fixtures (reduce hardcoding in tests)

**Migration Steps**:
1. Identify hardcoded endpoints
2. Replace with `EndpointResolver` or discovery helpers
3. Add environment variable documentation
4. Update tests to be environment-aware
5. Verify functionality

### **Phase 3: Documentation & Deprecation** (Pending)

- [ ] Mark hardcoded constants as deprecated
- [ ] Add migration examples
- [ ] Update README with configuration guide
- [ ] Document environment variables

---

## 🎯 **CONFIGURATION REFERENCE**

### **Endpoint Configuration (Priority Order)**

#### **1. Explicit Endpoint (Highest Priority)**
```bash
export SONGBIRD_ENDPOINT=http://remote-server:8443
export BEARDOG_ENDPOINT=unix:///custom/beardog.sock
export TOADSTOOL_ENDPOINT=ws://toadstool-cluster:8445
```

#### **2. Socket Path (Local Primals)**
```bash
export SONGBIRD_SOCKET=/run/user/1000/biomeos/songbird.sock
export BEARDOG_SOCKET=/run/user/1000/biomeos/beardog.sock
```

#### **3. Port Override (Network Fallback)**
```bash
export SONGBIRD_PORT=9443
export BEARDOG_PORT=9444
```

#### **4. Auto-Discovery (NUCLEUS Standard)**
```bash
# No configuration needed!
# Automatically finds sockets at /run/user/<uid>/biomeos/<primal>.sock
```

---

## 🚀 **RESOLUTION STRATEGIES**

### **PreferSocket** (Default - Recommended)

```rust
let resolver = EndpointResolver::new(); // Uses PreferSocket by default

// Resolution order:
// 1. SONGBIRD_ENDPOINT env var
// 2. SONGBIRD_SOCKET env var
// 3. /run/user/<uid>/biomeos/songbird.sock (standard path)
// 4. Network discovery (service mesh)
// 5. HTTP fallback (http://localhost:8443)
```

**Use When**: Deploying in NUCLEUS environment (local primals)

### **PreferNetwork** (Remote Focus)

```rust
let resolver = EndpointResolver::with_strategy(ResolutionStrategy::PreferNetwork);

// Resolution order:
// 1. SONGBIRD_ENDPOINT env var
// 2. Network discovery (service mesh)
// 3. Unix socket fallback
// 4. HTTP fallback
```

**Use When**: Connecting to remote primals or distributed deployment

### **SocketOnly** (Strict Local)

```rust
let resolver = EndpointResolver::with_strategy(ResolutionStrategy::SocketOnly);

// Only uses Unix sockets - fails if not available
// Use for: Strict NUCLEUS deployment, testing socket-only scenarios
```

### **NetworkOnly** (Strict Remote)

```rust
let resolver = EndpointResolver::with_strategy(ResolutionStrategy::NetworkOnly);

// Only uses network - fails if not available
// Use for: Remote-only deployment, cross-machine communication
```

---

## 📊 **EXAMPLES**

### **Example 1: Security Service (BearDog)**

**Before**:
```rust
// Hardcoded BearDog endpoint
let beardog_url = "http://localhost:8443";
let security_config = SecurityProviderConfig {
    service_endpoint: beardog_url.to_string(),
    ..Default::default()
};
```

**After**:
```rust
// Discovery-based BearDog endpoint
let resolver = EndpointResolver::new();
let beardog_endpoint = resolver.resolve("beardog").await?;

// Prefer using Unix socket discovery helper
use squirrel::capabilities::discovery::discover_beardog;

let beardog = discover_beardog().await?;
let security_config = SecurityProviderConfig {
    service_socket: Some(beardog.socket), // Unix socket!
    service_endpoint: None, // No HTTP needed for local primal
    ..Default::default()
};
```

**Environment**:
```bash
# For custom BearDog location
export BEARDOG_SOCKET=/custom/path/beardog.sock

# Or for remote BearDog
export BEARDOG_ENDPOINT=https://beardog-cluster:8443
```

---

### **Example 2: WebSocket Server**

**Before**:
```rust
// Hardcoded WebSocket port
let ws_port = 8080;
let listener = TcpListener::bind(("0.0.0.0", ws_port)).await?;
```

**After**:
```rust
// Discovery-based WebSocket port
use universal_constants::network::get_service_port;

let ws_port = get_service_port("websocket");
let bind_addr = get_bind_address();
let listener = TcpListener::bind((bind_addr.as_str(), ws_port)).await?;

println!("WebSocket server listening on {}:{}", bind_addr, ws_port);
```

**Environment**:
```bash
# Override WebSocket port
export WEBSOCKET_PORT=9090

# Override bind address
export BIND_ADDRESS=0.0.0.0
```

---

### **Example 3: Test Fixtures**

**Before**:
```rust
#[tokio::test]
async fn test_api_client() {
    // Hardcoded test endpoint
    let client = ApiClient::new("http://localhost:8080");
    // ...
}
```

**After**:
```rust
#[tokio::test]
async fn test_api_client() {
    // Environment-based test endpoint
    let resolver = EndpointResolver::new().warn_on_fallback(false);
    let endpoint = resolver.resolve("api").await.unwrap();
    
    let client = match endpoint {
        Endpoint::Http(url) => ApiClient::new(&url),
        _ => panic!("Expected HTTP endpoint for API client"),
    };
    // ...
}

// Or use a test helper:
#[tokio::test]
async fn test_api_client_with_helper() {
    // Test can be configured via: TEST_API_ENDPOINT=http://test:8080
    let endpoint = std::env::var("TEST_API_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    let client = ApiClient::new(&endpoint);
}
```

---

## 🎓 **BEST PRACTICES**

### **1. Prefer Unix Sockets for Local Primals**

```rust
// ✅ BEST: Use socket discovery helper
let songbird = discover_songbird().await?;
let stream = UnixStream::connect(&songbird.socket).await?;

// ⚠️ OK: Use EndpointResolver
let endpoint = resolver.resolve("songbird").await?;

// ❌ AVOID: Hardcoded endpoint
let url = "http://localhost:8443";
```

**Why**:
- Faster (no network stack)
- More secure (filesystem permissions)
- NUCLEUS-compliant
- Zero configuration needed

### **2. Use Environment Variables for External Services**

```rust
// External database (not a primal)
let db_url = std::env::var("DATABASE_URL")
    .expect("DATABASE_URL must be set for external database");

// External API
let api_endpoint = std::env::var("EXTERNAL_API_ENDPOINT")
    .unwrap_or_else(|_| {
        warn!("EXTERNAL_API_ENDPOINT not set, using fallback");
        "http://api.example.com".to_string()
    });
```

### **3. Warn on Fallbacks in Production**

```rust
let resolver = EndpointResolver::new(); // warn_on_fallback = true by default

// This will log warnings if using fallback defaults
let endpoint = resolver.resolve("service").await?;

// In tests, disable warnings:
let test_resolver = EndpointResolver::new().warn_on_fallback(false);
```

### **4. Document Environment Variables**

```rust
/// Connect to Songbird primal
///
/// ## Environment Variables
///
/// - `SONGBIRD_ENDPOINT`: Explicit endpoint (e.g., http://songbird-server:8443)
/// - `SONGBIRD_SOCKET`: Unix socket path (e.g., /run/user/1000/biomeos/songbird.sock)
/// - `SONGBIRD_PORT`: Network port (e.g., 9443)
///
/// ## Auto-Discovery
///
/// If no environment variables set, will auto-discover at:
/// `/run/user/<uid>/biomeos/songbird.sock` (NUCLEUS standard)
pub async fn connect_to_songbird() -> Result<SongbirdClient> {
    let resolver = EndpointResolver::new();
    let endpoint = resolver.resolve("songbird").await?;
    // ...
}
```

---

## 🔧 **MIGRATION STEPS**

### **Step 1: Identify Hardcoded Endpoints**

```bash
# Find hardcoded ports
rg ':80\d\d|:90\d\d|:30\d\d' --glob '**/*.rs'

# Find hardcoded localhost
rg 'localhost:\d+|127\.0\.0\.1:\d+' --glob '**/*.rs'

# Find hardcoded URLs
rg 'http://.*:\d+|https://.*:\d+' --glob '**/*.rs'
```

### **Step 2: Categorize by Priority**

**High Priority** (Production code):
- Security coordinator (BearDog)
- Ecosystem manager (primal registry)
- MCP transport (WebSocket/TCP)
- API clients (inter-primal communication)

**Medium Priority** (Configuration):
- Default configs
- Example code
- Integration helpers

**Low Priority** (Tests):
- Test fixtures
- Integration tests
- Example demos

### **Step 3: Migrate High-Priority Code**

For each hardcoded endpoint:

1. **Determine if it's a primal or external service**
   - Primal → Use `EndpointResolver` or discovery helpers
   - External → Use environment variable

2. **Choose resolution strategy**
   - Local primal → `PreferSocket` (default)
   - Remote primal → `PreferNetwork`
   - Testing → Configuration via env vars

3. **Update code**
   - Replace hardcoded value with resolver
   - Add error handling
   - Document environment variables

4. **Test**
   - Test with Unix socket
   - Test with HTTP fallback
   - Test with env var override

### **Step 4: Update Tests**

```rust
// Before: Hardcoded
assert_eq!(endpoint, "http://localhost:8080");

// After: Flexible
assert!(endpoint.starts_with("http://localhost:"));
// or
assert!(!endpoint.is_empty());
// or
std::env::set_var("TEST_ENDPOINT", "http://localhost:8080");
let endpoint = resolver.resolve("test").await?;
assert_eq!(endpoint.as_str(), "http://localhost:8080");
std::env::remove_var("TEST_ENDPOINT");
```

### **Step 5: Document & Deprecate**

```rust
/// Deprecated: Use EndpointResolver instead
#[deprecated(since = "4.0.0", note = "Use EndpointResolver::resolve() instead")]
pub const SONGBIRD_ENDPOINT: &str = "http://localhost:8443";
```

---

## 📊 **PROGRESS TRACKING**

### **Infrastructure** ✅ COMPLETE

- ✅ `EndpointResolver` created (multi-protocol support)
- ✅ Socket standardization complete (`/biomeos/` directory)
- ✅ Standard primal discovery helpers (songbird, beardog, toadstool, nestgate)
- ✅ Port resolution with environment override
- ✅ 207/207 tests passing

### **Migration Status** (469 instances found)

| Category | Total | Migrated | Remaining |
|----------|-------|----------|-----------|
| Production Code | ~50 | 0 | 50 |
| Configuration | ~80 | 0 | 80 |
| Tests | ~300+ | 0 | 300+ |
| Examples | ~30 | 0 | 30 |

**Target**: Migrate 50-100 high-priority instances

---

## 🎯 **RECOMMENDED MIGRATION ORDER**

### **1. Security Coordinator** (CRITICAL)

**File**: `crates/main/src/security/beardog_coordinator.rs`

```rust
// Current: hardcoded "http://localhost:8443"
// Target: discover_beardog() helper
```

### **2. Ecosystem Manager** (HIGH)

**File**: `crates/main/src/ecosystem/mod.rs`

```rust
// Current: hardcoded service mesh endpoint
// Target: EndpointResolver for service mesh
```

### **3. MCP Transport** (HIGH)

**File**: `crates/core/mcp/src/transport/websocket/mod.rs`

```rust
// Current: hardcoded WebSocket ports
// Target: get_service_port("websocket")
```

### **4. Test Fixtures** (MEDIUM)

**Files**: Various `*_tests.rs`

```rust
// Current: hardcoded test endpoints
// Target: Environment-based or resolver-based
```

---

## 🎉 **BENEFITS**

### **TRUE PRIMAL Alignment**

- ✅ No hardcoded primal knowledge
- ✅ Runtime discovery
- ✅ Environment-driven
- ✅ Capability-based

### **Production Flexibility**

- ✅ Easy deployment configuration
- ✅ Support for distributed systems
- ✅ Service mesh integration ready
- ✅ Graceful fallbacks

### **Developer Experience**

- ✅ Clear migration path
- ✅ Backward compatible (fallbacks)
- ✅ Comprehensive documentation
- ✅ Test-friendly

### **Security**

- ✅ Unix sockets (filesystem permissions)
- ✅ No exposed ports for local primals
- ✅ Environment-based secrets
- ✅ Audit trail (logs when using fallbacks)

---

## 📚 **RELATED DOCUMENTATION**

- `crates/universal-patterns/src/config/endpoint_resolver.rs` - EndpointResolver implementation
- `crates/main/src/capabilities/discovery.rs` - Standard primal discovery
- `crates/universal-constants/src/network.rs` - Port discovery functions
- `SOCKET_STANDARDIZATION_COMPLETE_JAN_30_2026.md` - Socket standardization details

---

## ✅ **NEXT STEPS**

### **Immediate**

1. Migrate security coordinator (BearDog) - HIGH PRIORITY
2. Migrate ecosystem manager - HIGH PRIORITY
3. Update 10-20 test fixtures - MEDIUM PRIORITY

### **Short-Term**

4. Migrate MCP transport - MEDIUM PRIORITY
5. Update configuration examples
6. Document migration for downstream teams

### **Long-Term**

7. Deprecate hardcoded constants
8. Complete remaining test migrations
9. Service mesh integration
10. Full zero-hardcoding compliance

---

**Migration Guide Created**: January 30, 2026 (Evening)  
**Infrastructure**: ✅ Complete  
**Status**: Ready for migration execution

🎯 **TRUE PRIMAL Evolution: In Progress!** 🦀✨
