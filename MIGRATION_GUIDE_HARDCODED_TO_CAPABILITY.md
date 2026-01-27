# 🔄 Migration Guide: Hardcoded → Capability-Based Discovery

**Purpose**: Transform hardcoded primal dependencies into TRUE PRIMAL capability-based patterns  
**Status**: Active Migration Guide  
**Target**: Zero hardcoded primal names in production code

---

## 🎯 Core Principle

**TRUE PRIMAL Standard**:
> Primals have self-knowledge only. They discover other primals at runtime by capability, not by name.

```rust
// ❌ WRONG (hardcoded primal name):
let beardog_url = "http://beardog:9000";
let client = BearDogClient::new(beardog_url);

// ✅ RIGHT (capability-based):
let crypto = discover_capability("crypto.signing").await?;
// We have ZERO knowledge of WHO provides it!
```

---

## 📋 Migration Patterns

### Pattern 1: Replace Hardcoded Enum with Capability

**BEFORE** (Hardcoded):
```rust
#[derive(Debug, Clone, Copy)]
pub enum EcosystemPrimalType {
    ToadStool,   // ❌ Hardcoded name
    Songbird,    // ❌ Hardcoded name
    BearDog,     // ❌ Hardcoded name
    NestGate,    // ❌ Hardcoded name
}

// Usage:
let primal = EcosystemPrimalType::Songbird;
let endpoint = primal.default_endpoint();
```

**AFTER** (Capability-Based):
```rust
// No enum! Just capabilities
pub struct CapabilityRequest {
    pub capability: String,  // "service.discovery", "crypto.signing", etc.
    pub version: Option<String>,
}

// Usage:
let request = CapabilityRequest {
    capability: "service.discovery".to_string(),
    version: None,
};
let providers = discover_providers(request).await?;
// We get WHOEVER provides service.discovery (could be anyone!)
```

---

### Pattern 2: Replace Hardcoded Methods with Discovery

**BEFORE** (Hardcoded Method Names):
```rust
impl EcosystemManager {
    /// Register with Songbird service mesh
    pub async fn register_with_songbird(&self) -> Result<()> {
        // ❌ Hardcoded: "songbird"
        let songbird_url = "http://songbird:8080";
        // ...
    }
    
    /// Store data using NestGate
    pub async fn store_with_nestgate(&self, data: &[u8]) -> Result<()> {
        // ❌ Hardcoded: "nestgate"
        let nestgate_url = "http://nestgate:9000";
        // ...
    }
}
```

**AFTER** (Capability-Based):
```rust
impl EcosystemManager {
    /// Register with service mesh (whoever provides it)
    pub async fn register_with_service_mesh(&self) -> Result<()> {
        // ✅ Discover service mesh capability
        let service_mesh = self.discover_capability("service.discovery").await?;
        
        // Call via JSON-RPC (no hardcoded knowledge!)
        self.call_json_rpc(&service_mesh.endpoint, "register", self.info()).await?;
        Ok(())
    }
    
    /// Store data using storage capability
    pub async fn store_data(&self, data: &[u8]) -> Result<()> {
        // ✅ Discover storage capability
        let storage = self.discover_capability("storage.put").await?;
        
        // Call via JSON-RPC
        self.call_json_rpc(&storage.endpoint, "put", json!({"data": data})).await?;
        Ok(())
    }
}
```

---

### Pattern 3: Environment-First Discovery

**BEFORE** (Hardcoded URLs):
```rust
let songbird_endpoint = "http://localhost:8080";  // ❌ Hardcoded
let beardog_endpoint = "http://beardog:9000";     // ❌ Hardcoded
```

**AFTER** (Environment-First):
```rust
// Priority 1: Environment variable
if let Ok(endpoint) = std::env::var("SERVICE_DISCOVERY_ENDPOINT") {
    return Ok(endpoint);
}

// Priority 2: Well-known socket paths
let well_known = [
    "/tmp/primal-service-discovery.sock",
    "/var/run/service-mesh.sock",
];
for path in &well_known {
    if path_exists(path).await {
        return Ok(path.to_string());
    }
}

// Priority 3: mDNS/DNS-SD discovery
// ...

// Only if all else fails:
Err(anyhow::anyhow!("Cannot discover service.discovery capability"))
```

