//! ToadStool integration implementation
//!
//! This module provides the main integration logic for interacting
//! with the ToadStool compute primal.

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use crate::error::PrimalError;

use super::config::ToadStoolConfig;
use super::health::HealthStatus;
use super::job::ComputeJob;
use super::messages::{ComputeJobRequest, ComputeJobResponse};
use super::node::ComputeNode;
use super::state::ComputeState;

/// ToadStool compute integration for intensive AI operations
#[derive(Debug)]
pub struct ToadStoolIntegration {
    pub config: ToadStoolConfig,
    pub compute_state: Arc<RwLock<ComputeState>>,
    pub health_status: HealthStatus,
    pub http_client: reqwest::Client,
}

impl ToadStoolIntegration {
    /// Create a new ToadStool integration
    pub fn new() -> Self {
        Self {
            config: ToadStoolConfig::default(),
            compute_state: Arc::new(RwLock::new(ComputeState::default())),
            health_status: HealthStatus::default(),
            http_client: reqwest::Client::new(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: ToadStoolConfig) -> Self {
        Self {
            config,
            compute_state: Arc::new(RwLock::new(ComputeState::default())),
            health_status: HealthStatus::default(),
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
        self.health_status.timestamp = chrono::Utc::now();

        info!("ToadStool integration initialized successfully");
        Ok(())
    }

    /// Test connection to ToadStool
    async fn test_connection(&self) -> Result<(), PrimalError> {
        let health_url = format!("{}/health", self.config.toadstool_endpoint);

        let response = self
            .http_client
            .get(&health_url)
            .timeout(std::time::Duration::from_secs(5))
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
                "version": env!("CARGO_PKG_VERSION"),
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
        let mut state = self.compute_state.write().await;
        state.registered = true;

        info!("Successfully registered with ToadStool");
        Ok(())
    }

    /// Discover compute nodes
    async fn discover_compute_nodes(&self) -> Result<(), PrimalError> {
        let discovery_url = format!("{}/api/v1/compute/nodes", self.config.toadstool_endpoint);

        let mut request = self.http_client.get(&discovery_url);

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

        let nodes: Vec<ComputeNode> = response
            .json()
            .await
            .map_err(|e| PrimalError::Network(format!("Failed to parse compute nodes: {}", e)))?;

        // Update compute state with discovered nodes
        let mut state = self.compute_state.write().await;
        for node in nodes {
            state.add_node(node.node_id.clone(), node);
        }

        info!("Discovered {} compute nodes", state.node_count());
        Ok(())
    }

    /// Submit a compute job to ToadStool
    pub async fn submit_job(
        &self,
        job_request: ComputeJobRequest,
    ) -> Result<ComputeJobResponse, PrimalError> {
        use uuid::Uuid;
        let job_id = format!("squirrel-job-{}", Uuid::new_v4());

        let job = ComputeJob {
            job_id: job_id.clone(),
            job_type: job_request.job_type,
            requester: "squirrel-ai".to_string(),
            resource_requirements: job_request.resource_requirements,
            payload: job_request.payload,
            status: super::job::JobStatus::Queued,
            created_at: chrono::Utc::now(),
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
            .map_err(|e| PrimalError::Network(format!("Failed to parse job response: {}", e)))?;

        // Add job to active jobs
        let mut state = self.compute_state.write().await;
        state.add_job(job_id.clone(), job);

        Ok(job_response)
    }

    /// Get the callback endpoint for job updates
    fn get_callback_endpoint(&self) -> String {
        use universal_constants::{builders, network};
        let port = network::get_port_from_env("SQUIRREL_HTTP_PORT", 9010);
        format!(
            "{}/api/v1/toadstool/callbacks",
            builders::localhost_http(port)
        )
    }

    /// Get health status
    pub fn get_health(&self) -> &HealthStatus {
        &self.health_status
    }

    /// Shutdown the integration
    pub async fn shutdown(&mut self) -> Result<(), PrimalError> {
        info!("Shutting down ToadStool integration");
        self.health_status.status = "shutdown".to_string();
        Ok(())
    }
}

impl Default for ToadStoolIntegration {
    fn default() -> Self {
        Self::new()
    }
}
