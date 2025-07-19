//! ToadStool Integration for Squirrel AI Primal
//!
//! This module provides integration with the ToadStool compute primal for
//! intensive AI operations, distributed computing, and resource management.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::error::PrimalError;
use crate::universal::{PrimalCapability, PrimalContext, UniversalResult};
use squirrel_mcp_config::DefaultConfigManager;

/// ToadStool compute integration for intensive AI operations
#[derive(Debug)]
pub struct ToadStoolIntegration {
    pub config: ToadStoolConfig,
    pub compute_state: Arc<RwLock<ComputeState>>,
    pub health_status: HealthStatus,
    pub http_client: reqwest::Client,
}

/// Configuration for ToadStool integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToadStoolConfig {
    pub toadstool_endpoint: String,
    pub heartbeat_interval: Duration,
    pub compute_timeout: Duration,
    pub max_retries: u32,
    pub auth_token: Option<String>,
    pub compute_pool_size: u32,
    pub resource_limits: ResourceLimits,
}

/// Resource limits for compute operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_cpu_cores: u32,
    pub max_memory_gb: u32,
    pub max_gpu_units: u32,
    pub max_concurrent_jobs: u32,
    pub max_job_duration: Duration,
}

/// Compute state management
#[derive(Debug, Clone, Default)]
pub struct ComputeState {
    pub active_jobs: HashMap<String, ComputeJob>,
    pub resource_allocations: HashMap<String, ResourceAllocation>,
    pub compute_nodes: HashMap<String, ComputeNode>,
    pub job_queue: Vec<QueuedJob>,
    pub registered: bool,
}

/// Compute job specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeJob {
    pub job_id: String,
    pub job_type: ComputeJobType,
    pub requester: String,
    pub resource_requirements: ResourceRequirements,
    pub payload: serde_json::Value,
    pub status: JobStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Types of compute jobs supported
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComputeJobType {
    /// AI model training
    ModelTraining,
    /// AI model inference
    ModelInference,
    /// Large language model processing
    LLMProcessing,
    /// Computer vision processing
    VisionProcessing,
    /// Neural network optimization
    NetworkOptimization,
    /// Data preprocessing
    DataPreprocessing,
    /// Distributed computing task
    DistributedCompute,
    /// Custom compute task
    Custom(String),
}

/// Job status tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobStatus {
    /// Job is queued waiting for resources
    Queued,
    /// Job is being prepared
    Preparing,
    /// Job is currently running
    Running,
    /// Job completed successfully
    Completed,
    /// Job failed with error
    Failed,
    /// Job was cancelled
    Cancelled,
    /// Job timed out
    TimedOut,
}

/// Resource requirements for compute jobs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu_cores: u32,
    pub memory_gb: u32,
    pub gpu_units: u32,
    pub storage_gb: u32,
    pub network_bandwidth: u32,
    pub estimated_duration: Duration,
    pub priority: JobPriority,
}

/// Job priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Resource allocation tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    pub allocation_id: String,
    pub job_id: String,
    pub node_id: String,
    pub resources: AllocatedResources,
    pub allocated_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub status: AllocationStatus,
}

/// Allocated resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocatedResources {
    pub cpu_cores: u32,
    pub memory_gb: u32,
    pub gpu_units: u32,
    pub storage_gb: u32,
}

/// Allocation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllocationStatus {
    Active,
    Released,
    Expired,
    Failed,
}

/// Compute node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeNode {
    pub node_id: String,
    pub node_type: NodeType,
    pub endpoint: String,
    pub capabilities: Vec<String>,
    pub total_resources: AllocatedResources,
    pub available_resources: AllocatedResources,
    pub health: NodeHealth,
    pub last_heartbeat: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

/// Types of compute nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    CPU,
    GPU,
    Hybrid,
    Specialized(String),
}

/// Node health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeHealth {
    Healthy,
    Degraded,
    Unhealthy,
    Offline,
}

/// Queued job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedJob {
    pub job: ComputeJob,
    pub queued_at: DateTime<Utc>,
    pub estimated_start: DateTime<Utc>,
    pub queue_position: u32,
}

