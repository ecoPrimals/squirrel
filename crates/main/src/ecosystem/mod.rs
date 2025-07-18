// Re-export registry manager types
pub mod registry_manager;
pub mod registry;
pub use registry_manager::*;
pub use registry::{EcosystemRegistryEvent, EcosystemRegistryConfig}; 

pub struct EcosystemServiceRegistration {
    /// Unique service identifier: "primal-squirrel-{instance}"
    pub service_id: String,

    /// Primal type from standardized enum
    pub primal_type: EcosystemPrimalType,

    /// Associated biome identifier (if applicable)
    pub biome_id: Option<String>,

    /// Service name
    pub name: String,

    /// Service version
    pub version: String,

    /// Service description
    pub description: String,

    /// Service capabilities (standardized format)
    pub capabilities: ServiceCapabilities,

    /// API endpoints (standardized format)
    pub endpoints: ServiceEndpoints,

    /// Service dependencies
    pub dependencies: Vec<String>,

    /// Resource requirements
    pub resource_requirements: ResourceSpec,

    /// Security configuration
    pub security_config: SecurityConfig,

    /// Health check configuration
    pub health_check: HealthCheckConfig,

    /// Additional metadata
    pub metadata: HashMap<String, String>,

    /// Service tags
    pub tags: Vec<String>,

    /// Primal provider reference
    pub primal_provider: Option<String>,

    /// Registration timestamp
    pub registered_at: DateTime<Utc>,
} 