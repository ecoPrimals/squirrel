# 🔄 Hardcoding Migration Guide - Squirrel Primal
**Date**: January 9, 2026  
**Status**: IN PROGRESS  
**Goal**: Migrate from hardcoded primal names/ports to universal adapter pattern

---

## 🎯 Overview

This guide documents the migration from hardcoded primal dependencies to the universal adapter pattern, implementing the core ecoPrimals principle: **"Each primal knows only itself and discovers others at runtime."**

### The Problem

**Current State** (Anti-Pattern):
```rust
// ❌ BAD: Hardcoded primal names and endpoints
let songbird_url = "https://songbird.ecosystem.local";
let beardog_url = "https://beardog.ecosystem.local";
let nestgate_url = "https://nestgate.ecosystem.local";

// This creates:
// 1. Compile-time coupling between primals
// 2. Inability to adapt to primal evolution
// 3. N² connection complexity (each primal hardcodes all others)
// 4. Deployment inflexibility
// 5. Sovereignty violations
```

**Target State** (Universal Adapter Pattern):
```rust
// ✅ GOOD: Capability-based discovery
let orchestration = universal_adapter
    .discover_capability("service-mesh")
    .await?;

let security = universal_adapter
    .discover_capability("security")
    .await?;

let storage = universal_adapter
    .discover_capability("storage")
    .await?;

// This provides:
// 1. Runtime discovery and binding
// 2. Zero compile-time dependencies
// 3. Automatic failover and load balancing
// 4. Deployment flexibility
// 5. Sovereignty compliance
```

---

## 📊 Migration Scope

### Hardcoding Statistics
- **Primal Names**: 2,546 instances across 234 files
- **Port Numbers**: 617 instances across 158 files
- **Localhost/IPs**: 902 instances across 203 files
- **Total**: 4,065 hardcoded values to migrate

### Priority Files (High Impact)
1. `crates/main/src/primal_provider/core.rs` - 35 instances
2. `crates/main/src/biomeos_integration/ecosystem_client.rs` - 91 instances
3. `crates/main/src/songbird/mod.rs` - 55 instances
4. `crates/main/src/ecosystem/mod.rs` - 70 instances
5. `crates/main/src/capability_migration.rs` - 33 instances

---

## 🏗️ Universal Adapter Architecture

### Components

#### 1. Universal Primal Registry
**Location**: `crates/universal-patterns/src/registry/mod.rs`

```rust
pub struct UniversalPrimalRegistry {
    /// Map of instance ID to primal provider
    registered_primals: RwLock<HashMap<String, Arc<dyn PrimalProvider>>>,
    /// Index of capability to primal instance IDs
    capability_index: RwLock<HashMap<PrimalCapability, Vec<String>>>,
    /// Dynamic port management
    port_manager: RwLock<HashMap<String, DynamicPortInfo>>,
}
```

**Key Methods**:
- `register_primal()` - Register a primal with capabilities
- `discover_by_capability()` - Find primals by capability
- `discover_by_type()` - Find primals by type
- `health_check()` - Check primal health

#### 2. Primal Provider Trait
**Location**: `crates/universal-patterns/src/traits/provider.rs`

```rust
#[async_trait]
pub trait PrimalProvider: Send + Sync {
    fn primal_id(&self) -> &str;
    fn instance_id(&self) -> &str;
    fn capabilities(&self) -> Vec<PrimalCapability>;
    fn dependencies(&self) -> Vec<PrimalDependency>;
    fn endpoints(&self) -> PrimalEndpoints;
    async fn handle_primal_request(&self, request: PrimalRequest) 
        -> PrimalResult<PrimalResponse>;
}
```

