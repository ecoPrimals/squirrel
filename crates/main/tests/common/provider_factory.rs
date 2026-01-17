//! Provider Factory for Tests
//!
//! Creates properly configured SquirrelPrimalProvider instances for testing
//! with all required dependencies initialized correctly.
//!
//! ## Design Principles
//! - Zero hardcoding: All configuration from environment or defaults
//! - Capability-based: Uses real capability registry
//! - Modern Rust: Proper error handling, no unwrap() in production paths
//! - Deep initialization: All dependencies properly constructed

use std::collections::HashMap;
use std::sync::Arc;

use squirrel::{
    capability_registry::{CapabilityRegistry, CapabilityRegistryConfig},
    ecosystem::EcosystemManager,
    error::PrimalError,
    primal_provider::SquirrelPrimalProvider,
    session::{SessionManager, SessionMetadata},
    universal::PrimalContext,
    universal_adapter_v2::UniversalAdapterV2,
    MetricsCollector,
};
use squirrel_mcp_config::EcosystemConfig;

/// Test session manager implementation
struct TestSessionManager;

#[async_trait::async_trait]
impl SessionManager for TestSessionManager {
    async fn create_session(&self, _client_info: Option<String>) -> Result<String, PrimalError> {
        Ok("test-session-id".to_string())
    }

    async fn get_session_metadata(&self, session_id: &str) -> Result<SessionMetadata, PrimalError> {
        Ok(SessionMetadata {
            session_id: session_id.to_string(),
            client_info: None,
            created_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            capabilities: Vec::new(),
        })
    }

    async fn update_session_data(
        &self,
        _session_id: &str,
        _data: HashMap<String, serde_json::Value>,
    ) -> Result<(), PrimalError> {
        Ok(())
    }

    async fn terminate_session(&self, _session_id: &str) -> Result<(), PrimalError> {
        Ok(())
    }
}

/// Builder for creating test SquirrelPrimalProvider instances
///
/// # Example
/// ```no_run
/// let provider = ProviderFactory::new()
///     .with_instance_id("test-instance")
///     .build()
///     .await?;
/// ```
pub struct ProviderFactory {
    instance_id: Option<String>,
    config: Option<EcosystemConfig>,
    context: Option<PrimalContext>,
}

impl ProviderFactory {
    /// Create a new provider factory with defaults
    pub fn new() -> Self {
        Self {
            instance_id: None,
            config: None,
            context: None,
        }
    }

    /// Set custom instance ID
    pub fn with_instance_id(mut self, id: impl Into<String>) -> Self {
        self.instance_id = Some(id.into());
        self
    }

    /// Set custom configuration
    pub fn with_config(mut self, config: EcosystemConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Set custom context
    pub fn with_context(mut self, context: PrimalContext) -> Self {
        self.context = Some(context);
        self
    }

    /// Build the provider with all dependencies
    pub async fn build(self) -> Result<SquirrelPrimalProvider, Box<dyn std::error::Error>> {
        let instance_id = self
            .instance_id
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        let config = self.config.unwrap_or_else(EcosystemConfig::default);

        // Create capability registry (no hardcoded knowledge)
        let registry_config = CapabilityRegistryConfig::default();
        let capability_registry = Arc::new(CapabilityRegistry::new(registry_config));

        // Create metrics collector
        let metrics_collector = Arc::new(MetricsCollector::new());

        // Create ecosystem manager (capability-based discovery)
        // Convert McpEcosystemConfig to squirrel::EcosystemConfig
        let ecosystem_config = squirrel::ecosystem::EcosystemConfig {
            biome_id: config.ecosystem.biome_id.clone(),
            primal_type: squirrel::ecosystem::EcosystemPrimalType::Intelligence,
            discovery_config: squirrel::ecosystem::DiscoveryConfig {
                enabled: true,
                mechanism: squirrel::ecosystem::DiscoveryMechanism::FileSystem,
                registry_path: None,
            },
        };
        let ecosystem_manager = Arc::new(EcosystemManager::new(
            ecosystem_config,
            metrics_collector.clone(),
        ));

        // Create universal adapter (infant primal - awakens with zero hardcoded knowledge)
        let adapter = UniversalAdapterV2::awaken()
            .await
            .map_err(|e| format!("Failed to awaken universal adapter: {:?}", e))?;

        // Create session manager
        let session_manager: Arc<dyn SessionManager> = Arc::new(TestSessionManager);

        // Construct provider with modern pattern
        Ok(SquirrelPrimalProvider::new(
            instance_id,
            config,
            adapter,
            ecosystem_manager,
            capability_registry,
            session_manager,
        ))
    }
}

impl Default for ProviderFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Quick helper to create a default test provider
pub async fn create_test_provider() -> Result<SquirrelPrimalProvider, Box<dyn std::error::Error>> {
    ProviderFactory::new().build().await
}

/// Create test provider with custom config
pub async fn create_test_provider_with_config(
    config: EcosystemConfig,
) -> Result<SquirrelPrimalProvider, Box<dyn std::error::Error>> {
    ProviderFactory::new().with_config(config).build().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_provider_factory_creates_valid_instance() {
        let provider = create_test_provider()
            .await
            .expect("Should create provider");

        // Verify it's properly initialized
        assert!(!provider.instance_id().is_empty());
    }

    #[tokio::test]
    async fn test_provider_factory_with_custom_id() {
        let provider = ProviderFactory::new()
            .with_instance_id("custom-test-id")
            .build()
            .await
            .expect("Should create provider");

        assert_eq!(provider.instance_id(), "custom-test-id");
    }
}
