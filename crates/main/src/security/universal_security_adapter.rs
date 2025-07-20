//! Universal Security Adapter
//!
//! Capability-based security provider discovery and integration following
//! the Universal Primal Architecture standard. This adapter dynamically
//! discovers and integrates with ANY security primal based on capabilities,
//! not hardcoded service names.
//!
//! ## Core Principle: Capability-First, Name-Agnostic
//!
//! > "Systems should discover and integrate based on what they can do, not what they're called"
//!
//! ## Architecture: Leveraging Songbird Service Mesh
//!
//! This adapter uses **Songbird's proven service mesh infrastructure** for:
//! - 🎵 **Service Registry & Discovery** - Leverage Songbird's broader ecosystem registry
//! - ⚖️ **Load Balancing** - Use Songbird's intelligent load balancing
//! - 🎼 **Network Effects** - Access to larger ecosystem participant network
//! - 📡 **Communications** - Proven networking infrastructure
//!
//! While **Squirrel focuses on** what it does best:
//! - 🐿️ **MCP Protocol Coordination** - AI-first MCP request handling
//! - 🧠 **AI Intelligence Routing** - Context-aware AI request optimization
//! - 📋 **Context Management** - Session and state coordination

use async_trait::async_trait;
use chrono::Utc;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

// Songbird service mesh integration
// TODO: Uncomment when songbird dependencies are available
// use songbird_core::service::{ServiceCapability as SongbirdCapability, ServiceInfo};
// use songbird_discovery::client::DiscoveryClient;
// use songbird_registry::service::ServiceRegistry;

// Temporary mock implementations for songbird types
#[derive(Debug, Clone)]
pub enum SongbirdCapability {
    Authentication { methods: Vec<String> },
    Authorization { features: Vec<String> },
    Security { level: String, features: Vec<String> },
    Custom { name: String, metadata: HashMap<String, Value> },
}

#[derive(Debug, Clone)]
pub struct ServiceInfo {
    pub id: String,
    pub name: String,
    pub capabilities: Vec<SongbirdCapability>,
}

#[derive(Debug)]
pub struct DiscoveryClient;

impl DiscoveryClient {
    pub fn new(_config: &str) -> Self { Self }
    pub async fn discover_services(&self) -> Result<Vec<ServiceInfo>, PrimalError> {
        Ok(vec![])
    }
}

#[derive(Debug)]
pub struct ServiceRegistry;

impl ServiceRegistry {
    pub fn new(_config: &str) -> Self { Self }
    pub async fn register_service(&self, _service: &ServiceInfo) -> Result<(), PrimalError> {
        Ok(())
    }
}

use crate::error::PrimalError;
use crate::universal::{
    ServiceCapability, ServiceEndpoint, UniversalSecurityProvider, UniversalSecuritySession,
    UniversalServiceRegistration,
};

/// Universal Security Adapter powered by Songbird Service Mesh
///
/// Leverages Songbird's proven service mesh infrastructure for discovery
/// while focusing Squirrel on MCP protocol and AI coordination strengths.
#[derive(Debug)]
pub struct UniversalSecurityAdapter {
    /// Songbird service registry client
    songbird_registry: Arc<ServiceRegistry>,

    /// Songbird discovery client for real-time service discovery
    songbird_discovery: Arc<DiscoveryClient>,

    /// Currently active security provider (discovered via Songbird)
    active_provider: Arc<RwLock<Option<DiscoveredSecurityProvider>>>,

    /// Required security capabilities for our MCP/AI use case
    required_capabilities: Vec<SecurityCapabilityRequirement>,

    /// Active security sessions (managed by Squirrel)
    active_sessions: Arc<RwLock<HashMap<String, UniversalSecuritySession>>>,

    /// Adapter configuration
    config: UniversalSecurityConfig,

    /// Fallback security implementation (Squirrel's focus area)
    fallback_provider: Arc<dyn SecurityProvider + Send + Sync>,
}

/// Discovered security provider from Songbird service mesh
#[derive(Debug, Clone)]
pub struct DiscoveredSecurityProvider {
    /// Songbird service information
    pub service_info: ServiceInfo,

    /// Capability match score for our MCP/AI requirements
    pub capability_score: f64,

    /// Security level provided
    pub security_level: SecurityLevel,