/// Health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub compute_nodes_healthy: u32,
    pub compute_nodes_total: u32,
    pub active_jobs: u32,
    pub queued_jobs: u32,
    pub resource_utilization: f64,
}

/// Compute job request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeJobRequest {
    pub job_type: ComputeJobType,
    pub resource_requirements: ResourceRequirements,
    pub payload: serde_json::Value,
    pub callback_url: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Compute job response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeJobResponse {
    pub job_id: String,
    pub status: JobStatus,
    pub estimated_completion: Option<DateTime<Utc>>,
    pub queue_position: Option<u32>,
    pub allocated_resources: Option<AllocatedResources>,
}

/// Job result response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResultResponse {
    pub job_id: String,
    pub status: JobStatus,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub execution_time: Option<Duration>,
    pub resource_usage: Option<ResourceUsage>,
}

/// Resource usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub gpu_usage: f64,
    pub network_usage: f64,
    pub peak_memory: u32,
    pub peak_cpu: f64,
}

impl ToadStoolIntegration {
    /// Create a new ToadStool integration
    pub fn new() -> Self {
        let config_manager = DefaultConfigManager::new();
        let external_services = config_manager.get_external_services_config();

        Self {
            config: ToadStoolConfig {
                toadstool_endpoint: std::env::var("TOADSTOOL_ENDPOINT").unwrap_or_else(|_| {
                    external_services
                        .get("toadstool")
                        .cloned()
                        .unwrap_or("http://localhost:9001".to_string())
                }),
                heartbeat_interval: Duration::from_secs(30),
                compute_timeout: Duration::from_secs(3600), // 1 hour
                max_retries: 3,
                auth_token: std::env::var("TOADSTOOL_AUTH_TOKEN").ok(),
                compute_pool_size: 10,
                resource_limits: ResourceLimits {
                    max_cpu_cores: 64,
                    max_memory_gb: 512,
                    max_gpu_units: 8,
                    max_concurrent_jobs: 100,
                    max_job_duration: Duration::from_secs(3600),
                },
            },
            compute_state: Arc::new(RwLock::new(ComputeState::default())),
            health_status: HealthStatus {
                status: "initializing".to_string(),
                timestamp: Utc::now(),
                compute_nodes_healthy: 0,
                compute_nodes_total: 0,
                active_jobs: 0,
                queued_jobs: 0,
                resource_utilization: 0.0,
            },
            http_client: reqwest::Client::new(),
        }
    }

    /// Initialize ToadStool integration
    pub async fn initialize(&mut self) -> Result<(), PrimalError> {
        info!("Initializing ToadStool integration");

        // Test connection to ToadStool
        if let Err(e) = self.test_connection().await {
            warn!("Failed to connect to ToadStool: {}", e);
            info!("Continuing without ToadStool compute delegation");
        } else {
            info!("Successfully connected to ToadStool");

            // Register as compute client
            if let Err(e) = self.register_compute_client().await {
                error!("Failed to register with ToadStool: {}", e);
                return Err(e);
            }

            // Discover available compute nodes
            if let Err(e) = self.discover_compute_nodes().await {
                warn!("Failed to discover compute nodes: {}", e);
            }
        }

        self.health_status.status = "running".to_string();
        self.health_status.timestamp = Utc::now();

        info!("ToadStool integration initialized successfully");
        Ok(())
    }

    /// Test connection to ToadStool
    async fn test_connection(&self) -> Result<(), PrimalError> {
        let health_url = format!("{}/health", self.config.toadstool_endpoint);

        let response = self
            .http_client
            .get(&health_url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| PrimalError::Network(format!("Failed to connect to ToadStool: {}", e)))?;

        if !response.status().is_success() {
            return Err(PrimalError::Network(format!(
                "ToadStool health check failed: {}",
                response.status()
            )));
        }

        Ok(())
    }

