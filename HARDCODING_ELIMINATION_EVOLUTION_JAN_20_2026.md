# Hardcoding Elimination Evolution - TRUE PRIMAL Infant Pattern
## January 20, 2026

**Goal**: Deploy with ZERO knowledge, discover everything like an infant  
**Pattern**: Each primal knows only itself, discovers all others via capabilities  
**Status**: Comprehensive audit complete, evolution plan ready  

---

## 🔍 Audit Results - Hardcoding Found

### **CRITICAL: Primal Name Hardcoding**
```
BearDog/Songbird:   1,428 matches across 170 files  🔴 CRITICAL
ToadStool/NestGate:   (included above)              🔴 CRITICAL  
Total Impact:       ~2,000+ primal references        🔴 CRITICAL
```

### **HIGH: Vendor/Service Hardcoding**
```
Kubernetes/Consul:    114 matches across 22 files   🟡 HIGH
Etcd/Eureka:          (included above)              🟡 HIGH
Service Registries:   Hardcoded vendor names        🟡 HIGH
```

### **MEDIUM: Infrastructure Hardcoding**
```
Port Numbers:         460 matches across 116 files  🟠 MEDIUM
Socket Paths:          23 matches across 9 files    🟠 MEDIUM  
IP Addresses:         localhost, 127.0.0.1          🟠 MEDIUM
```

---

## 🎯 TRUE PRIMAL Infant Pattern

### **The Vision**

```rust
// ❌ WRONG: Hardcoded primal knowledge
let beardog = BearDogClient::new("/tmp/beardog.sock")?;
let signature = beardog.sign(data).await?;

// ✅ RIGHT: Capability discovery (infant pattern)
let crypto = discover_capability("crypto.signing").await?;
let signature = crypto.call("sign", data).await?;
```

### **Key Principles**

1. **Self-Knowledge Only**: Squirrel knows "I am Squirrel"
2. **Zero Primal Knowledge**: Doesn't know BearDog, Songbird, etc. exist
3. **Capability Discovery**: Discovers "crypto.signing" provider at runtime
4. **Vendor Agnostic**: Doesn't know Kubernetes, Consul, etc.
5. **Infrastructure Discovery**: No hardcoded ports, IPs, paths

---

## 📋 Evolution Plan

### **Phase 1: Primal Name Elimination** (CRITICAL - 1,428 instances)

#### **1.1: Deprecate EcosystemPrimalType Enum**

**File**: `crates/main/src/ecosystem/types.rs`  
**Current**:
```rust
pub enum EcosystemPrimalType {
    ToadStool,
    Songbird,
    BearDog,    // ❌ Hardcoded!
    NestGate,
    Squirrel,
    BiomeOS,
}
```

**Evolution**:
```rust
// DEPRECATED - Use capability discovery instead
#[deprecated(since = "2.1.0", note = "Use CapabilityRegistry for discovery")]
pub enum EcosystemPrimalType {
    // Keep for backward compatibility in tests only
    ToadStool,
    Songbird,
    BearDog,
    NestGate,
    Squirrel,
    BiomeOS,
}

// NEW: Capability-based pattern
pub struct DiscoveredPrimal {
    pub id: String,              // Runtime-assigned ID
    pub capabilities: Vec<String>, // What it can do
    pub socket: PathBuf,          // Where to reach it
    // NO primal_type field!
}
```

#### **1.2: Evolve Doctor Module**

**File**: `crates/main/src/doctor.rs`  
**Current** (6 matches):
```rust
fn check_songbird_connectivity() -> HealthCheck {  // ❌ Hardcoded!
    // ...
}

fn check_beardog_connectivity() -> HealthCheck {   // ❌ Hardcoded!
    // ...
}
```