    /// Available authentication methods
    pub auth_methods: Vec<String>,

    /// Available authorization features
    pub auth_features: Vec<String>,

    /// Provider health status (from Songbird)
    pub health_status: ProviderHealthStatus,
}

/// Security capability requirement for MCP/AI coordination
#[derive(Debug, Clone)]
pub struct SecurityCapabilityRequirement {
    /// Capability type required for MCP/AI workflows
    pub capability_type: String,

    /// Minimum features required for AI coordination
    pub required_features: Vec<String>,

    /// Priority weight for MCP protocol needs (0.0 to 1.0)
    pub weight: f64,

    /// Is this capability mandatory for MCP operations?
    pub mandatory: bool,
}

/// Security level classification
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecurityLevel {
    Basic,
    Standard,
    Enhanced,
    Enterprise,
    MilitaryGrade,
}

/// Provider health status
#[derive(Debug, Clone, PartialEq)]
pub enum ProviderHealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Configuration for universal security adapter
#[derive(Debug, Clone)]
pub struct UniversalSecurityConfig {
    /// Enable automatic provider discovery via Songbird
    pub enable_songbird_discovery: bool,

    /// Minimum security level required for MCP operations
    pub minimum_security_level: SecurityLevel,

    /// Refresh interval for Songbird provider discovery (seconds)
    pub discovery_refresh_interval: u64,

    /// Enable fallback to Squirrel's internal security
    pub enable_fallback: bool,

    /// Maximum session duration for MCP sessions (minutes)
    pub max_session_duration: u32,

    /// Songbird health check interval (seconds)
    pub health_check_interval: u64,

    /// Focus on MCP protocol security requirements
    pub mcp_protocol_focus: bool,

    /// Enable AI-enhanced security decisions
    pub enable_ai_security: bool,
}

impl Default for UniversalSecurityConfig {
    fn default() -> Self {
        Self {
            enable_songbird_discovery: true,
            minimum_security_level: SecurityLevel::Standard,
            discovery_refresh_interval: 300, // 5 minutes
            enable_fallback: true,
            max_session_duration: 480, // 8 hours
            health_check_interval: 60, // 1 minute
            mcp_protocol_focus: true,  // Squirrel's specialty
            enable_ai_security: true,  // AI-first design
        }
    }
}

impl UniversalSecurityAdapter {
    /// Create new universal security adapter powered by Songbird service mesh
    pub async fn new(
        songbird_registry: Arc<ServiceRegistry>,
        songbird_discovery: Arc<DiscoveryClient>,
        config: UniversalSecurityConfig,
    ) -> Result<Self, PrimalError> {
        info!("🎵 Initializing Universal Security Adapter powered by Songbird Service Mesh");
        info!("🐿️ Focusing Squirrel on MCP protocol and AI coordination while leveraging Songbird network effects");

        // Define MCP/AI-focused security requirements
        let required_capabilities = vec![
            SecurityCapabilityRequirement {
                capability_type: "mcp_authentication".to_string(),
                required_features: vec![
                    "mcp_session_management".to_string(),
                    "ai_credential_validation".to_string(),
                    "context_aware_auth".to_string(),
                ],
                weight: 1.0,
                mandatory: true,
            },
            SecurityCapabilityRequirement {
                capability_type: "ai_authorization".to_string(),
                required_features: vec![
                    "ai_permission_routing".to_string(),
                    "context_based_access".to_string(),
                    "intelligent_authorization".to_string(),
                ],
                weight: 0.9,
                mandatory: true,
            },
            SecurityCapabilityRequirement {
                capability_type: "enterprise_security".to_string(),
                required_features: vec![
                    "cryptographic_proofs".to_string(),
                    "cross_node_trust".to_string(),
                    "genetic_security".to_string(),
                ],
                weight: 0.7,
                mandatory: false, // BearDog's specialty if available
            },
            SecurityCapabilityRequirement {
                capability_type: "session_intelligence".to_string(),
                required_features: vec![
                    "ai_session_optimization".to_string(),
                    "predictive_security".to_string(),
                    "adaptive_permissions".to_string(),
                ],
                weight: 0.6,
                mandatory: false, // AI enhancement
            },
        ];

        // Create Squirrel's MCP-focused fallback security provider
        let fallback_provider = Arc::new(SquirrelMcpSecurityProvider::new());

        let adapter = Self {
            songbird_registry,
            songbird_discovery,
            active_provider: Arc::new(RwLock::new(None)),
            required_capabilities,
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            config,
            fallback_provider,
        };

        // Perform initial provider discovery via Songbird
        adapter.discover_security_providers_via_songbird().await?;

        // Start background tasks for Songbird integration
        adapter.start_songbird_integration_tasks().await;

        Ok(adapter)
    }