#### 3. Capability Types
**Location**: `crates/universal-patterns/src/traits/mod.rs`

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimalCapability {
    // Core capabilities
    Security,           // Authentication, authorization, encryption
    Storage,            // Data persistence, retrieval
    Compute,            // Task execution, processing
    AI,                 // AI inference, model management
    ServiceMesh,        // Service discovery, routing
    Monitoring,         // Health checks, metrics
    Configuration,      // Config management
    
    // Specialized capabilities
    Encryption,
    KeyManagement,
    FileStorage,
    ObjectStorage,
    ContainerRuntime,
    Serverless,
    ModelInference,
    PromptRouting,
    
    // Custom capability
    Custom(String),
}
```

---

## 🔄 Migration Patterns

### Pattern 1: Hardcoded Primal Name → Capability Discovery

#### Before (Anti-Pattern)
```rust
pub async fn coordinate_with_songbird(&self) -> Result<Response, Error> {
    // ❌ Hardcoded primal name and endpoint
    let songbird_url = "https://songbird.ecosystem.local";
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/coordinate", songbird_url))
        .json(&request)
        .send()
        .await?;
    Ok(response)
}
```

#### After (Universal Adapter)
```rust
pub async fn coordinate_with_orchestrator(&self) -> Result<Response, Error> {
    // ✅ Capability-based discovery
    let orchestrator = self.universal_adapter
        .discover_capability(PrimalCapability::ServiceMesh)
        .await?
        .first()
        .ok_or(Error::NoServiceMeshAvailable)?;
    
    let response = self.universal_adapter
        .send_request(orchestrator, request)
        .await?;
    
    Ok(response)
}
```

**Key Changes**:
1. Removed hardcoded "songbird" name
2. Discover by capability ("service-mesh")
3. Use universal adapter for communication
4. Automatic failover if multiple providers exist

---

### Pattern 2: Hardcoded Endpoint → Environment + Discovery

#### Before (Anti-Pattern)
```rust
pub fn new() -> Self {
    // ❌ Hardcoded endpoint
    let beardog_url = "http://localhost:8600";
    Self {
        security_client: SecurityClient::new(beardog_url),
    }
}
```

#### After (Environment + Discovery)
```rust
pub async fn new(universal_adapter: Arc<UniversalAdapter>) -> Result<Self, Error> {
    // ✅ Environment-based with discovery fallback
    let security_provider = if let Ok(url) = env::var("SECURITY_PROVIDER_URL") {
        // Use explicit environment configuration
        SecurityProvider::from_url(url)
    } else {
        // Fall back to capability discovery
        universal_adapter
            .discover_capability(PrimalCapability::Security)
            .await?
            .first()
            .ok_or(Error::NoSecurityProviderAvailable)?
            .clone()
    };
    
    Ok(Self {
        security_client: SecurityClient::new(security_provider),
        universal_adapter,
    })
}
```

**Key Changes**:
1. Accept `UniversalAdapter` in constructor
2. Check environment variables first (explicit config)
3. Fall back to capability discovery
4. No hardcoded URLs or ports

---

### Pattern 3: Multiple Hardcoded Services → Batch Discovery

#### Before (Anti-Pattern)
```rust
pub async fn discover_ecosystem_services(&self) -> Result<Vec<Service>, Error> {
    // ❌ Hardcoded list of all primals
    let services = vec![
        Service {
            id: "songbird-orchestrator",
            url: "https://songbird.ecosystem.local",
            capabilities: vec!["orchestration"],
        },
        Service {
            id: "beardog-security",
            url: "https://beardog.ecosystem.local",
            capabilities: vec!["security"],
        },
        Service {
            id: "nestgate-storage",
            url: "https://nestgate.ecosystem.local",
            capabilities: vec!["storage"],
        },
        Service {
            id: "toadstool-compute",
            url: "https://toadstool.ecosystem.local",
            capabilities: vec!["compute"],
        },
    ];
    Ok(services)
}
```

#### After (Batch Capability Discovery)
```rust
pub async fn discover_ecosystem_services(&self) -> Result<Vec<Service>, Error> {
    // ✅ Discover by required capabilities
    let required_capabilities = vec![
        PrimalCapability::ServiceMesh,
        PrimalCapability::Security,
        PrimalCapability::Storage,
        PrimalCapability::Compute,
    ];
    
    let mut services = Vec::new();
    
    for capability in required_capabilities {
        // Discover all providers for this capability
        let providers = self.universal_adapter
            .discover_capability(capability)
            .await?;
        
        for provider in providers {
            services.push(Service {
                id: provider.instance_id().to_string(),
                url: provider.endpoint().to_string(),
                capabilities: provider.capabilities(),
                health: provider.health().await?,
            });
        }
    }
    
    Ok(services)
}
```

**Key Changes**:
1. Define required capabilities (not primal names)
2. Discover all providers for each capability
3. Support multiple providers per capability
4. Include health status
5. No hardcoded primal names or URLs

---

### Pattern 4: Port Hardcoding → Dynamic Port Resolution

#### Before (Anti-Pattern)
```rust
pub async fn start_server() -> Result<(), Error> {
    // ❌ Hardcoded port
    let addr = "0.0.0.0:9010".parse()?;
    let listener = TcpListener::bind(addr).await?;
    Ok(())
}
```

#### After (Dynamic Port Resolution)
```rust
pub async fn start_server(config: &UniversalPrimalConfig) -> Result<(), Error> {
    // ✅ Dynamic port from config
    let port = config.get_port("squirrel")
        .or_else(|| env::var("SQUIRREL_PORT").ok().and_then(|p| p.parse().ok()))
        .unwrap_or(9010); // Fallback only if no config
    
    let host = config.get_host()
        .or_else(|| env::var("SQUIRREL_HOST").ok())
        .unwrap_or_else(|| "0.0.0.0".to_string());
    
    let addr = format!("{}:{}", host, port).parse()?;
    let listener = TcpListener::bind(addr).await?;
    
    // Register with universal adapter
    config.register_port("squirrel", port).await?;
    
    Ok(())
}
```

**Key Changes**:
1. Accept configuration object
2. Check config first, then environment
3. Fallback to default only if no config
4. Register port with universal adapter
5. Support dynamic host binding

---

## 📋 Migration Checklist

### Phase 1: Core Modules (HIGH PRIORITY)

#### File: `crates/main/src/primal_provider/core.rs`
- [ ] Replace `discover_ecosystem_services()` hardcoded list
- [ ] Replace `coordinate_with_songbird()` hardcoded endpoint
- [ ] Replace service-specific methods with capability-based
- [ ] Add `UniversalAdapter` field to struct
- [ ] Update constructor to accept adapter
- [ ] Update tests to use mock adapter

#### File: `crates/main/src/songbird/mod.rs`
- [ ] Replace `discover_complementary_services()` hardcoded list
- [ ] Replace direct Songbird references with service-mesh capability
- [ ] Update `SongbirdIntegration` to use universal adapter
- [ ] Remove hardcoded endpoints
- [ ] Update tests

#### File: `crates/main/src/biomeos_integration/ecosystem_client.rs`
- [ ] Replace all primal-specific methods
- [ ] Use capability-based discovery
- [ ] Remove hardcoded URLs
- [ ] Update registration logic
- [ ] Update tests

#### File: `crates/main/src/ecosystem/mod.rs`
- [ ] Update `EcosystemPrimalType` enum usage
- [ ] Replace `build_service_endpoint()` hardcoded logic
- [ ] Use universal adapter for discovery
- [ ] Update registry integration
- [ ] Update tests

#### File: `crates/main/src/capability_migration.rs`
- [ ] Complete migration to universal patterns
- [ ] Remove temporary compatibility code
- [ ] Update discovery logic
- [ ] Update tests

### Phase 2: Integration Modules (MEDIUM PRIORITY)

#### Universal Adapters
- [ ] `crates/main/src/universal_adapters/orchestration_adapter.rs`
- [ ] `crates/main/src/universal_adapters/security_adapter.rs`
- [ ] `crates/main/src/universal_adapters/storage_adapter.rs`
- [ ] `crates/main/src/universal_adapters/compute_adapter.rs`

#### Ecosystem Registry
- [ ] `crates/main/src/ecosystem/registry/discovery.rs`
- [ ] `crates/main/src/ecosystem/registry/types.rs`
- [ ] `crates/main/src/ecosystem/registry_manager.rs`

### Phase 3: Client Modules (MEDIUM PRIORITY)

#### Security Client
- [ ] `crates/main/src/security_client/client.rs`
- [ ] `crates/main/src/security/beardog_coordinator.rs`
- [ ] Remove BearDog hardcoding

#### Storage Client
- [ ] `crates/main/src/storage_client/client.rs`
- [ ] Remove NestGate hardcoding

#### Compute Client
- [ ] `crates/main/src/compute_client/client.rs`
- [ ] `crates/main/src/toadstool/integration.rs`
- [ ] Remove Toadstool hardcoding

### Phase 4: Test Fixtures & Examples

#### Test Fixtures
- [ ] Update test configurations
- [ ] Use mock universal adapter
- [ ] Remove hardcoded test endpoints

#### Examples
- [ ] Update example code
- [ ] Use capability discovery
- [ ] Update documentation

---

## 🧪 Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use universal_patterns::testing::MockUniversalAdapter;
    
    #[tokio::test]
    async fn test_capability_discovery() {
        // Create mock adapter
        let mut adapter = MockUniversalAdapter::new();
        
        // Configure mock to return security provider
        adapter.expect_discover_capability()
            .with(eq(PrimalCapability::Security))
            .returning(|_| {
                Ok(vec![MockSecurityProvider::new()])
            });
        
        // Test discovery
        let provider = MyService::new(Arc::new(adapter)).await?;
        let security = provider.get_security_provider().await?;
        
        assert!(security.is_some());
    }
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_real_primal_discovery() {
    // Use real universal adapter with test configuration
    let config = UniversalPrimalConfig::test_config();
    let adapter = UniversalAdapter::new(config).await?;
    
    // Register test primals
    adapter.register_test_primal("test-security", 
        vec![PrimalCapability::Security]).await?;
    
    // Test discovery
    let security = adapter
        .discover_capability(PrimalCapability::Security)
        .await?;
    
    assert!(!security.is_empty());
}
```

