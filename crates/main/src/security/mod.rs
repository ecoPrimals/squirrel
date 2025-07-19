//! Universal Security Adapter for Ecosystem Integration
//!
//! This module implements universal security patterns that enable Squirrel to integrate
//! with any security primal in the ecosystem through standardized interfaces.

/// Security provider implementation
pub mod adapter;

/// Universal Security Adapter - capability-based security provider discovery
pub mod universal_security_adapter;

/// Security configuration
pub mod config;

/// Metrics collection
pub mod metrics;

pub use adapter::*;
pub use universal_security_adapter::*;
pub use config::SecurityConfig;
pub use metrics::SecurityMetrics;

// Re-export the main types and traits for convenience
use crate::universal::{UniversalSecurityProvider, ServiceCapability};

/// Create universal security adapter powered by Songbird service mesh
/// 
/// This function demonstrates the correct Universal Primal Architecture pattern:
/// - **Leverages Songbird's proven service mesh** for registry, discovery, load balancing
/// - **Focuses Squirrel** on MCP protocol coordination and AI intelligence
/// - **Discovers any security provider** based on capabilities (BearDog, future primals, etc.)
/// - **Benefits from network effects** through Songbird's broader ecosystem
/// 
/// Architecture:
/// - 🎵 **Songbird handles**: Service registry, discovery, communications, orchestration
/// - 🐿️ **Squirrel handles**: MCP protocol, AI coordination, context management
/// - 🔗 **Universal pattern**: Capability-based discovery of ANY compatible security primal
pub async fn create_songbird_powered_security_provider(
    songbird_registry: std::sync::Arc<songbird_registry::service::ServiceRegistry>,
    songbird_discovery: std::sync::Arc<songbird_discovery::client::DiscoveryClient>,
    config: Option<UniversalSecurityConfig>,
) -> Result<impl UniversalSecurityProvider<Session = crate::universal::UniversalSecuritySession, Error = crate::error::PrimalError>, crate::error::PrimalError> {
    let config = config.unwrap_or_default();
    
    tracing::info!("🎵 Initializing Songbird-Powered Universal Security Provider");
    tracing::info!("🐿️ Squirrel focusing on MCP/AI coordination while leveraging Songbird network effects");
    tracing::info!("🔍 Will discover: BearDog (if available), future primals, custom security services");
    
    let adapter = UniversalSecurityAdapter::new(songbird_registry, songbird_discovery, config).await?;
    
    tracing::info!("✅ Songbird-powered security provider ready - dynamic discovery enabled");
    
    Ok(adapter)
}

/// Initialize Songbird clients for security provider integration
/// 
/// This helper function sets up the necessary Songbird infrastructure
/// for the universal security adapter to leverage network effects.
pub async fn initialize_songbird_integration() -> Result<(
    std::sync::Arc<songbird_registry::service::ServiceRegistry>,
    std::sync::Arc<songbird_discovery::client::DiscoveryClient>
), crate::error::PrimalError> {
    use songbird_registry::service::ServiceRegistry;
    use songbird_discovery::client::DiscoveryClient;
    
    tracing::info!("🎵 Initializing Songbird service mesh integration");
    
    // Initialize Songbird registry client
    let registry = ServiceRegistry::new()
        .await
        .map_err(|e| crate::error::PrimalError::InitializationFailed(
            format!("Failed to initialize Songbird registry: {}", e)
        ))?;
        
    // Initialize Songbird discovery client  
    let discovery = DiscoveryClient::new()
        .await
        .map_err(|e| crate::error::PrimalError::InitializationFailed(
            format!("Failed to initialize Songbird discovery: {}", e)
        ))?;
    
    tracing::info!("✅ Songbird service mesh clients initialized successfully");
    
    Ok((std::sync::Arc::new(registry), std::sync::Arc::new(discovery)))
}

/// Register Squirrel with Songbird service mesh
/// 
/// This registers Squirrel as an MCP/AI coordination service in Songbird's
/// ecosystem registry, enabling other services to discover our capabilities.
pub async fn register_squirrel_with_songbird(
    songbird_registry: std::sync::Arc<songbird_registry::service::ServiceRegistry>,
) -> Result<(), crate::error::PrimalError> {
    use songbird_core::service::{ServiceInfo, ServiceCapability as SongbirdCapability};
    use std::collections::HashMap;
    use uuid::Uuid;
    
    tracing::info!("📝 Registering Squirrel with Songbird service mesh");
    
    let squirrel_service = ServiceInfo {
        id: Uuid::new_v4().to_string(),
        name: "squirrel".to_string(),
        service_type: "mcp_coordination".to_string(),
        version: "1.0.0".to_string(),
        endpoints: vec![
            songbird_core::service::ServiceEndpoint {
                name: "mcp_protocol".to_string(),
                url: "internal://squirrel/mcp".to_string(),
                protocol: "mcp".to_string(),
                port: None,
            },
            songbird_core::service::ServiceEndpoint {
                name: "ai_coordination".to_string(),
                url: "internal://squirrel/ai".to_string(),
                protocol: "ai_coordination".to_string(),
                port: None,
            },
        ],
        capabilities: vec![
            SongbirdCapability::Custom {
                name: "mcp_protocol".to_string(),
                metadata: HashMap::from([
                    ("version".to_string(), serde_json::Value::String("2024".to_string())),
                    ("ai_enhanced".to_string(), serde_json::Value::Bool(true)),
                    ("context_management".to_string(), serde_json::Value::Bool(true)),
                ]),
            },
            SongbirdCapability::Custom {
                name: "ai_coordination".to_string(),
                metadata: HashMap::from([
                    ("intelligent_routing".to_string(), serde_json::Value::Bool(true)),
                    ("context_awareness".to_string(), serde_json::Value::Bool(true)),
                    ("cost_optimization".to_string(), serde_json::Value::Bool(true)),
                ]),
            },
            SongbirdCapability::Custom {
                name: "session_management".to_string(),
                metadata: HashMap::from([
                    ("mcp_optimized".to_string(), serde_json::Value::Bool(true)),
                    ("ai_enhanced".to_string(), serde_json::Value::Bool(true)),
                ]),
            },
        ],
        metadata: HashMap::from([
            ("primal_type".to_string(), serde_json::Value::String("squirrel".to_string())),
            ("focus".to_string(), serde_json::Value::String("mcp_ai_coordination".to_string())),
            ("ecosystem_role".to_string(), serde_json::Value::String("protocol_coordinator".to_string())),
            ("ai_first_score".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.98).unwrap())),
        ]),
        health_endpoint: Some("internal://squirrel/health".to_string()),
        tags: vec!["mcp".to_string(), "ai".to_string(), "coordination".to_string()],
    };
    
    songbird_registry.register_service(squirrel_service)
        .await
        .map_err(|e| crate::error::PrimalError::RegistrationError(
            format!("Failed to register Squirrel with Songbird: {}", e)
        ))?;
        
    tracing::info!("✅ Squirrel successfully registered with Songbird service mesh");
    tracing::info!("🌐 Now participating in ecosystem network effects");
    
    Ok(())
}