    /// Discover available security providers via Songbird service mesh
    pub async fn discover_security_providers_via_songbird(
        &self,
    ) -> Result<Vec<DiscoveredSecurityProvider>, PrimalError> {
        info!("🔍 Discovering security providers via Songbird service mesh...");
        info!("🎯 Focusing on MCP protocol and AI coordination requirements");

        // Query Songbird registry for security-capable services
        let security_services = self
            .songbird_registry
            .get_services_by_capability("security")
            .await
            .map_err(|e| PrimalError::ServiceError(format!("Songbird registry error: {}", e)))?;

        let mut discovered_providers = Vec::new();

        for service_info in security_services {
            // Analyze service capabilities against our MCP/AI requirements
            let capability_score = self.calculate_mcp_capability_score(&service_info).await;

            if capability_score > 0.0 {
                let provider = DiscoveredSecurityProvider {
                    service_info: service_info.clone(),
                    capability_score,
                    security_level: self
                        .determine_security_level_from_songbird(&service_info)
                        .await,
                    auth_methods: self.extract_auth_methods_from_songbird(&service_info).await,
                    auth_features: self
                        .extract_auth_features_from_songbird(&service_info)
                        .await,
                    health_status: self.get_provider_health_from_songbird(&service_info).await,
                };

                info!(
                    "📋 Discovered via Songbird: {} (MCP score: {:.2}, level: {:?})",
                    service_info.name, capability_score, provider.security_level
                );

                discovered_providers.push(provider);
            }
        }

        // Sort by MCP/AI capability score and security level
        discovered_providers.sort_by(|a, b| {
            b.capability_score
                .partial_cmp(&a.capability_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.security_level.cmp(&a.security_level))
        });

        // Select the best provider for MCP/AI operations
        if let Some(best_provider) = discovered_providers.first() {
            if best_provider.security_level >= self.config.minimum_security_level {
                info!(
                    "🎯 Selected optimal security provider via Songbird: {} with MCP score {:.2}",
                    best_provider.service_info.name, best_provider.capability_score
                );

                // Notify Songbird of our selection for network effects
                self.notify_songbird_of_selection(&best_provider).await?;

                // Update active provider
                *self.active_provider.write().await = Some(best_provider.clone());
            } else {
                warn!(
                    "⚠️ Best available provider ({:?}) doesn't meet minimum security level ({:?}) for MCP operations",
                    best_provider.security_level, self.config.minimum_security_level
                );

                if self.config.enable_fallback {
                    info!("🔄 Falling back to Squirrel's MCP-focused internal security provider");
                    *self.active_provider.write().await = None; // Use Squirrel fallback
                }
            }
        } else {
            warn!("⚠️ No security providers discovered via Songbird! Using Squirrel MCP fallback");

            if self.config.enable_fallback {
                info!("🐿️ Using Squirrel's MCP-focused internal security provider");
                *self.active_provider.write().await = None; // Use Squirrel fallback
            } else {
                return Err(PrimalError::InitializationFailed(
                    "No suitable security providers found via Songbird and fallback disabled"
                        .to_string(),
                ));
            }
        }

        Ok(discovered_providers)
    }

    /// Calculate MCP/AI capability score for a Songbird service
    async fn calculate_mcp_capability_score(&self, service_info: &ServiceInfo) -> f64 {
        let mut total_score = 0.0;
        let mut total_weight = 0.0;

        for requirement in &self.required_capabilities {
            total_weight += requirement.weight;

            // Check if Songbird service has this capability for MCP/AI operations
            let capability_match = service_info
                .capabilities
                .iter()
                .any(|cap| self.matches_mcp_capability(cap, requirement));

            if capability_match {
                total_score += requirement.weight;
            } else if requirement.mandatory {
                // Mandatory MCP capability missing - return 0
                return 0.0;
            }
        }

        if total_weight > 0.0 {
            total_score / total_weight
        } else {
            0.0
        }
    }