**Evolution**:
```rust
// ✅ Capability-based health checks
async fn check_capability_connectivity(
    capability: &str,  // "http.request", "crypto.signing", etc.
) -> HealthCheck {
    match discover_capability(capability).await {
        Ok(provider) => {
            // Test connectivity to discovered provider
            HealthCheck::healthy(format!(
                "Capability '{}' available from provider {}",
                capability, provider.id
            ))
        }
        Err(_) => {
            HealthCheck::warning(format!(
                "Capability '{}' not currently available",
                capability
            ))
        }
    }
}

// Doctor checks capabilities, not specific primals
pub async fn run_doctor_checks() -> DoctorReport {
    let mut checks = vec![];
    
    // Check for needed capabilities
    checks.push(check_capability_connectivity("crypto.signing").await);
    checks.push(check_capability_connectivity("http.request").await);
    checks.push(check_capability_connectivity("storage.object").await);
    
    // NO mention of BearDog, Songbird, etc.!
    DoctorReport::from_checks(checks)
}
```

#### **1.3: Evolution Checklist - Primal Names**

**High Priority** (Production Code):
- [ ] `crates/main/src/ecosystem/mod.rs` - Deprecate primal enums
- [ ] `crates/main/src/doctor.rs` - Capability-based health checks
- [ ] `crates/main/src/security/` - Use capability discovery
- [ ] `crates/universal-patterns/src/` - Remove primal references
- [ ] `crates/core/auth/` - Capability-based auth

**Medium Priority** (Examples/Demos):
- [ ] `crates/main/examples/` - Update to show capability pattern
- [ ] `docs/` - Update documentation

**Low Priority** (Tests - can keep for specific scenarios):
- [ ] Test files can keep hardcoded names for testing specific integrations
- [ ] Add `#[allow(deprecated)]` to test files

---

### **Phase 2: Vendor Name Elimination** (HIGH - 114 instances)

#### **2.1: Service Registry Abstraction**

**Current** (from discovery mechanisms):
```rust
pub enum RegistryType {
    Consul,      // ❌ Vendor hardcoded!
    Etcd,        // ❌ Vendor hardcoded!
    Kubernetes,  // ❌ Vendor hardcoded!
    Eureka,      // ❌ Vendor hardcoded!
    Custom,
}
```

**Evolution**:
```rust
// ✅ Vendor-agnostic registry interface
pub trait ServiceRegistry: Send + Sync {
    async fn register_service(&self, service: ServiceInfo) -> Result<()>;
    async fn discover_services(&self, capability: &str) -> Result<Vec<ServiceInfo>>;
    async fn health_check(&self) -> Result<HealthStatus>;
}

// Runtime detection from environment
pub async fn create_registry() -> Result<Box<dyn ServiceRegistry>> {
    // Discover which registry is available
    if std::env::var("KUBERNETES_SERVICE_HOST").is_ok() {
        Ok(Box::new(KubernetesRegistry::detect().await?))
    } else if std::env::var("CONSUL_HTTP_ADDR").is_ok() {
        Ok(Box::new(ConsulRegistry::detect().await?))
    } else {
        // Default to file-based or mDNS
        Ok(Box::new(LocalRegistry::new()))
    }
}
```

#### **2.2: Evolution Checklist - Vendor Names**

- [ ] `crates/main/src/discovery/mechanisms/registry.rs` - Abstract registry type
- [ ] `crates/main/src/discovery/mechanisms/` - Vendor-agnostic interfaces
- [ ] Remove hardcoded registry type selection
- [ ] Auto-detect registry from environment

---

### **Phase 3: Port/IP Elimination** (MEDIUM - 460 instances)

#### **3.1: Dynamic Port Assignment**

**Current**:
```rust
const DEFAULT_PORT: u16 = 9010;  // ❌ Hardcoded!
const API_PORT: u16 = 9200;      // ❌ Hardcoded!
```

**Evolution**:
```rust
// ✅ Environment-driven or auto-assigned
fn get_port(service_name: &str) -> u16 {
    // Try environment
    if let Ok(port_str) = std::env::var(format!("{}_PORT", service_name.to_uppercase())) {
        return port_str.parse().unwrap_or(0);
    }
    
    // Auto-assign (OS chooses)
    0  // Let OS assign available port
}

// For Unix sockets (preferred):
fn get_socket_path(service_name: &str) -> PathBuf {
    // Try environment
    if let Ok(path) = std::env::var(format!("{}_SOCKET", service_name.to_uppercase())) {
        return PathBuf::from(path);
    }
    
    // Default pattern (not hardcoded per-primal)
    PathBuf::from(format!("/tmp/{}.sock", service_name))
}
```

