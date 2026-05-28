// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

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
//! - ecoPrimals compute primal (discovered via capability routing)

use crate::compute_client::types::{ComputeCapabilityType, ResourceRequirements};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Result type for compute operations
pub type ComputeResult<T> = Result<T, ComputeProviderError>;

/// Compute provider errors
#[derive(Debug, thiserror::Error)]
pub enum ComputeProviderError {
    /// Provider is not available for the requested operation.
    #[error("Provider not available: {0}")]
    NotAvailable(String),

    /// Insufficient resources to fulfill the request.
    #[error("Insufficient resources: {0}")]
    InsufficientResources(String),

    /// Workload execution failed.
    #[error("Workload execution failed: {0}")]
    ExecutionFailed(String),

    /// Generic provider error.
    #[error("Provider error: {0}")]
    ProviderError(String),

    /// Operation timed out waiting for workload completion.
    #[error("Timeout waiting for workload")]
    Timeout,

    /// The requested workload was not found.
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
#[expect(
    async_fn_in_trait,
    reason = "Native async compute surface; use `ComputeBackend` enum instead of dyn"
)]
pub trait ComputeProvider: Send + Sync {
    /// Provider name (for logging/debugging)
    ///
    /// Examples: "kubernetes", "docker", "nomad", "remote", "local"
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

/// Auto-detect compute provider based on runtime environment.
///
/// # Detection Order
///
/// 1. `COMPUTE_PROVIDER_TYPE` env: explicit type ("local", "remote")
/// 2. `COMPUTE_SERVICE_ENDPOINT` / `COMPUTE_ENDPOINT` env:
///    resolve endpoint and create [`RemoteComputeProvider`] for IPC delegation
/// 3. Fall back to [`LocalProcessProvider`] (development only)
///
/// In composition, the compute primal is discovered via capability-based
/// resolution — no primal names are hardcoded.
pub async fn auto_detect_compute_provider() -> ComputeResult<Box<ComputeBackend>> {
    use tracing::{debug, info};
    use universal_constants::env_vars;

    // 1. Explicit provider type from environment
    if let Ok(provider_type) = std::env::var(env_vars::compute::PROVIDER_TYPE) {
        info!(provider = %provider_type, "Compute provider type specified via env");
        return create_compute_from_type(&provider_type, None).await;
    }

    // 2. Capability-based: resolve compute primal endpoint from env
    let endpoint = std::env::var(env_vars::compute::SERVICE_ENDPOINT)
        .or_else(|_| std::env::var(env_vars::compute::ENDPOINT))
        .ok();

    if let Some(ref ep) = endpoint {
        info!(endpoint = %ep, "Detected compute primal endpoint — creating remote provider");
        return create_compute_from_type("remote", Some(ep)).await;
    }

    // 3. Fall back to local execution (development mode)
    debug!("No compute endpoint detected, using local execution (dev fallback)");
    create_compute_from_type("local", None).await
}

/// Create compute provider from type string.
///
/// - `"local"` — development fallback (always available, rejects `execute_workload`)
/// - `"remote"` / `"capability"` — delegates to compute primal via JSON-RPC at `endpoint`
async fn create_compute_from_type(
    provider_type: &str,
    endpoint: Option<&str>,
) -> ComputeResult<Box<ComputeBackend>> {
    match provider_type.to_lowercase().as_str() {
        "local" => Ok(Box::new(ComputeBackend::Local(LocalProcessProvider::new()))),
        "remote" | "capability" => {
            let ep = endpoint.ok_or_else(|| {
                ComputeProviderError::NotAvailable(
                    "Remote compute requires an endpoint (set COMPUTE_ENDPOINT or COMPUTE_SERVICE_ENDPOINT)".into(),
                )
            })?;
            Ok(Box::new(ComputeBackend::Remote(
                RemoteComputeProvider::new(ep.to_string()),
            )))
        }
        other => {
            tracing::info!(
                provider = other,
                "Unknown compute type — trying as remote endpoint"
            );
            if let Some(ep) = endpoint {
                Ok(Box::new(ComputeBackend::Remote(
                    RemoteComputeProvider::new(ep.to_string()),
                )))
            } else {
                Err(ComputeProviderError::NotAvailable(format!(
                    "Provider '{other}' is not embedded and no endpoint is configured \
                     — set COMPUTE_ENDPOINT to delegate to the compute primal"
                )))
            }
        }
    }
}

/// Embedded compute backends (enum dispatch instead of `dyn ComputeProvider`).
pub enum ComputeBackend {
    /// Local in-process stub used for development.
    Local(LocalProcessProvider),
    /// Remote compute primal delegation via JSON-RPC IPC.
    Remote(RemoteComputeProvider),
}

impl ComputeProvider for ComputeBackend {
    fn provider_name(&self) -> &str {
        match self {
            Self::Local(p) => p.provider_name(),
            Self::Remote(p) => p.provider_name(),
        }
    }