    /// Check if a Songbird service capability matches our MCP requirement
    fn matches_mcp_capability(
        &self,
        songbird_cap: &SongbirdCapability,
        requirement: &SecurityCapabilityRequirement,
    ) -> bool {
        // Map Songbird capabilities to our MCP/AI requirements
        match (songbird_cap, requirement.capability_type.as_str()) {
            (SongbirdCapability::Security { level, features }, "enterprise_security") => {
                level == "enterprise"
                    && requirement
                        .required_features
                        .iter()
                        .all(|feature| features.contains(feature))
            }
            (SongbirdCapability::Authentication { methods }, "mcp_authentication") => requirement
                .required_features
                .iter()
                .any(|feature| methods.contains(feature)),
            (SongbirdCapability::Custom { name, metadata }, _) => {
                name == &requirement.capability_type
            }
            _ => false,
        }
    }

    /// Notify Songbird of our provider selection for network effects
    async fn notify_songbird_of_selection(
        &self,
        provider: &DiscoveredSecurityProvider,
    ) -> Result<(), PrimalError> {
        info!("📡 Notifying Songbird of security provider selection for network effects");

        // In production, this would use Songbird's client registration API
        // This enables network effects - other services can discover our choice
        // and potentially optimize their own selections

        let selection_metadata = serde_json::json!({
            "selector_service": "squirrel",
            "selected_provider": provider.service_info.name,
            "selection_reason": "mcp_ai_optimization",
            "capability_score": provider.capability_score,
            "security_level": format!("{:?}", provider.security_level),
        });

        // This would be: self.songbird_registry.notify_selection(selection_metadata).await?;
        debug!("Would notify Songbird: {}", selection_metadata);

        Ok(())
    }

    /// Start background tasks for Songbird integration
    async fn start_songbird_integration_tasks(&self) {
        info!("🎵 Starting Songbird integration background tasks");

        // Start Songbird service discovery refresh task
        let adapter_weak = Arc::downgrade(&Arc::new(self.clone()));
        let discovery_interval = self.config.discovery_refresh_interval;

        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(tokio::time::Duration::from_secs(discovery_interval));

            loop {
                interval.tick().await;

                if let Some(adapter) = adapter_weak.upgrade() {
                    if let Err(e) = adapter.discover_security_providers_via_songbird().await {
                        warn!("Songbird provider discovery refresh failed: {}", e);
                    }
                } else {
                    break; // Adapter dropped
                }
            }
        });

        // Start Songbird health monitoring task
        let adapter_weak = Arc::downgrade(&Arc::new(self.clone()));
        let health_interval = self.config.health_check_interval;

        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(tokio::time::Duration::from_secs(health_interval));

            loop {
                interval.tick().await;

                if let Some(adapter) = adapter_weak.upgrade() {
                    if let Err(e) = adapter.check_songbird_provider_health().await {
                        warn!("Songbird provider health check failed: {}", e);
                    }
                } else {
                    break; // Adapter dropped
                }
            }
        });
    }

    /// Check health of active provider via Songbird
    async fn check_songbird_provider_health(&self) -> Result<(), PrimalError> {
        let provider = self.active_provider.read().await;

        if let Some(ref active) = *provider {
            // Query health via Songbird service mesh
            let health = self
                .songbird_registry
                .get_service_health(&active.service_info.id)
                .await
                .map_err(|e| {
                    PrimalError::ServiceError(format!("Songbird health check error: {}", e))
                })?;

            if !health.is_healthy {
                warn!(
                    "Active provider {} health degraded via Songbird: {:?}",
                    active.service_info.name, health
                );

                // Trigger rediscovery via Songbird if provider is unhealthy
                if health.is_critical {
                    info!("Triggering Songbird provider rediscovery due to critical health");
                    drop(provider); // Release read lock
                    self.discover_security_providers_via_songbird().await?;
                }
            }
        }

        Ok(())
    }

    // Helper methods for extracting Songbird service information
    async fn determine_security_level_from_songbird(
        &self,
        service_info: &ServiceInfo,
    ) -> SecurityLevel {
        // Extract security level from Songbird service metadata
        service_info
            .metadata
            .get("security_level")
            .and_then(|v| v.as_str())
            .map(|level| match level {
                "basic" => SecurityLevel::Basic,
                "standard" => SecurityLevel::Standard,
                "enhanced" => SecurityLevel::Enhanced,
                "enterprise" => SecurityLevel::Enterprise,
                "military_grade" => SecurityLevel::MilitaryGrade,
                _ => SecurityLevel::Standard,
            })
            .unwrap_or(SecurityLevel::Standard)
    }

    async fn extract_auth_methods_from_songbird(&self, service_info: &ServiceInfo) -> Vec<String> {
        // Extract auth methods from Songbird service capabilities
        service_info
            .capabilities
            .iter()
            .filter_map(|cap| match cap {
                SongbirdCapability::Authentication { methods } => Some(methods.clone()),
                _ => None,
            })
            .flatten()
            .collect()
    }

    async fn extract_auth_features_from_songbird(&self, service_info: &ServiceInfo) -> Vec<String> {
        // Extract auth features from Songbird service capabilities
        service_info
            .capabilities
            .iter()
            .filter_map(|cap| match cap {
                SongbirdCapability::Authorization { features } => Some(features.clone()),
                _ => None,
            })
            .flatten()
            .collect()
    }

    async fn get_provider_health_from_songbird(
        &self,
        service_info: &ServiceInfo,
    ) -> ProviderHealthStatus {
        // Get health status from Songbird service registry
        match self
            .songbird_registry
            .get_service_health(&service_info.id)
            .await
        {
            Ok(health) => {
                if health.is_healthy {
                    ProviderHealthStatus::Healthy
                } else if health.is_critical {
                    ProviderHealthStatus::Unhealthy
                } else {
                    ProviderHealthStatus::Degraded
                }
            }
            Err(_) => ProviderHealthStatus::Unknown,
        }
    }
}