/// Demonstrate how BearDog (or any security primal) would register with Songbird
/// 
/// This shows the proper Universal Primal Architecture pattern where any service
/// can register its capabilities with Songbird, and Squirrel will automatically
/// discover and integrate with it based on capability matching.
pub async fn example_beardog_songbird_registration(
    songbird_registry: std::sync::Arc<songbird_registry::service::ServiceRegistry>,
) -> Result<(), crate::error::PrimalError> {
    use songbird_core::service::{ServiceInfo, ServiceCapability as SongbirdCapability};
    use std::collections::HashMap;
    use uuid::Uuid;
    
    tracing::info!("🐕 Example: How BearDog would register with Songbird service mesh");
    
    let beardog_service = ServiceInfo {
        id: Uuid::new_v4().to_string(),
        name: "beardog".to_string(),
        service_type: "security_provider".to_string(),
        version: "2.0.0".to_string(),
        endpoints: vec![
            songbird_core::service::ServiceEndpoint {
                name: "authenticate".to_string(),
                url: "https://beardog.local:8443/auth".to_string(),
                protocol: "https".to_string(),
                port: Some(8443),
            },
            songbird_core::service::ServiceEndpoint {
                name: "authorize".to_string(),
                url: "https://beardog.local:8443/authz".to_string(),
                protocol: "https".to_string(),
                port: Some(8443),
            },
        ],
        capabilities: vec![
            SongbirdCapability::Authentication {
                methods: vec![
                    "mcp_session_management".to_string(),
                    "ai_credential_validation".to_string(),
                    "context_aware_auth".to_string(),
                    "cryptographic_proof".to_string(),
                    "genetic_security".to_string(),
                ],
            },
            SongbirdCapability::Authorization {
                features: vec![
                    "ai_permission_routing".to_string(),
                    "context_based_access".to_string(),
                    "intelligent_authorization".to_string(),
                    "consensus_workflows".to_string(),
                    "risk_assessment".to_string(),
                ],
            },
            SongbirdCapability::Security {
                level: "enterprise".to_string(),
                features: vec![
                    "cryptographic_proofs".to_string(),
                    "cross_node_trust".to_string(),
                    "genetic_security".to_string(),
                ],
            },
        ],
        metadata: HashMap::from([
            ("security_level".to_string(), serde_json::Value::String("enterprise".to_string())),
            ("primal_type".to_string(), serde_json::Value::String("beardog".to_string())),
            ("genetic_algorithms".to_string(), serde_json::Value::Bool(true)),
            ("zero_unsafe_code".to_string(), serde_json::Value::Bool(true)),
        ]),
        health_endpoint: Some("https://beardog.local:8443/health".to_string()),
        tags: vec!["security".to_string(), "enterprise".to_string(), "genetic".to_string()],
    };
    
    // In a real scenario, BearDog would do this registration itself
    songbird_registry.register_service(beardog_service)
        .await
        .map_err(|e| crate::error::PrimalError::RegistrationError(
            format!("Failed to register BearDog example with Songbird: {}", e)
        ))?;
        
    tracing::info!("✅ BearDog example registered with Songbird");
    tracing::info!("🔍 Squirrel will now automatically discover and integrate with BearDog based on capabilities");
    tracing::info!("🎯 Perfect capability match - BearDog provides all MCP/AI security features needed");
    
    Ok(())
}

// Old registry-building functions removed - we now use Songbird's proven service mesh
// for all registry, discovery, load balancing, and orchestration needs.
// This follows the Universal Primal Architecture principle of leveraging
// existing ecosystem infrastructure rather than rebuilding it.

// Re-export commonly used types from std and external crates
pub use async_trait::async_trait;
pub use chrono::{DateTime, Utc};
pub use serde::{Deserialize, Serialize};
pub use std::collections::HashMap;
pub use std::time::Duration;
pub use tracing::{debug, info, warn};

use crate::universal::SecurityLevel;
