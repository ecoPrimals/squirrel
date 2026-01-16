//! Agnostic Compute Provider Trait - Infant Primal Pattern
//!
//! **Philosophy**: Zero vendor lock-in for compute resources.
//!
//! Following the infant primal pattern:
//! - No hardcoded orchestrators (k8s, Docker, etc.)
//! - Runtime provider selection
//! - Graceful capability negotiation
//!
//! # Example
//!
//! ```rust,ignore
//! // ❌ BAD: Vendor-specific
//! let k8s = KubernetesClient::new("https://k8s-api:6443");
//! let pod = k8s.create_pod(spec).await?;
//!
//! // ✅ GOOD: Agnostic trait
//! let compute: Box<dyn ComputeProvider> = detect_compute_provider().await?;
//! let job = compute.execute_workload(spec).await?;
//! ```
//!
//! # Implementations
//!
//! Any compute platform can implement this trait:
//! - Kubernetes (container orchestration)
//! - Docker Swarm
//! - Nomad (HashiCorp)
//! - AWS ECS/Fargate
//! - Local process execution
//! - Toadstool (ecoPrimals compute primal)

use crate::compute_client::types::{ComputeCapabilityType, ResourceRequirements};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Result type for compute operations
pub type ComputeResult<T> = Result<T, ComputeProviderError>;

/// Compute provider errors
#[derive(Debug, thiserror::Error)]
pub enum ComputeProviderError {
    #[error("Provider not available: {0}")]
    NotAvailable(String),

    #[error("Insufficient resources: {0}")]
    InsufficientResources(String),

    #[error("Workload execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Provider error: {0}")]
    ProviderError(String),

    #[error("Timeout waiting for workload")]
    Timeout,

    #[error("Workload not found: {0}")]
    NotFound(String),
}

/// Workload execution specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadExecutionSpec {
    /// Unique workload ID
    pub id: Uuid,

    /// Human-readable name
    pub name: String,

    /// Container image or executable
    pub image: String,

    /// Command to execute
    pub command: Vec<String>,

    /// Environment variables
    pub environment: HashMap<String, String>,

    /// Resource requirements
    pub resources: ResourceRequirements,

    /// Labels for discovery and routing
    pub labels: HashMap<String, String>,
}

/// Workload status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkloadStatus {
    /// Pending execution
    Pending,
    /// Currently running
    Running,
    /// Completed successfully
    Completed,
    /// Failed
    Failed,
    /// Cancelled
    Cancelled,
}

/// Workload execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadExecutionResult {
    /// Workload ID
    pub id: Uuid,

    /// Current status
    pub status: WorkloadStatus,

    /// Exit code (if completed)
    pub exit_code: Option<i32>,

    /// Logs (if available)
    pub logs: Option<String>,

    /// Execution metadata
    pub metadata: HashMap<String, String>,
}

/// Agnostic Compute Provider
///
/// Any compute platform can implement this trait to provide
/// workload execution capabilities without vendor lock-in.
///
/// # Infant Primal Pattern
///
/// Implementations should:
/// 1. Discover their own configuration from environment
/// 2. Gracefully handle resource constraints
/// 3. Provide health and capability reporting
/// 4. Support heterogeneous workload types
#[async_trait]
pub trait ComputeProvider: Send + Sync {
    /// Provider name (for logging/debugging)
    ///
    /// Examples: "kubernetes", "docker", "nomad", "toadstool", "local"
    fn provider_name(&self) -> &str;

    /// Get available compute capabilities
    ///
    /// Returns what this provider can execute.
    async fn get_capabilities(&self) -> ComputeResult<Vec<ComputeCapabilityType>>;

    /// Execute a workload
    ///
    /// Schedules and runs the specified workload.
    ///
    /// # Arguments
    ///
    /// * `spec` - Workload execution specification
    ///
    /// # Returns
    ///
    /// * `Ok(Uuid)` - Workload ID for tracking
    /// * `Err(ComputeProviderError)` - If execution cannot be started
    async fn execute_workload(&self, spec: WorkloadExecutionSpec) -> ComputeResult<Uuid>;

    /// Get workload status
    ///
    /// Returns the current status of a running or completed workload.
    async fn get_workload_status(&self, id: Uuid) -> ComputeResult<WorkloadExecutionResult>;

    /// Cancel a workload
    ///
    /// Stops a running workload.
    async fn cancel_workload(&self, id: Uuid) -> ComputeResult<()>;

    /// List all workloads
    ///
    /// Returns all workloads managed by this provider.
    async fn list_workloads(&self) -> ComputeResult<Vec<WorkloadExecutionResult>>;

    /// Health check - verify provider is available
    async fn health_check(&self) -> bool;

    /// Get provider metadata
    ///
    /// Returns additional information about this provider.
    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("provider".to_string(), self.provider_name().to_string());
        meta
    }

    /// Get available resources
    ///
    /// Returns current resource availability (CPU, memory, GPU).
    async fn get_available_resources(&self) -> ComputeResult<ResourceRequirements> {
        // Default implementation returns unlimited
        Ok(ResourceRequirements {
            cpu_cores: u32::MAX,
            memory_gb: u32::MAX,
            gpu_units: None,
            storage_gb: u32::MAX,
            max_execution_time: std::time::Duration::from_secs(3600), // 1 hour default
            network_bandwidth_mbps: None,
        })
    }
}