    /// Register as compute client with ToadStool
    async fn register_compute_client(&self) -> Result<(), PrimalError> {
        let registration = serde_json::json!({
            "client_id": "squirrel-ai",
            "client_type": "ai_coordinator",
            "capabilities": [
                "job_submission",
                "resource_management",
                "load_balancing"
            ],
            "callback_endpoint": self.get_callback_endpoint(),
            "metadata": {
                "version": "2.2.0",
                "primal_type": "squirrel"
            }
        });

        let register_url = format!("{}/api/v1/clients/register", self.config.toadstool_endpoint);

        let mut request = self
            .http_client
            .post(&register_url)
            .json(&registration)
            .timeout(self.config.compute_timeout);

        if let Some(ref token) = self.config.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request.send().await.map_err(|e| {
            PrimalError::Network(format!("Failed to register with ToadStool: {}", e))
        })?;

        if !response.status().is_success() {
            return Err(PrimalError::Network(format!(
                "ToadStool registration failed: {}",
                response.status()
            )));
        }

        // Update registration status
        let mut state = self.compute_state.write();
        state.registered = true;

        info!("Successfully registered with ToadStool");
        Ok(())
    }

    /// Discover available compute nodes
    async fn discover_compute_nodes(&self) -> Result<(), PrimalError> {
        let discovery_url = format!("{}/api/v1/compute/nodes", self.config.toadstool_endpoint);

        let mut request = self
            .http_client
            .get(&discovery_url)
            .timeout(self.config.compute_timeout);

        if let Some(ref token) = self.config.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request.send().await.map_err(|e| {
            PrimalError::Network(format!("Failed to discover compute nodes: {}", e))
        })?;

        if !response.status().is_success() {
            return Err(PrimalError::Network(format!(
                "Node discovery failed: {}",
                response.status()
            )));
        }

        let nodes: Vec<ComputeNode> = response.json().await.map_err(|e| {
            PrimalError::Internal(format!("Failed to parse node discovery response: {}", e))
        })?;

        // Update compute state with discovered nodes
        let mut state = self.compute_state.write();
        for node in nodes {
            state.compute_nodes.insert(node.node_id.clone(), node);
        }

        info!("Discovered {} compute nodes", state.compute_nodes.len());
        Ok(())
    }

    /// Submit a compute job to ToadStool
    pub async fn submit_job(
        &self,
        job_request: ComputeJobRequest,
    ) -> Result<ComputeJobResponse, PrimalError> {
        debug!("Submitting compute job: {:?}", job_request.job_type);

        let job_id = format!("squirrel-job-{}", uuid::Uuid::new_v4());
        let job = ComputeJob {
            job_id: job_id.clone(),
            job_type: job_request.job_type,
            requester: "squirrel-ai".to_string(),
            resource_requirements: job_request.resource_requirements,
            payload: job_request.payload,
            status: JobStatus::Queued,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            result: None,
            error: None,
            metadata: job_request.metadata,
        };

        let submit_url = format!("{}/api/v1/compute/jobs", self.config.toadstool_endpoint);

        let mut request = self
            .http_client
            .post(&submit_url)
            .json(&job)
            .timeout(self.config.compute_timeout);

        if let Some(ref token) = self.config.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request
            .send()
            .await
            .map_err(|e| PrimalError::Network(format!("Failed to submit job: {}", e)))?;

        if !response.status().is_success() {
            return Err(PrimalError::Network(format!(
                "Job submission failed: {}",
                response.status()
            )));
        }

        let job_response: ComputeJobResponse = response
            .json()
            .await
            .map_err(|e| PrimalError::Internal(format!("Failed to parse job response: {}", e)))?;

        // Update local state
        let mut state = self.compute_state.write();
        state.active_jobs.insert(job_id.clone(), job);

        info!("Successfully submitted job: {}", job_id);
        Ok(job_response)
    }

