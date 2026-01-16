# Capability-Based Integration Template
## For New Primal Integrations

**Based on**: `songbird/mod.rs` (Perfect Example ⭐)  
**Pattern**: Infant Primal - Zero Knowledge, Runtime Discovery  
**Last Updated**: January 13, 2026

---

## Quick Start

When integrating with a new capability (not a specific primal!), follow this template:

### Step 1: Define the Capability (if new)

```rust
// In crates/main/src/capability_registry.rs or crates/universal-patterns/src/capabilities.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimalCapability {
    // Existing capabilities...
    Authentication,      // Security services
    ServiceMesh,        // Orchestration services
    Compute,            // Compute services
    Storage,            // Storage services
    AIInference,        // AI services
    
    // Add your new capability here
    YourNewCapability,  // Description of what this capability does
}
```

### Step 2: Create the Coordinator Module

```rust
//! [CapabilityName] Integration for Squirrel AI Primal
//!
//! This module provides integration with services providing [capability description]
//! using capability-based discovery instead of hardcoded primal names.
//!
//! # Primal Sovereignty
//!
//! This module uses `CapabilityRegistry` for discovering [capability] services
//! at runtime, rather than hardcoding specific primal names. This allows the
//! ecosystem to evolve without code changes.
//!
//! # Usage Example
//!
//! ```rust,ignore
//! // ✅ GOOD: Capability-based discovery
//! let coordinator = [Capability]Coordinator::new(config, capability_registry)?;
//! coordinator.initialize().await?;
//! 
//! // The coordinator will discover ANY service providing [capability]
//! // No hardcoded primal names!
//! ```

use crate::capability_registry::{CapabilityRegistry, PrimalCapability};
use crate::ecosystem::EcosystemConfig;
use crate::error::PrimalError;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// [Capability] coordinator using capability-based discovery
///
/// # Primal Sovereignty
///
/// This coordinator uses `CapabilityRegistry` to discover [capability] services
/// at runtime, eliminating hardcoded primal references. Each primal knows only
/// itself and discovers others by capability.
pub struct [Capability]Coordinator {
    /// Instance identifier
    instance_id: String,
    /// Ecosystem configuration
    config: EcosystemConfig,
    /// Capability registry for dynamic service discovery
    capability_registry: Arc<CapabilityRegistry>,
    /// Your state here
    state: Arc<RwLock<YourState>>,
    /// Initialization state
    initialized: bool,
}

impl [Capability]Coordinator {
    /// Create a new [capability] coordinator with capability-based discovery
    ///
    /// # Arguments
    ///
    /// * `config` - Ecosystem configuration
    /// * `capability_registry` - Registry for discovering services
    ///
    /// # Primal Sovereignty
    ///
    /// This constructor accepts a `CapabilityRegistry` instead of creating hardcoded
    /// clients. The coordinator discovers services at runtime.
    pub fn new(
        config: EcosystemConfig,
        capability_registry: Arc<CapabilityRegistry>,
    ) -> Result<Self, PrimalError> {
        let instance_id = Uuid::new_v4().to_string();
        let state = Arc::new(RwLock::new(YourState::default()));
        
        Ok(Self {
            instance_id,
            config,
            capability_registry,
            state,
            initialized: false,
        })
    }
    
    /// Initialize [capability] integration via capability-based discovery
    ///
    /// # Primal Sovereignty
    ///
    /// This method discovers [capability] services dynamically rather than
    /// hardcoding connections to specific primal names.
    pub async fn initialize(&mut self) -> Result<(), PrimalError> {
        info!("Initializing [capability] coordinator via capability discovery");
        
        // Discover services providing this capability
        let providers = self
            .capability_registry
            .discover_by_capability(&PrimalCapability::YourCapability)
            .await
            .map_err(|e| {
                PrimalError::ServiceDiscoveryError(format!(
                    "Failed to discover [capability] services: {e}"
                ))
            })?;
        
        if providers.is_empty() {
            warn!("No [capability] services discovered - running in standalone mode");
            // Graceful degradation - service still works without this capability
        } else {
            info!(
                "Discovered {} [capability] service(s)",
                providers.len()
            );
            // Connect to discovered services
        }
        
        self.initialized = true;
        info!("[Capability] coordinator initialized successfully");
        Ok(())
    }
    