---

## 📈 Progress Tracking

### Metrics

| Metric | Before | Current | Target |
|--------|--------|---------|--------|
| **Primal Name Hardcoding** | 2,546 | 2,546 | 0 |
| **Port Hardcoding** | 617 | 617 | <50 |
| **Localhost Hardcoding** | 902 | 902 | <100 |
| **Capability-Based Files** | 5% | 5% | 95% |

### Phase Completion

- [x] **Phase 0**: Build stabilization (COMPLETE)
- [ ] **Phase 1**: Core modules (0/5 files)
- [ ] **Phase 2**: Integration modules (0/8 files)
- [ ] **Phase 3**: Client modules (0/6 files)
- [ ] **Phase 4**: Test fixtures (0/20 files)

### Time Estimates

| Phase | Files | Est. Hours | Status |
|-------|-------|------------|--------|
| Phase 1 | 5 | 10-12 | Not Started |
| Phase 2 | 8 | 8-10 | Not Started |
| Phase 3 | 6 | 6-8 | Not Started |
| Phase 4 | 20 | 6-10 | Not Started |
| **Total** | **39** | **30-40** | **0% Complete** |

---

## 🎯 Quick Reference

### Common Replacements

| Old Pattern | New Pattern |
|-------------|-------------|
| `"songbird"` | `PrimalCapability::ServiceMesh` |
| `"beardog"` | `PrimalCapability::Security` |
| `"nestgate"` | `PrimalCapability::Storage` |
| `"toadstool"` | `PrimalCapability::Compute` |
| `"squirrel"` | `PrimalCapability::AI` |
| `"http://localhost:8080"` | `config.get_endpoint("service")` |
| `"0.0.0.0:9010"` | `config.get_bind_address()` |