    /// Get job status
    pub async fn get_job_status(&self, job_id: &str) -> Result<JobResultResponse, PrimalError> {
        let status_url = format!(
            "{}/api/v1/compute/jobs/{}",
            self.config.toadstool_endpoint, job_id
        );

        let mut request = self
            .http_client
            .get(&status_url)
            .timeout(Duration::from_secs(10));

        if let Some(ref token) = self.config.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request
            .send()
            .await
            .map_err(|e| PrimalError::Network(format!("Failed to get job status: {}", e)))?;

        if !response.status().is_success() {
            return Err(PrimalError::Network(format!(
                "Job status query failed: {}",
                response.status()
            )));
        }

        let job_result: JobResultResponse = response
            .json()
            .await
            .map_err(|e| PrimalError::Internal(format!("Failed to parse job result: {}", e)))?;

        Ok(job_result)
    }

    /// Cancel a running job
    pub async fn cancel_job(&self, job_id: &str) -> Result<(), PrimalError> {
        let cancel_url = format!(
            "{}/api/v1/compute/jobs/{}/cancel",
            self.config.toadstool_endpoint, job_id
        );

        let mut request = self
            .http_client
            .post(&cancel_url)
            .timeout(Duration::from_secs(10));

        if let Some(ref token) = self.config.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request
            .send()
            .await
            .map_err(|e| PrimalError::Network(format!("Failed to cancel job: {}", e)))?;

        if !response.status().is_success() {
            return Err(PrimalError::Network(format!(
                "Job cancellation failed: {}",
                response.status()
            )));
        }

        // Update local state
        let mut state = self.compute_state.write();
        if let Some(job) = state.active_jobs.get_mut(job_id) {
            job.status = JobStatus::Cancelled;
        }

        info!("Successfully cancelled job: {}", job_id);
        Ok(())
    }

    /// Get resource utilization statistics
    pub async fn get_resource_utilization(&self) -> Result<ResourceUsage, PrimalError> {
        let stats_url = format!("{}/api/v1/compute/stats", self.config.toadstool_endpoint);

        let mut request = self
            .http_client
            .get(&stats_url)
            .timeout(Duration::from_secs(10));

        if let Some(ref token) = self.config.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request
            .send()
            .await
            .map_err(|e| PrimalError::Network(format!("Failed to get resource stats: {}", e)))?;

        if !response.status().is_success() {
            return Err(PrimalError::Network(format!(
                "Resource stats query failed: {}",
                response.status()
            )));
        }

        let stats: ResourceUsage = response
            .json()
            .await
            .map_err(|e| PrimalError::Internal(format!("Failed to parse resource stats: {}", e)))?;

        Ok(stats)
    }

    /// Update health status
    pub async fn update_health(&mut self) -> Result<(), PrimalError> {
        let state = self.compute_state.read();

        self.health_status.timestamp = Utc::now();
        self.health_status.active_jobs = state.active_jobs.len() as u32;
        self.health_status.queued_jobs = state.job_queue.len() as u32;
        self.health_status.compute_nodes_total = state.compute_nodes.len() as u32;
        self.health_status.compute_nodes_healthy = state
            .compute_nodes
            .values()
            .filter(|node| matches!(node.health, NodeHealth::Healthy))
            .count() as u32;

        // Calculate resource utilization
        let total_resources: u32 = state
            .compute_nodes
            .values()
            .map(|node| node.total_resources.cpu_cores)
            .sum();
        let available_resources: u32 = state
            .compute_nodes
            .values()
            .map(|node| node.available_resources.cpu_cores)
            .sum();

        self.health_status.resource_utilization = if total_resources > 0 {
            1.0 - (available_resources as f64 / total_resources as f64)
        } else {
            0.0
        };

        Ok(())
    }