    /// Use the capability
    ///
    /// # Primal Sovereignty
    ///
    /// Discovers services dynamically instead of hardcoding targets.
    pub async fn use_capability(&self, params: YourParams) -> Result<YourResult, PrimalError> {
        if !self.initialized {
            return Err(PrimalError::NotInitialized(
                "[Capability] coordinator not initialized".to_string()
            ));
        }
        
        // Discover services dynamically
        let providers = self
            .capability_registry
            .discover_by_capability(&PrimalCapability::YourCapability)
            .await
            .map_err(|e| {
                PrimalError::ServiceDiscoveryError(format!(
                    "Failed to discover [capability] services: {e}"
                ))
            })?;
        
        if providers.is_empty() {
            warn!("No [capability] services available - using fallback");
            return self.fallback_implementation(params).await;
        }
        
        // Use first healthy provider (could implement load balancing here)
        let provider = &providers[0];
        info!(
            "Using [capability] service at: {}",
            provider.endpoint.as_ref()
        );
        
        // Make the actual call to the discovered service
        // (via HTTP, gRPC, or other protocol)
        // ...
        
        Ok(YourResult::default())
    }
    
    /// Fallback implementation when no services available
    ///
    /// This enables graceful degradation and standalone operation.
    async fn fallback_implementation(&self, params: YourParams) -> Result<YourResult, PrimalError> {
        warn!("Using fallback implementation for [capability]");
        // Implement basic functionality that works without external services
        Ok(YourResult::default())
    }
    
    /// Register this coordinator's capabilities
    ///
    /// # Primal Sovereignty
    ///
    /// Self-registers this coordinator's capabilities in the registry, allowing
    /// other primals to discover us dynamically.
    pub async fn register_capabilities(&self) -> Result<String, PrimalError> {
        info!("Registering [capability] capabilities in capability registry");
        
        let service_id = format!("squirrel-[capability]-{}", self.instance_id);
        
        let mut capabilities = std::collections::HashSet::new();
        capabilities.insert(PrimalCapability::YourCapability);
        
        // Capability-based discovery: discover endpoint at runtime
        // Falls back to environment configuration only if discovery unavailable
        let endpoint = std::env::var("YOUR_CAPABILITY_ENDPOINT")
            .or_else(|_| {
                let port = std::env::var("YOUR_CAPABILITY_PORT")
                    .unwrap_or_else(|_| "8080".to_string());
                // Note: This is a fallback for local development only
                // Production should use capability discovery from service mesh
                Ok::<String, std::env::VarError>(format!("http://localhost:{port}"))
            })
            .unwrap_or_else(|_| {
                // Final fallback for local dev
                tracing::warn!("Using fallback endpoint - capability discovery recommended");
                "http://localhost:8080".to_string()
            });
        
        let health_endpoint = format!("{endpoint}/health");
        
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("coordinator_instance".to_string(), self.instance_id.clone());
        metadata.insert("primal_type".to_string(), "squirrel".to_string());
        
        self.capability_registry
            .register_primal(
                service_id.clone(),
                "Squirrel [Capability] Coordinator".to_string(),
                capabilities,
                endpoint,
                health_endpoint,
                metadata,
            )
            .await
            .map_err(|e| {
                PrimalError::Registry(format!(
                    "Failed to register [capability] capabilities: {e}"
                ))
            })?;
        
        info!("Successfully registered [capability] service: {}", service_id);
        Ok(service_id)
    }
    
    /// Shutdown the coordinator
    pub async fn shutdown(&mut self) -> Result<(), PrimalError> {
        info!("Shutting down [capability] coordinator");
        self.initialized = false;
        Ok(())
    }
}

// Your state/config types
#[derive(Debug, Default)]
struct YourState {
    // ...
}

#[derive(Debug)]
struct YourParams {
    // ...
}

#[derive(Debug, Default)]
struct YourResult {
    // ...
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability_registry::CapabilityRegistryConfig;
    
    fn create_test_registry() -> Arc<CapabilityRegistry> {
        Arc::new(CapabilityRegistry::new(CapabilityRegistryConfig::default()))
    }
    
    #[tokio::test]
    async fn test_coordinator_initialization() {
        let registry = create_test_registry();
        let mut coordinator = [Capability]Coordinator::new(
            EcosystemConfig::default(),
            registry
        ).unwrap();
        
        assert!(coordinator.initialize().await.is_ok());
        assert!(coordinator.initialized);
    }
    
    #[tokio::test]
    async fn test_capability_discovery() {
        let registry = create_test_registry();
        
        // Register a test service
        let mut capabilities = std::collections::HashSet::new();
        capabilities.insert(PrimalCapability::YourCapability);
        
        registry.register_primal(
            "test-provider-1".to_string(),
            "Test Provider".to_string(),
            capabilities,
            "http://localhost:9000".to_string(),
            "http://localhost:9000/health".to_string(),
            std::collections::HashMap::new(),
        ).await.unwrap();
        
        // Test discovery
        let providers = registry
            .discover_by_capability(&PrimalCapability::YourCapability)
            .await
            .unwrap();
        
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0].display_name.as_ref(), "Test Provider");
    }
}
```

---

## Key Patterns

### ✅ DO: Capability-Based Discovery

```rust
// Discover by capability, not by primal name
let providers = capability_registry
    .discover_by_capability(&PrimalCapability::Authentication)
    .await?;