#### **3.2: Socket Path Discovery**

**Current**:
```rust
let beardog_socket = "/tmp/beardog.sock";  // ❌ Hardcoded path!
```

**Evolution**:
```rust
// ✅ Discover via capability
let crypto_provider = discover_capability("crypto.signing").await?;
let socket = crypto_provider.socket;  // Got from discovery, not hardcoded
```

#### **3.3: Evolution Checklist - Infrastructure**

- [ ] `crates/universal-constants/src/network.rs` - Remove port constants
- [ ] Replace all `DEFAULT_*_PORT` with environment/auto-assign
- [ ] Replace hardcoded socket paths with discovery
- [ ] Update configuration system to use discovery

---

## 🔄 Migration Strategy

### **Step 1: Create Capability Discovery Module** (Foundation)

**File**: `crates/main/src/capabilities/discovery.rs` (NEW)

```rust
//! Capability Discovery - TRUE PRIMAL Infant Pattern
//! 
//! Discovers capabilities at runtime, NO hardcoded primal names

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Discovered capability provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityProvider {
    /// Runtime-assigned ID (not primal name!)
    pub id: String,
    
    /// Capabilities this provider offers
    pub capabilities: Vec<String>,
    
    /// How to reach it
    pub socket: PathBuf,
    
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

/// Discover provider for a capability
pub async fn discover_capability(
    capability: &str
) -> Result<CapabilityProvider, DiscoveryError> {
    // Method 1: Check explicit environment variable
    let cap_var = format!("{}_PROVIDER_SOCKET", 
        capability.to_uppercase().replace('.', "_"));
    
    if let Ok(socket) = std::env::var(&cap_var) {
        return Ok(CapabilityProvider {
            id: format!("explicit-{}", capability),
            capabilities: vec![capability.to_string()],
            socket: PathBuf::from(socket),
            metadata: Default::default(),
        });
    }
    
    // Method 2: Scan standard socket directory
    let socket_dir = std::env::var("SOCKET_DIR")
        .unwrap_or_else(|_| "/tmp".to_string());
    
    for entry in std::fs::read_dir(socket_dir)? {
        let entry = entry?;
        if entry.path().extension().and_then(|s| s.to_str()) == Some("sock") {
            // Probe socket to ask what capabilities it provides
            if let Ok(provider) = probe_socket(&entry.path()).await {
                if provider.capabilities.contains(&capability.to_string()) {
                    return Ok(provider);
                }
            }
        }
    }
    
    Err(DiscoveryError::CapabilityNotFound(capability.to_string()))
}

/// Probe a socket to discover its capabilities
async fn probe_socket(socket: &Path) -> Result<CapabilityProvider> {
    // Connect and send discovery request
    let mut stream = UnixStream::connect(socket).await?;
    
    // JSON-RPC discovery request
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "discover_capabilities",
        "id": 1
    });
    
    // Send and receive
    // ... implementation ...
    
    Ok(CapabilityProvider {
        id: socket.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string(),
        capabilities: vec![],  // Parse from response
        socket: socket.to_path_buf(),
        metadata: Default::default(),
    })
}
```

### **Step 2: Deprecation Warnings** (Immediate)

Add deprecation warnings to all hardcoded patterns:

```rust
#[deprecated(
    since = "2.1.0",
    note = "Use discover_capability() instead. See HARDCODING_ELIMINATION_EVOLUTION_JAN_20_2026.md"
)]
pub enum EcosystemPrimalType { /* ... */ }

#[deprecated(
    since = "2.1.0",
    note = "Use capability-based health checks instead"
)]
pub fn check_beardog_connectivity() -> HealthCheck { /* ... */ }
```

### **Step 3: Parallel Implementation** (Safe Evolution)