/// Squirrel's MCP-focused fallback security provider
///
/// This focuses on what Squirrel does best: MCP protocol and AI coordination
#[derive(Debug)]
pub struct SquirrelMcpSecurityProvider;

impl SquirrelMcpSecurityProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl SecurityProvider for SquirrelMcpSecurityProvider {
    async fn authenticate(
        &self,
        credentials: Value,
    ) -> Result<UniversalSecuritySession, PrimalError> {
        info!("🐿️ Using Squirrel's MCP-focused authentication");

        // Focus on MCP protocol authentication needs
        let user_id = credentials
            .get("user_id")
            .and_then(|v| v.as_str())
            .unwrap_or("mcp_anonymous");

        let session_id = Uuid::new_v4().to_string();

        // Create MCP-optimized session
        let mut metadata = HashMap::new();
        metadata.insert("provider".to_string(), "squirrel_mcp_internal".to_string());
        metadata.insert("security_level".to_string(), "mcp_optimized".to_string());
        metadata.insert("mcp_protocol_version".to_string(), "2024".to_string());
        metadata.insert("ai_coordination_enabled".to_string(), "true".to_string());

        // Add MCP-specific session metadata
        if let Some(mcp_version) = credentials.get("mcp_version") {
            metadata.insert("client_mcp_version".to_string(), mcp_version.to_string());
        }
        if let Some(ai_capabilities) = credentials.get("ai_capabilities") {
            metadata.insert(
                "client_ai_capabilities".to_string(),
                ai_capabilities.to_string(),
            );
        }

        Ok(UniversalSecuritySession {
            session_id,
            user_id: Some(user_id.to_string()),
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::minutes(120), // MCP sessions typically shorter
            metadata,
        })
    }

    async fn authorize(
        &self,
        session_id: &str,
        resource: &str,
        action: &str,
    ) -> Result<bool, PrimalError> {
        debug!(
            "🐿️ Squirrel MCP authorization: session={}, resource={}, action={}",
            session_id, resource, action
        );

        // MCP-focused authorization logic
        // Allow most MCP protocol operations, be cautious with admin functions
        let authorized = !resource.contains("admin") && !action.contains("system_modify");

        Ok(authorized)
    }
}

/// Trait for security providers (implemented by discovered providers and Squirrel fallback)
#[async_trait]
pub trait SecurityProvider {
    async fn authenticate(
        &self,
        credentials: Value,
    ) -> Result<UniversalSecuritySession, PrimalError>;
    async fn authorize(
        &self,
        session_id: &str,
        resource: &str,
        action: &str,
    ) -> Result<bool, PrimalError>;
}