### Helper Functions

```rust
// Discover single provider
async fn discover_security(&self) -> Result<Arc<dyn PrimalProvider>, Error> {
    self.universal_adapter
        .discover_capability(PrimalCapability::Security)
        .await?
        .first()
        .cloned()
        .ok_or(Error::NoProviderFound)
}

// Discover with fallback
async fn discover_with_fallback(
    &self,
    primary: PrimalCapability,
    fallback: PrimalCapability,
) -> Result<Arc<dyn PrimalProvider>, Error> {
    self.universal_adapter
        .discover_capability(primary)
        .await?
        .first()
        .cloned()
        .or_else(|| {
            self.universal_adapter
                .discover_capability(fallback)
                .await
                .ok()
                .and_then(|p| p.first().cloned())
        })
        .ok_or(Error::NoProviderFound)
}

// Discover all providers for capability
async fn discover_all(&self, capability: PrimalCapability) 
    -> Result<Vec<Arc<dyn PrimalProvider>>, Error> {
    self.universal_adapter
        .discover_capability(capability)
        .await
}
```

---

## 📚 Additional Resources

### Documentation
- [Universal Patterns Specification](specs/active/UNIVERSAL_PATTERNS_SPECIFICATION.md)
- [Universal Squirrel Ecosystem Spec](specs/active/UNIVERSAL_SQUIRREL_ECOSYSTEM_SPEC.md)
- [Capability-Based Architecture](docs/CAPABILITY_BASED_ARCHITECTURE.md)
- [Sovereignty Compliance](SOVEREIGNTY_COMPLIANCE.md)

### Code References
- Universal Patterns: `crates/universal-patterns/`
- Universal Constants: `crates/universal-constants/`
- Ecosystem API: `crates/ecosystem-api/`

### Mature Primal Examples
- Songbird: `../../songbird/` - Protocol-agnostic, zero hardcoding
- NestGate: `../../nestgate/` - Capability discovery patterns

---

**Status**: 🔄 IN PROGRESS  
**Last Updated**: January 9, 2026  
**Next Update**: After Phase 1 completion

🐿️ **Building a truly sovereign, capability-based ecosystem!** 🦀