    async fn get_capabilities(&self) -> ComputeResult<Vec<ComputeCapabilityType>> {
        match self {
            Self::Local(p) => p.get_capabilities().await,
            Self::Remote(p) => p.get_capabilities().await,
        }
    }

    async fn execute_workload(&self, spec: WorkloadExecutionSpec) -> ComputeResult<Uuid> {
        match self {
            Self::Local(p) => p.execute_workload(spec).await,
            Self::Remote(p) => p.execute_workload(spec).await,
        }
    }

    async fn get_workload_status(&self, id: Uuid) -> ComputeResult<WorkloadExecutionResult> {
        match self {
            Self::Local(p) => p.get_workload_status(id).await,
            Self::Remote(p) => p.get_workload_status(id).await,
        }
    }

    async fn cancel_workload(&self, id: Uuid) -> ComputeResult<()> {
        match self {
            Self::Local(p) => p.cancel_workload(id).await,
            Self::Remote(p) => p.cancel_workload(id).await,
        }
    }

    async fn list_workloads(&self) -> ComputeResult<Vec<WorkloadExecutionResult>> {
        match self {
            Self::Local(p) => p.list_workloads().await,
            Self::Remote(p) => p.list_workloads().await,
        }
    }

    async fn health_check(&self) -> bool {
        match self {
            Self::Local(p) => p.health_check().await,
            Self::Remote(p) => p.health_check().await,
        }
    }

    fn metadata(&self) -> HashMap<String, String> {
        match self {
            Self::Local(p) => p.metadata(),
            Self::Remote(p) => p.metadata(),
        }
    }

    async fn get_available_resources(&self) -> ComputeResult<ResourceRequirements> {
        match self {
            Self::Local(p) => p.get_available_resources().await,
            Self::Remote(p) => p.get_available_resources().await,
        }
    }
}

/// Minimal local-process compute provider for development and testing.
///
/// In production, compute workloads are delegated to the compute primal
/// (e.g. ToadStool) via `compute.execute` capability discovery. This
/// provider exists so that `auto_detect_compute_provider` always has a
/// usable fallback during local development.
pub struct LocalProcessProvider {
    workloads: std::sync::Mutex<HashMap<Uuid, WorkloadExecutionResult>>,
}

impl LocalProcessProvider {
    fn new() -> Self {
        Self {
            workloads: std::sync::Mutex::new(HashMap::new()),
        }
    }
}

impl ComputeProvider for LocalProcessProvider {
    fn provider_name(&self) -> &'static str {
        "local"
    }

    async fn get_capabilities(&self) -> ComputeResult<Vec<ComputeCapabilityType>> {
        Ok(vec![ComputeCapabilityType::CpuIntensive {
            cores: num_cpus(),
            memory_gb: 8,
            architecture: std::env::consts::ARCH.to_string(),
        }])
    }