// Clone implementation for UniversalSecurityAdapter
impl Clone for UniversalSecurityAdapter {
    fn clone(&self) -> Self {
        Self {
            songbird_registry: Arc::clone(&self.songbird_registry),
            songbird_discovery: Arc::clone(&self.songbird_discovery),
            active_provider: Arc::clone(&self.active_provider),
            required_capabilities: self.required_capabilities.clone(),
            active_sessions: Arc::clone(&self.active_sessions),
            config: self.config.clone(),
            fallback_provider: Arc::clone(&self.fallback_provider),
        }
    }
}

#[async_trait]
impl UniversalSecurityProvider for UniversalSecurityAdapter {
    type Session = UniversalSecuritySession;
    type Error = PrimalError;

    async fn authenticate(&self, credentials: Value) -> Result<Self::Session, Self::Error> {
        debug!("🔑 Universal authentication request via capability-based provider");

        let provider = self.active_provider.read().await;

        if let Some(ref discovered_provider) = *provider {
            info!(
                "🎯 Using discovered security provider: {} (capability score: {:.2})",
                discovered_provider.service_info.name, discovered_provider.capability_score
            );

            // Create dynamic security client based on discovered provider endpoints
            let auth_endpoint = discovered_provider
                .service_info
                .endpoints
                .iter()
                .find(|ep| ep.name == "authenticate")
                .ok_or_else(|| {
                    PrimalError::ServiceError(
                        "No authenticate endpoint found in discovered provider".to_string(),
                    )
                })?;

            // In production, this would make actual HTTP/RPC calls to the discovered provider
            // For now, simulate successful authentication based on provider capabilities
            let session = self
                .create_session_for_provider(&discovered_provider, credentials)
                .await?;

            // Store session
            let session_id = session.session_id.clone();
            self.active_sessions
                .write()
                .await
                .insert(session_id, session.clone());

            info!("✅ Authentication successful via discovered provider");
            Ok(session)
        } else {
            info!("🔄 No discovered provider available, using fallback security");

            if !self.config.enable_fallback {
                return Err(PrimalError::ServiceError(
                    "No security provider available and fallback disabled".to_string(),
                ));
            }

            let session = self.fallback_provider.authenticate(credentials).await?;

            // Store session
            let session_id = session.session_id.clone();
            self.active_sessions
                .write()
                .await
                .insert(session_id, session.clone());

            Ok(session)
        }
    }

    async fn authorize(
        &self,
        session_id: &str,
        resource: &str,
        action: &str,
    ) -> Result<bool, Self::Error> {
        debug!(
            "🔐 Universal authorization check: session={}, resource={}, action={}",
            session_id, resource, action
        );

        // Check if session exists
        let sessions = self.active_sessions.read().await;
        if !sessions.contains_key(session_id) {
            warn!("❌ Authorization denied: invalid session {}", session_id);
            return Ok(false);
        }

        let provider = self.active_provider.read().await;

        if let Some(ref discovered_provider) = *provider {
            // Use discovered provider for authorization
            let authorized = self
                .authorize_with_provider(&discovered_provider, session_id, resource, action)
                .await?;

            debug!(
                "🎯 Authorization via provider {}: {}",
                discovered_provider.service_info.name,
                if authorized {
                    "✅ GRANTED"
                } else {
                    "❌ DENIED"
                }
            );

            Ok(authorized)
        } else {
            // Use fallback authorization
            let authorized = self
                .fallback_provider
                .authorize(session_id, resource, action)
                .await?;

            debug!(
                "🔄 Fallback authorization: {}",
                if authorized {
                    "✅ GRANTED"
                } else {
                    "❌ DENIED"
                }
            );

            Ok(authorized)
        }
    }

    async fn get_session(&self, session_id: &str) -> Result<Option<Self::Session>, Self::Error> {
        let sessions = self.active_sessions.read().await;
        Ok(sessions.get(session_id).cloned())
    }

    async fn revoke_session(&self, session_id: &str) -> Result<(), Self::Error> {
        let mut sessions = self.active_sessions.write().await;
        if sessions.remove(session_id).is_some() {
            info!(
                "🗑️ Session {} revoked via universal security adapter",
                session_id
            );
        } else {
            warn!(
                "⚠️ Attempted to revoke non-existent session: {}",
                session_id
            );
        }
        Ok(())
    }