---

### Pattern 4: Replace Direct Imports with Abstraction

**BEFORE** (Direct Import):
```rust
use beardog_client::BearDogClient;  // ❌ Direct primal dependency

pub struct JwtService {
    beardog: BearDogClient,  // ❌ Hardcoded client
}

impl JwtService {
    pub async fn sign(&self, data: &[u8]) -> Result<Vec<u8>> {
        self.beardog.ed25519_sign(data).await  // ❌ Direct call
    }
}
```

**AFTER** (Abstraction Layer):
```rust
use capability_crypto::CapabilityCryptoProvider;  // ✅ Generic provider

pub struct JwtService {
    crypto: CapabilityCryptoProvider,  // ✅ Capability-based
}

impl JwtService {
    pub async fn sign(&self, data: &[u8]) -> Result<Vec<u8>> {
        // ✅ Discovers crypto.signing at runtime
        self.crypto.sign_ed25519(data).await
    }
}
```

---

## 🛠️ Step-by-Step Migration Process

### Step 1: Identify Hardcoded References

```bash
# Find all primal name references:
rg -i "beardog|songbird|nestgate|toadstool" crates/main/src/

# Exclude:
# - Tests (crates/main/tests/) - OK to have hardcoded names
# - Examples (examples/) - OK for documentation
# - Comments (review manually)
```

### Step 2: Categorize References

For each file with references:

**A. Production Code** (MUST fix):
- Direct primal imports
- Hardcoded URLs/endpoints
- Primal-specific method names
- Enum variants with primal names

**B. Configuration** (Environment variables):
- Move to environment variables
- Document in `.env.example`
- Use capability-based naming

**C. Tests** (OK to keep):
- Test fixtures
- Mock setups
- Integration test scenarios

### Step 3: Create Capability Abstraction

For each hardcoded reference, create a capability-based alternative:

```rust
// Pattern:
// 1. Identify capability needed: "crypto", "storage", "service.discovery"
// 2. Create discovery call
// 3. Use JSON-RPC for communication
// 4. No hardcoded primal knowledge

pub async fn discover_capability(
    &self,
    capability: &str,
) -> Result<DiscoveredService> {
    // Environment first
    let env_key = format!("{}_ENDPOINT", capability.to_uppercase().replace('.', "_"));
    if let Ok(endpoint) = std::env::var(&env_key) {
        return Ok(DiscoveredService { endpoint, ..Default::default() });
    }
    
    // Well-known paths
    // mDNS/DNS-SD
    // Service registry
    
    Err(anyhow::anyhow!("Capability {} not found", capability))
}
```

### Step 4: Update Usage Sites

Replace each hardcoded reference systematically:

```rust
// BEFORE:
let beardog = BearDogClient::new("http://beardog:9000");
let signature = beardog.ed25519_sign(data).await?;

// AFTER:
let crypto = self.discover_capability("crypto.signing").await?;
let request = json!({
    "jsonrpc": "2.0",
    "method": "crypto.sign",
    "params": {"algorithm": "ed25519", "data": base64_encode(data)},
    "id": 1
});
let response = self.call_json_rpc(&crypto.endpoint, request).await?;
let signature = base64_decode(response["result"]["signature"].as_str().unwrap())?;
```

### Step 5: Test Migration

```bash
# 1. Set environment variables for testing:
export CRYPTO_SIGNING_ENDPOINT=/tmp/beardog.sock
export SERVICE_DISCOVERY_ENDPOINT=/tmp/songbird.sock
export STORAGE_ENDPOINT=/tmp/nestgate.sock

# 2. Run tests:
cargo test --workspace

# 3. Verify no hardcoded names in production:
rg -i "beardog|songbird|nestgate" crates/main/src/ --type rust \
  | grep -v test \
  | grep -v "// " \
  | wc -l
# Should be 0 or very close to 0
```

---

## 📊 Progress Tracking

### Files Migrated: 0 / ~84 files