    async fn execute_workload(&self, spec: WorkloadExecutionSpec) -> ComputeResult<Uuid> {
        tracing::warn!(
            name = %spec.name,
            id = %spec.id,
            "Local compute provider cannot execute workloads — use compute capability discovery to route to a real compute primal"
        );
        Err(ComputeProviderError::ProviderError(
            "local compute provider is a development fallback and cannot execute workloads — configure a compute primal via capability discovery".into(),
        ))
    }

    async fn get_workload_status(&self, id: Uuid) -> ComputeResult<WorkloadExecutionResult> {
        self.workloads
            .lock()
            .map_err(|e| ComputeProviderError::ProviderError(e.to_string()))?
            .get(&id)
            .cloned()
            .ok_or_else(|| ComputeProviderError::NotFound(id.to_string()))
    }

    async fn cancel_workload(&self, id: Uuid) -> ComputeResult<()> {
        self.workloads
            .lock()
            .map_err(|e| ComputeProviderError::ProviderError(e.to_string()))?
            .remove(&id);
        Ok(())
    }

    async fn list_workloads(&self) -> ComputeResult<Vec<WorkloadExecutionResult>> {
        Ok(self
            .workloads
            .lock()
            .map_err(|e| ComputeProviderError::ProviderError(e.to_string()))?
            .values()
            .cloned()
            .collect())
    }

    async fn health_check(&self) -> bool {
        true
    }
}

fn num_cpus() -> u32 {
    std::thread::available_parallelism()
        .map(|n| n.get() as u32)
        .unwrap_or(1)
}

/// Remote compute provider that delegates workloads to a compute primal
/// via JSON-RPC IPC over Unix socket or TCP.
///
/// Translates Squirrel's `WorkloadExecutionSpec` into the compute primal's
/// `compute.execute` JSON-RPC call (`JsonWorkloadSubmission` wire format).
pub struct RemoteComputeProvider {
    endpoint: String,
}

impl RemoteComputeProvider {
    #[must_use]
    pub const fn new(endpoint: String) -> Self {
        Self { endpoint }
    }

    /// Send a JSON-RPC request to the compute endpoint and return the result.
    async fn rpc_call(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> ComputeResult<serde_json::Value> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let request_id = Uuid::new_v4().to_string();
        let rpc_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "method": method,
            "params": params,
        });

        let request_bytes = serde_json::to_vec(&rpc_request).map_err(|e| {
            ComputeProviderError::ProviderError(format!("Failed to serialize request: {e}"))
        })?;

        let response_bytes = if self.endpoint.starts_with("unix://") {
            let socket_path = self.endpoint.strip_prefix("unix://").ok_or_else(|| {
                ComputeProviderError::ProviderError("Invalid unix:// endpoint".into())
            })?;
            let mut stream = tokio::net::UnixStream::connect(socket_path)
                .await
                .map_err(|e| {
                    ComputeProviderError::ProviderError(format!(
                        "Failed to connect to compute primal at {socket_path}: {e}"
                    ))
                })?;
            stream.write_all(&request_bytes).await.map_err(|e| {
                ComputeProviderError::ProviderError(format!("Socket write failed: {e}"))
            })?;
            stream.write_all(b"\n").await.map_err(|e| {
                ComputeProviderError::ProviderError(format!("Socket write delimiter failed: {e}"))
            })?;
            stream.shutdown().await.map_err(|e| {
                ComputeProviderError::ProviderError(format!("Socket shutdown failed: {e}"))
            })?;
            let mut buf = Vec::new();
            stream.read_to_end(&mut buf).await.map_err(|e| {
                ComputeProviderError::ProviderError(format!("Socket read failed: {e}"))
            })?;
            buf
        } else {
            // TCP endpoint (host:port or http://host:port)
            let addr = self
                .endpoint
                .strip_prefix("http://")
                .unwrap_or(&self.endpoint);
            let mut stream = tokio::net::TcpStream::connect(addr).await.map_err(|e| {
                ComputeProviderError::ProviderError(format!(
                    "Failed to connect to compute primal at {addr}: {e}"
                ))
            })?;
            stream.write_all(&request_bytes).await.map_err(|e| {
                ComputeProviderError::ProviderError(format!("TCP write failed: {e}"))
            })?;
            stream.write_all(b"\n").await.map_err(|e| {
                ComputeProviderError::ProviderError(format!("TCP write delimiter failed: {e}"))
            })?;
            stream.shutdown().await.map_err(|e| {
                ComputeProviderError::ProviderError(format!("TCP shutdown failed: {e}"))
            })?;
            let mut buf = Vec::new();
            stream.read_to_end(&mut buf).await.map_err(|e| {
                ComputeProviderError::ProviderError(format!("TCP read failed: {e}"))
            })?;
            buf
        };

        let rpc_response: serde_json::Value =
            serde_json::from_slice(&response_bytes).map_err(|e| {
                ComputeProviderError::ProviderError(format!(
                    "Failed to parse response from compute primal: {e}"
                ))
            })?;

        if let Some(err) = rpc_response.get("error") {
            let msg = err
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("unknown remote error");
            return Err(ComputeProviderError::ExecutionFailed(format!(
                "Compute primal returned error: {msg}"
            )));
        }

        rpc_response.get("result").cloned().ok_or_else(|| {
            ComputeProviderError::ProviderError(
                "Compute primal response missing 'result' field".into(),
            )
        })
    }
}