```

### ❌ DON'T: Hardcoded Primal Names

```rust
// Never hardcode primal names
let beardog_client = BearDogClient::new("http://localhost:7443")?; // ❌ BAD
```

### ✅ DO: Graceful Degradation

```rust
if providers.is_empty() {
    warn!("No services discovered - running in standalone mode");
    return self.fallback_implementation().await;
}
```

### ✅ DO: Environment Fallbacks (Development Only)

```rust
let endpoint = std::env::var("SERVICE_MESH_ENDPOINT")
    .or_else(|_| std::env::var("CAPABILITY_ENDPOINT"))
    .unwrap_or_else(|_| {
        tracing::warn!("Using fallback endpoint - discovery recommended");
        format!("http://localhost:{}", find_available_port())
    });
```

### ✅ DO: Document Primal Sovereignty

```rust
/// # Primal Sovereignty
///
/// This method discovers services dynamically rather than hardcoding
/// connections to specific primal names.
```

---

## Configuration Pattern

For configuration structs, follow the `SongbirdConfig::default()` pattern:

```rust
#[derive(Debug, Clone)]
pub struct [Capability]Config {
    pub endpoint: String,
    pub timeout: Duration,
    pub max_retries: u32,
}

impl Default for [Capability]Config {
    fn default() -> Self {
        Self {
            // Prefer discovery, fallback to environment, final fallback for dev
            endpoint: std::env::var("SERVICE_MESH_ENDPOINT")
                .or_else(|_| std::env::var("[CAPABILITY]_ENDPOINT"))
                .unwrap_or_else(|_| {
                    let port = std::env::var("[CAPABILITY]_PORT")
                        .unwrap_or_else(|_| "8080".to_string());
                    tracing::warn!(
                        "Using fallback [capability] endpoint - service mesh discovery recommended"
                    );
                    format!("http://localhost:{port}")
                }),
            
            timeout: std::env::var("[CAPABILITY]_TIMEOUT_SECS")
                .ok()
                .and_then(|s| s.parse::<u64>().ok())
                .map(Duration::from_secs)
                .unwrap_or(Duration::from_secs(30)),
            
            max_retries: std::env::var("[CAPABILITY]_MAX_RETRIES")
                .ok()
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(3),
        }
    }
}
```

---

## Environment Variables

Use capability-based naming:

```bash
# ✅ GOOD: Capability-based
SERVICE_MESH_ENDPOINT=http://songbird:9090
AUTHENTICATION_ENDPOINT=http://beardog:7443
COMPUTE_ENDPOINT=http://toadstool:8500

# ❌ BAD: Primal name-based
BEARDOG_URL=http://beardog:7443
SONGBIRD_URL=http://songbird:9090
```

---

## Testing

Always test with multiple providers:

```rust
#[tokio::test]
async fn test_works_with_any_provider() {
    let registry = create_test_registry();
    
    // Register different providers for the same capability
    for i in 1..=3 {
        let mut capabilities = HashSet::new();
        capabilities.insert(PrimalCapability::YourCapability);
        
        registry.register_primal(
            format!("provider-{}", i),
            format!("Provider {}", i),
            capabilities,
            format!("http://localhost:{}", 9000 + i),
            format!("http://localhost:{}/health", 9000 + i),
            HashMap::new(),
        ).await.unwrap();
    }
    
    // Coordinator should work with any of them
    let coordinator = [Capability]Coordinator::new(
        EcosystemConfig::default(),
        registry
    ).unwrap();
    
    // Test succeeds regardless of which provider is used
    assert!(coordinator.use_capability(params).await.is_ok());
}
```

---

## Migration from Hardcoded Integration

If you have existing hardcoded integration:

1. **Add deprecation warning**:
```rust
#[deprecated(
    since = "3.0.0",
    note = "Use CapabilityRegistry to discover [capability] instead. See CAPABILITY_INTEGRATION_TEMPLATE.md"
)]
pub struct OldHardcodedClient { }
```

2. **Create new capability-based coordinator** using this template

3. **Update callers** to use new coordinator

4. **Remove deprecated code** after migration period

---

## Checklist

When creating a new capability integration:

- [ ] Define capability in `PrimalCapability` enum
- [ ] Create coordinator struct with `CapabilityRegistry`
- [ ] Implement `initialize()` with discovery
- [ ] Implement graceful degradation (standalone mode)
- [ ] Add "Primal Sovereignty" documentation
- [ ] Use environment variables only as fallback
- [ ] Register own capabilities for discoverability
- [ ] Add tests with multiple providers
- [ ] Document in module-level docs
- [ ] Add usage examples

---

**Template Version**: 1.0.0  
**Based On**: `crates/main/src/songbird/mod.rs`  
**Last Updated**: January 13, 2026

🐿️ **Zero hardcoding. Pure capability discovery.** 🦀