**High Priority** (most references):
- [ ] `crates/main/src/ecosystem/mod.rs` (57 refs) - IN PROGRESS
- [ ] `crates/main/src/biomeos_integration/mod.rs` (46 refs)
- [ ] `crates/main/src/ecosystem/types.rs` (25 refs)
- [ ] `crates/main/src/ecosystem/ecosystem_types_tests.rs` (41 refs) - TESTS OK
- [ ] `crates/main/src/security/beardog_coordinator.rs` (8 refs)

**Medium Priority**:
- [ ] `crates/main/src/universal_provider.rs` (20 refs)
- [ ] `crates/main/src/primal_provider/core.rs` (14 refs)
- [ ] Others...

### Migration Checklist Per File:

- [ ] Identify all hardcoded primal references
- [ ] Separate production code from tests
- [ ] Create capability-based alternatives
- [ ] Update all usage sites
- [ ] Remove hardcoded imports
- [ ] Test with environment variables
- [ ] Update documentation
- [ ] Mark as complete

---

## 🎯 Target Architecture

### After Migration:

```rust
// Squirrel only knows about itself:
pub struct SquirrelIdentity {
    name: "squirrel",
    capabilities: ["ai", "mcp", "orchestration"],
    version: "2.0.0",
}

// Squirrel discovers others by capability:
let crypto = discover_capability("crypto").await?;      // Could be BearDog, could be anything
let storage = discover_capability("storage").await?;    // Could be NestGate, could be anything
let mesh = discover_capability("service.discovery").await?;  // Could be Songbird, could be anything

// Zero compile-time knowledge of other primals!
```

---

## 💡 Common Patterns

### 1. Service Registration

```rust
// BEFORE:
manager.register_with_songbird().await?;

// AFTER:
let mesh = manager.discover_capability("service.discovery").await?;
manager.register_with_service_mesh(&mesh).await?;
```

### 2. Crypto Operations

```rust
// BEFORE:
let beardog = BearDogClient::new(config);
beardog.sign(data).await?;

// AFTER:
let mut crypto = CapabilityCryptoProvider::new();
crypto.sign_ed25519(data).await?;  // Discovers at runtime
```

### 3. Storage Operations

```rust
// BEFORE:
let nestgate = NestGateClient::new("http://nestgate:9000");
nestgate.put(key, value).await?;

// AFTER:
let storage = discover_capability("storage.put").await?;
call_json_rpc(&storage.endpoint, "put", json!({"key": key, "value": value})).await?;
```

---

## 🚨 Anti-Patterns to Avoid

### ❌ Don't Create New Hardcoded References

```rust
// ❌ BAD - hardcoded name:
const BEARDOG_ENDPOINT: &str = "http://beardog:9000";

// ✅ GOOD - environment-based:
fn get_crypto_endpoint() -> Result<String> {
    std::env::var("CRYPTO_ENDPOINT")
        .or_else(|_| discover_via_mdns("crypto"))
}
```

### ❌ Don't Use Primal Names in Method Names

```rust
// ❌ BAD:
pub async fn call_beardog(&self) -> Result<()> { }
pub async fn query_songbird(&self) -> Result<()> { }

// ✅ GOOD:
pub async fn call_crypto_provider(&self) -> Result<()> { }
pub async fn query_service_mesh(&self) -> Result<()> { }
```

### ❌ Don't Create Primal-Specific Types

```rust
// ❌ BAD:
pub struct BearDogConfig { }
pub struct SongbirdConfig { }

// ✅ GOOD:
pub struct CryptoProviderConfig { }
pub struct ServiceMeshConfig { }
```

---

## ✅ Success Criteria

Migration is complete when:

1. ✅ **Zero hardcoded primal names** in production code
2. ✅ **All capabilities discovered at runtime**
3. ✅ **Environment variables for configuration**
4. ✅ **JSON-RPC for all inter-primal communication**
5. ✅ **Tests pass** with capability discovery
6. ✅ **Documentation updated** with capability patterns
7. ✅ **Grade: TRUE PRIMAL Compliant** (not "F")

---

**Status**: 🚀 Active Migration  
**Progress**: 5% Complete (infrastructure created)  
**Target**: 100% TRUE PRIMAL Compliance

🐿️ **From hardcoded to capability-based!** 🦀✨