impl ComputeProvider for RemoteComputeProvider {
    #[allow(clippy::unused_self, clippy::unnecessary_literal_bound)]
    fn provider_name(&self) -> &str {
        "remote"
    }

    async fn get_capabilities(&self) -> ComputeResult<Vec<ComputeCapabilityType>> {
        let result = self
            .rpc_call("compute.capabilities", serde_json::json!({}))
            .await;

        match result {
            Ok(val) => {
                // Best-effort parse; if the shape doesn't match, return generic
                if let Ok(caps) = serde_json::from_value(val) {
                    Ok(caps)
                } else {
                    Ok(vec![ComputeCapabilityType::CpuIntensive {
                        cores: 0,
                        memory_gb: 0,
                        architecture: "remote".to_string(),
                    }])
                }
            }
            Err(_) => Ok(vec![ComputeCapabilityType::CpuIntensive {
                cores: 0,
                memory_gb: 0,
                architecture: "remote".to_string(),
            }]),
        }
    }

    async fn execute_workload(&self, spec: WorkloadExecutionSpec) -> ComputeResult<Uuid> {
        use base64::Engine;

        // Translate WorkloadExecutionSpec → toadStool's JsonWorkloadSubmission wire format
        let data_payload = serde_json::json!({
            "image": spec.image,
            "command": spec.command,
            "environment": spec.environment,
        });
        let data_b64 = base64::engine::general_purpose::STANDARD.encode(
            serde_json::to_vec(&data_payload).map_err(|e| {
                ComputeProviderError::ProviderError(format!("Failed to encode payload: {e}"))
            })?,
        );

        let params = serde_json::json!({
            "workload_id": spec.id.to_string(),
            "workload_type": spec.labels.get("workload_type").map_or("generic", String::as_str),
            "data": data_b64,
            "metadata": spec.labels,
            "priority": "Normal",
            "requirements": {
                "cpu_cores": spec.resources.cpu_cores,
                "memory_bytes": u64::from(spec.resources.memory_gb) * 1_073_741_824,
                "timeout_secs": spec.resources.max_execution_time.as_secs(),
            }
        });

        let result = self.rpc_call("compute.execute", params).await?;

        // toadStool returns workload_id in the result
        if let Some(id_str) = result.get("workload_id").and_then(|v| v.as_str()) {
            Uuid::parse_str(id_str).map_err(|e| {
                ComputeProviderError::ProviderError(format!(
                    "Compute primal returned invalid workload_id: {e}"
                ))
            })
        } else {
            // Fall back to using our own ID
            Ok(spec.id)
        }
    }