Keep old code working while adding new patterns:

```rust
// Old code (deprecated but still works)
#[allow(deprecated)]
let beardog = connect_to_beardog()?;

// New code (preferred)
let crypto = discover_capability("crypto.signing").await?;
```

### **Step 4: Gradual Migration** (Phase by Phase)

1. **Phase 1**: Add capability discovery alongside existing code
2. **Phase 2**: Update main.rs to use capabilities
3. **Phase 3**: Update modules one by one
4. **Phase 4**: Remove deprecated code (v3.0.0)

---

## ✅ Success Criteria

When complete, Squirrel will:

### **Zero Knowledge Deployment**
```bash
# Start Squirrel with ZERO configuration
./squirrel server

# Output:
# 🐿️  Squirrel starting...
# 👶 Infant mode: Discovering ecosystem...
# 🔍 Scanning /tmp for capability providers...
# ✅ Found capability: crypto.signing (provider: unknown-af3b2)
# ✅ Found capability: http.request (provider: unknown-7f9e1)
# ✅ Found capability: storage.object (provider: unknown-2c4d8)
# 🚀 Ready with 3 capabilities discovered
```

### **No Hardcoded Names**
```bash
# Verify zero primal name hardcoding in production code
$ grep -r "BearDog\|Songbird" crates/main/src --exclude-dir=tests
# Should return ZERO matches (or only deprecated/test code)
```

### **Capability-Based Operations**
```rust
// All operations via capabilities, not primal names
let crypto = discover_capability("crypto.signing").await?;
let http = discover_capability("http.request").await?;
let storage = discover_capability("storage.object").await?;

// Squirrel has NO IDEA who provides these!
// Could be BearDog, could be something else entirely
```

---

## 📊 Impact Assessment

### **Code Changes Required**

```
Primal Names:        1,428 instances → 0 (deprecated/tests only)
Vendor Names:          114 instances → 0 (abstracted)
Port Numbers:          460 instances → environment-driven
Socket Paths:           23 instances → discovered

Total Impact:        2,025 hardcoded references to evolve
Estimated Effort:    15-20 hours (phased over 2-3 weeks)
```

### **Benefits**

1. **TRUE PRIMAL Pattern** ✅
   - Deploy with zero knowledge
   - Discover everything at runtime
   - Like an infant learning

2. **Vendor Agnostic** ✅
   - Works with any service registry
   - Not locked to Kubernetes/Consul
   - Portable across platforms

3. **Dynamic Composition** ✅
   - Ecosystem can evolve
   - Add/remove primals dynamically
   - No recompilation needed

4. **Testing Flexibility** ✅
   - Mock capabilities, not specific primals
   - Test isolation easier
   - No vendor dependencies in tests

---

## 🎯 Next Steps

### **Immediate** (This Week)
1. Create `crates/main/src/capabilities/discovery.rs` module
2. Add deprecation warnings to `EcosystemPrimalType`
3. Update `doctor.rs` to use capability-based checks
4. Test capability discovery with existing infrastructure

### **Short-term** (Next 2 Weeks)
1. Evolve main.rs to use capability discovery
2. Update security module to discover crypto capability
3. Update AI router to discover HTTP capability
4. Add comprehensive tests

### **Long-term** (v3.0.0)
1. Remove all deprecated primal enums
2. Complete vendor abstraction
3. Dynamic port assignment everywhere
4. Full "infant pattern" deployment

---

## 📝 Documentation Updates Needed

- [ ] Update README.md - Show capability discovery
- [ ] Update START_HERE.md - Infant deployment pattern
- [ ] Create CAPABILITY_DISCOVERY_GUIDE.md
- [ ] Update all examples to use capabilities
- [ ] Migration guide for existing deployments

---

**Status**: Audit complete, ready for execution  
**Pattern**: TRUE PRIMAL Infant (zero knowledge)  
**Timeline**: Phased over 2-3 weeks  
**Grade**: Will achieve A++ TRUE PRIMAL certification  

*Deploy like an infant - knows nothing, discovers everything* 🐿️👶