    /// Get callback endpoint for job notifications
    fn get_callback_endpoint(&self) -> String {
        std::env::var("SQUIRREL_CALLBACK_ENDPOINT").unwrap_or_else(|_| {
            let host =
                std::env::var("SQUIRREL_SERVICE_HOST").unwrap_or_else(|_| "localhost".to_string());
            let port = std::env::var("SQUIRREL_SERVICE_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080);
            format!("http://{}:{}/api/v1/compute/callback", host, port)
        })
    }

    /// Shutdown ToadStool integration
    pub async fn shutdown(&mut self) -> Result<(), PrimalError> {
        info!("Shutting down ToadStool integration");

        // Cancel all active jobs
        let active_jobs: Vec<String> = {
            let state = self.compute_state.read();
            state.active_jobs.keys().cloned().collect()
        };

        for job_id in active_jobs {
            if let Err(e) = self.cancel_job(&job_id).await {
                warn!("Failed to cancel job {}: {}", job_id, e);
            }
        }

        // Unregister from ToadStool
        if let Err(e) = self.unregister_compute_client().await {
            warn!("Failed to unregister from ToadStool: {}", e);
        }

        self.health_status.status = "shutdown".to_string();
        self.health_status.timestamp = Utc::now();

        info!("ToadStool integration shut down successfully");
        Ok(())
    }

    /// Unregister from ToadStool
    async fn unregister_compute_client(&self) -> Result<(), PrimalError> {
        let unregister_url = format!(
            "{}/api/v1/clients/squirrel-ai/unregister",
            self.config.toadstool_endpoint
        );

        let mut request = self
            .http_client
            .post(&unregister_url)
            .timeout(Duration::from_secs(5));

        if let Some(ref token) = self.config.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request.send().await.map_err(|e| {
            PrimalError::Network(format!("Failed to unregister from ToadStool: {}", e))
        })?;

        if !response.status().is_success() {
            return Err(PrimalError::Network(format!(
                "Unregistration failed: {}",
                response.status()
            )));
        }

        info!("Successfully unregistered from ToadStool");
        Ok(())
    }
}

impl Default for ToadStoolIntegration {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_toadstool_integration_initialization() {
        let integration = ToadStoolIntegration::new();
        assert_eq!(integration.health_status.status, "initializing");

        // Test with mock would require actual ToadStool service
        // Here we just test the creation
        assert!(integration.config.compute_pool_size > 0);
        assert!(integration.config.max_retries > 0);
    }

    #[tokio::test]
    async fn test_job_creation() {
        let job_request = ComputeJobRequest {
            job_type: ComputeJobType::ModelInference,
            resource_requirements: ResourceRequirements {
                cpu_cores: 4,
                memory_gb: 16,
                gpu_units: 1,
                storage_gb: 100,
                network_bandwidth: 1000,
                estimated_duration: Duration::from_secs(300),
                priority: JobPriority::Normal,
            },
            payload: serde_json::json!({
                "model": "gpt-4",
                "input": "Hello, world!",
                "parameters": {
                    "max_tokens": 100,
                    "temperature": 0.7
                }
            }),
            callback_url: Some({
                let host = std::env::var("SQUIRREL_SERVICE_HOST")
                    .unwrap_or_else(|_| "localhost".to_string());
                let port = std::env::var("SQUIRREL_SERVICE_PORT")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(8080);
                format!("http://{}:{}/callback", host, port)
            }),
            metadata: HashMap::new(),
        };

        // Test job request creation
        assert_eq!(job_request.resource_requirements.cpu_cores, 4);
        assert_eq!(job_request.resource_requirements.memory_gb, 16);
        assert!(matches!(
            job_request.job_type,
            ComputeJobType::ModelInference
        ));
        assert!(matches!(
            job_request.resource_requirements.priority,
            JobPriority::Normal
        ));
    }

    #[tokio::test]
    async fn test_resource_limits() {
        let integration = ToadStoolIntegration::new();

        assert!(integration.config.resource_limits.max_cpu_cores > 0);
        assert!(integration.config.resource_limits.max_memory_gb > 0);
        assert!(integration.config.resource_limits.max_concurrent_jobs > 0);
    }

    #[tokio::test]
    async fn test_health_status_update() {
        let mut integration = ToadStoolIntegration::new();
        let original_timestamp = integration.health_status.timestamp;

        // Wait a bit to ensure timestamp changes
        tokio::time::sleep(Duration::from_millis(10)).await;

        integration.update_health().await.unwrap();
        assert!(integration.health_status.timestamp > original_timestamp);
    }
}