    async fn get_workload_status(&self, id: Uuid) -> ComputeResult<WorkloadExecutionResult> {
        let result = self
            .rpc_call(
                "compute.status",
                serde_json::json!({"workload_id": id.to_string()}),
            )
            .await?;

        let status_str = result
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("Pending");

        let status = match status_str {
            "Running" => WorkloadStatus::Running,
            "Completed" => WorkloadStatus::Completed,
            "Failed" => WorkloadStatus::Failed,
            "Cancelled" => WorkloadStatus::Cancelled,
            _ => WorkloadStatus::Pending,
        };

        Ok(WorkloadExecutionResult {
            id,
            status,
            exit_code: result
                .get("exit_code")
                .and_then(serde_json::Value::as_i64)
                .map(|c| c as i32),
            logs: result
                .get("logs")
                .and_then(serde_json::Value::as_str)
                .map(Into::into),
            metadata: HashMap::new(),
        })
    }

    async fn cancel_workload(&self, id: Uuid) -> ComputeResult<()> {
        self.rpc_call(
            "compute.cancel",
            serde_json::json!({"workload_id": id.to_string()}),
        )
        .await?;
        Ok(())
    }

    async fn list_workloads(&self) -> ComputeResult<Vec<WorkloadExecutionResult>> {
        let result = self.rpc_call("compute.list", serde_json::json!({})).await?;

        if let Some(arr) = result.as_array() {
            Ok(arr
                .iter()
                .filter_map(|v| {
                    let id = v
                        .get("workload_id")
                        .and_then(|i| i.as_str())
                        .and_then(|s| Uuid::parse_str(s).ok())?;
                    let status_str = v
                        .get("status")
                        .and_then(|s| s.as_str())
                        .unwrap_or("Pending");
                    let status = match status_str {
                        "Running" => WorkloadStatus::Running,
                        "Completed" => WorkloadStatus::Completed,
                        "Failed" => WorkloadStatus::Failed,
                        "Cancelled" => WorkloadStatus::Cancelled,
                        _ => WorkloadStatus::Pending,
                    };
                    Some(WorkloadExecutionResult {
                        id,
                        status,
                        exit_code: None,
                        logs: None,
                        metadata: HashMap::new(),
                    })
                })
                .collect())
        } else {
            Ok(Vec::new())
        }
    }

    async fn health_check(&self) -> bool {
        self.rpc_call("health.check", serde_json::json!({}))
            .await
            .is_ok()
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("provider".to_string(), "remote".to_string());
        meta.insert("endpoint".to_string(), self.endpoint.clone());
        meta
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "Invariant or startup failure: expect after validation"
)]
mod tests {
    use super::*;

    // Mock implementation for testing
    struct MockComputeProvider {
        name: String,
    }

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

        let capabilities = provider.get_capabilities().await.expect("should succeed");
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