    async fn get_capabilities(&self) -> Result<Vec<ServiceCapability>, Self::Error> {
        let provider = self.active_provider.read().await;

        if let Some(ref discovered_provider) = *provider {
            Ok(discovered_provider
                .service_info
                .capabilities
                .iter()
                .map(|cap| match cap {
                    SongbirdCapability::Authentication { methods } => {
                        ServiceCapability::Authentication {
                            methods: methods.clone(),
                        }
                    }
                    SongbirdCapability::Authorization { features } => {
                        ServiceCapability::Authorization {
                            features: features.clone(),
                        }
                    }
                    SongbirdCapability::Security { level, features } => {
                        if features.contains("cryptographic_proofs") {
                            ServiceCapability::Security {
                                level: SecurityLevel::Enterprise,
                                compliance: None,
                            }
                        } else if features.contains("cross_node_trust") {
                            ServiceCapability::Security {
                                level: SecurityLevel::Enhanced,
                                compliance: None,
                            }
                        } else {
                            ServiceCapability::Security {
                                level: SecurityLevel::Standard,
                                compliance: None,
                            }
                        }
                    }
                    SongbirdCapability::Custom { name, metadata } => ServiceCapability::Custom {
                        capability_type: name.clone(),
                        metadata: metadata.clone(),
                    },
                })
                .collect())
        } else {
            // Return basic fallback capabilities
            Ok(vec![
                ServiceCapability::Authentication {
                    methods: vec!["basic".to_string(), "session".to_string()],
                },
                ServiceCapability::Authorization {
                    features: vec!["resource_access".to_string()],
                },
            ])
        }
    }
}

impl UniversalSecurityAdapter {
    /// Create session for discovered provider
    async fn create_session_for_provider(
        &self,
        provider: &DiscoveredSecurityProvider,
        credentials: Value,
    ) -> Result<UniversalSecuritySession, PrimalError> {
        let user_id = credentials
            .get("user_id")
            .and_then(|v| v.as_str())
            .unwrap_or("anonymous");

        let session_id = Uuid::new_v4().to_string();

        let mut metadata = HashMap::new();
        metadata.insert("provider".to_string(), provider.service_info.name.clone());
        metadata.insert(
            "provider_id".to_string(),
            provider.service_info.id.to_string(),
        );
        metadata.insert(
            "security_level".to_string(),
            format!("{:?}", provider.security_level),
        );
        metadata.insert(
            "capability_score".to_string(),
            provider.capability_score.to_string(),
        );
        metadata.insert("auth_methods".to_string(), provider.auth_methods.join(","));

        // Add any provider-specific metadata
        for (key, value) in &provider.service_info.metadata {
            metadata.insert(format!("provider_{}", key), value.clone());
        }

        Ok(UniversalSecuritySession {
            session_id,
            user_id: Some(user_id.to_string()),
            created_at: Utc::now(),
            expires_at: Utc::now()
                + chrono::Duration::minutes(self.config.max_session_duration as i64),
            metadata,
        })
    }

    /// Authorize with discovered provider
    async fn authorize_with_provider(
        &self,
        provider: &DiscoveredSecurityProvider,
        session_id: &str,
        resource: &str,
        action: &str,
    ) -> Result<bool, PrimalError> {
        // In production, this would make actual authorization calls to the provider's endpoints
        // For now, simulate authorization based on provider capabilities and security level

        let has_auth_capability = provider
            .service_info
            .capabilities
            .iter()
            .any(|cap| matches!(cap, SongbirdCapability::Authorization { .. }));

        if !has_auth_capability {
            warn!(
                "Provider {} lacks authorization capability",
                provider.service_info.name
            );
            return Ok(false);
        }

        // Simulate more restrictive authorization for higher security levels
        let authorized = match provider.security_level {
            SecurityLevel::Enterprise | SecurityLevel::MilitaryGrade => {
                // More restrictive for high security
                !resource.contains("admin") && !action.contains("delete")
            }
            SecurityLevel::Enhanced => {
                // Moderate restrictions
                !action.contains("delete")
            }
            _ => {
                // Basic authorization allows most operations
                true
            }
        };

        Ok(authorized)
    }
}