/// Auto-detect compute provider
///
/// Attempts to detect and create an appropriate compute provider
/// based on the runtime environment.
///
/// # Detection Order
///
/// 1. Environment variable `COMPUTE_PROVIDER_TYPE`
/// 2. Toadstool detection (ecoPrimals compute primal)
/// 3. Kubernetes detection (if `/var/run/secrets/kubernetes.io` exists)
/// 4. Docker detection (if Docker socket exists)
/// 5. Local process execution (fallback)
///
/// # Example
///
/// ```rust,ignore
/// let compute = auto_detect_compute_provider().await?;
/// println!("Using compute: {}", compute.provider_name());
/// ```
pub async fn auto_detect_compute_provider() -> ComputeResult<Box<dyn ComputeProvider>> {
    use tracing::{debug, info};

    // 1. Check environment variable
    if let Ok(provider_type) = std::env::var("COMPUTE_PROVIDER_TYPE") {
        info!("Compute provider type specified: {}", provider_type);
        return create_compute_from_type(&provider_type).await;
    }

    // 2. Detect Toadstool (ecoPrimals compute primal)
    // This would use the universal adapter to discover Toadstool
    if std::env::var("TOADSTOOL_ENDPOINT").is_ok() {
        debug!("Detected Toadstool compute primal");
        return create_compute_from_type("toadstool").await;
    }

    // 3. Detect Kubernetes
    if std::path::Path::new("/var/run/secrets/kubernetes.io").exists() {
        debug!("Detected Kubernetes environment");
        return create_compute_from_type("kubernetes").await;
    }

    // 4. Detect Docker
    if std::path::Path::new("/var/run/docker.sock").exists() {
        debug!("Detected Docker environment");
        return create_compute_from_type("docker").await;
    }

    // 5. Fall back to local execution
    debug!("No compute provider detected, using local execution");
    create_compute_from_type("local").await
}

/// Create compute provider from type string
async fn create_compute_from_type(provider_type: &str) -> ComputeResult<Box<dyn ComputeProvider>> {
    match provider_type.to_lowercase().as_str() {
        "toadstool" => Err(ComputeProviderError::NotAvailable(
            "Toadstool provider not yet implemented".to_string(),
        )),
        "kubernetes" | "k8s" => Err(ComputeProviderError::NotAvailable(
            "Kubernetes provider not yet implemented".to_string(),
        )),
        "docker" => Err(ComputeProviderError::NotAvailable(
            "Docker provider not yet implemented".to_string(),
        )),
        "nomad" => Err(ComputeProviderError::NotAvailable(
            "Nomad provider not yet implemented".to_string(),
        )),
        "local" => Err(ComputeProviderError::NotAvailable(
            "Local provider not yet implemented".to_string(),
        )),
        unknown => Err(ComputeProviderError::NotAvailable(format!(
            "Unknown provider type: {}",
            unknown
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock implementation for testing
    struct MockComputeProvider {
        name: String,
    }

    #[async_trait]
    impl ComputeProvider for MockComputeProvider {
        fn provider_name(&self) -> &str {
            &self.name
        }

        async fn get_capabilities(&self) -> ComputeResult<Vec<ComputeCapabilityType>> {
            Ok(vec![])
        }

        async fn execute_workload(&self, _spec: WorkloadExecutionSpec) -> ComputeResult<Uuid> {
            Ok(Uuid::new_v4())
        }

        async fn get_workload_status(&self, id: Uuid) -> ComputeResult<WorkloadExecutionResult> {
            Ok(WorkloadExecutionResult {
                id,
                status: WorkloadStatus::Running,
                exit_code: None,
                logs: None,
                metadata: HashMap::new(),
            })
        }

        async fn cancel_workload(&self, _id: Uuid) -> ComputeResult<()> {
            Ok(())
        }

        async fn list_workloads(&self) -> ComputeResult<Vec<WorkloadExecutionResult>> {
            Ok(vec![])
        }

        async fn health_check(&self) -> bool {
            true
        }
    }

    #[tokio::test]
    async fn test_compute_provider_trait() {
        let provider = MockComputeProvider {
            name: "test".to_string(),
        };

        assert_eq!(provider.provider_name(), "test");
        assert!(provider.health_check().await);

        let capabilities = provider.get_capabilities().await.unwrap();
        assert_eq!(capabilities.len(), 0);
    }

    #[tokio::test]
    async fn test_execute_workload() {
        let provider = MockComputeProvider {
            name: "test".to_string(),
        };

        let spec = WorkloadExecutionSpec {
            id: Uuid::new_v4(),
            name: "test-workload".to_string(),
            image: "test-image".to_string(),
            command: vec!["echo".to_string(), "hello".to_string()],
            environment: HashMap::new(),
            resources: ResourceRequirements {
                cpu_cores: 1,
                memory_gb: 1,
                gpu_units: None,
                storage_gb: 10,
                max_execution_time: std::time::Duration::from_secs(60),
                network_bandwidth_mbps: None,
            },
            labels: HashMap::new(),
        };

        let workload_id = provider.execute_workload(spec).await.unwrap();
        let status = provider.get_workload_status(workload_id).await.unwrap();
        assert_eq!(status.status, WorkloadStatus::Running);
    }
}