        let workload_id = provider
            .execute_workload(spec)
            .await
            .expect("should succeed");
        let status = provider
            .get_workload_status(workload_id)
            .await
            .expect("should succeed");
        assert_eq!(status.status, WorkloadStatus::Running);
    }

    #[test]
    fn auto_detect_unknown_provider_type_from_env_errors() {
        temp_env::with_var("COMPUTE_PROVIDER_TYPE", Some("quantum-hypervisor"), || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("rt");
            let result = rt.block_on(auto_detect_compute_provider());
            assert!(result.is_err(), "expected err");
            let Err(e) = result else {
                unreachable!("expected err");
            };
            match e {
                ComputeProviderError::NotAvailable(msg) => {
                    assert!(
                        msg.contains("COMPUTE_ENDPOINT"),
                        "Error should guide to setting COMPUTE_ENDPOINT: {msg}"
                    );
                }
                ref other => unreachable!("unexpected {other:?}"),
            }
        });
    }

    #[test]
    fn auto_detect_local_provider_succeeds() {
        temp_env::with_var("COMPUTE_PROVIDER_TYPE", Some("local"), || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("rt");
            let provider = rt
                .block_on(auto_detect_compute_provider())
                .expect("local provider should succeed");
            assert_eq!(provider.provider_name(), "local");
        });
    }

    #[tokio::test]
    async fn local_provider_rejects_workload_execution() {
        let provider = LocalProcessProvider::new();
        let spec = WorkloadExecutionSpec {
            id: Uuid::new_v4(),
            name: "test-local".to_string(),
            image: "none".to_string(),
            command: vec!["echo".to_string()],
            environment: HashMap::new(),
            resources: ResourceRequirements {
                cpu_cores: 1,
                memory_gb: 1,
                gpu_units: None,
                storage_gb: 1,
                max_execution_time: std::time::Duration::from_secs(5),
                network_bandwidth_mbps: None,
            },
            labels: HashMap::new(),
        };
        let result = provider.execute_workload(spec).await;
        assert!(
            result.is_err(),
            "local provider should reject workloads (development fallback only)"
        );
    }

    #[tokio::test]
    async fn mock_metadata_includes_provider_key() {
        let provider = MockComputeProvider {
            name: "meta-test".to_string(),
        };
        let m = provider.metadata();
        assert_eq!(m.get("provider").map(String::as_str), Some("meta-test"));
    }

    #[tokio::test]
    async fn test_get_available_resources_default_impl() {
        let provider = MockComputeProvider {
            name: "res-test".to_string(),
        };
        let r = provider
            .get_available_resources()
            .await
            .expect("should succeed");
        assert_eq!(r.cpu_cores, u32::MAX);
    }

    #[test]
    fn auto_detect_with_compute_endpoint_creates_remote() {
        temp_env::with_vars(
            [
                ("COMPUTE_PROVIDER_TYPE", None::<&str>),
                ("COMPUTE_ENDPOINT", Some("unix:///tmp/compute.sock")),
            ],
            || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("rt");
                let provider = rt
                    .block_on(auto_detect_compute_provider())
                    .expect("remote provider creation should succeed");
                assert_eq!(provider.provider_name(), "remote");
            },
        );
    }

    #[test]
    fn auto_detect_explicit_remote_type_needs_endpoint() {
        temp_env::with_vars(
            [
                ("COMPUTE_PROVIDER_TYPE", Some("remote")),
                ("COMPUTE_ENDPOINT", None::<&str>),
                ("COMPUTE_SERVICE_ENDPOINT", None::<&str>),
                ("TOADSTOOL_ENDPOINT", None::<&str>),
            ],
            || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("rt");
                let result = rt.block_on(auto_detect_compute_provider());
                assert!(result.is_err(), "remote type without endpoint should error");
            },
        );
    }

    #[tokio::test]
    async fn remote_provider_metadata_includes_endpoint() {
        let provider = RemoteComputeProvider::new("unix:///tmp/test.sock".to_string());
        let m = provider.metadata();
        assert_eq!(m.get("provider").map(String::as_str), Some("remote"));
        assert_eq!(
            m.get("endpoint").map(String::as_str),
            Some("unix:///tmp/test.sock")
        );
    }

    #[tokio::test]
    async fn remote_provider_health_check_fails_on_unreachable() {
        let provider = RemoteComputeProvider::new("unix:///nonexistent/socket.sock".to_string());
        assert!(!provider.health_check().await);
    }

    #[test]
    fn auto_detect_no_env_falls_back_to_local() {
        temp_env::with_vars(
            [
                ("COMPUTE_PROVIDER_TYPE", None::<&str>),
                ("COMPUTE_ENDPOINT", None::<&str>),
                ("COMPUTE_SERVICE_ENDPOINT", None::<&str>),
                ("TOADSTOOL_ENDPOINT", None::<&str>),
            ],
            || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("rt");
                let provider = rt
                    .block_on(auto_detect_compute_provider())
                    .expect("should fall back to local");
                assert_eq!(provider.provider_name(), "local");
            },
        );
    }
}
