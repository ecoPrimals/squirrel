// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Orchestration integration module for universal patterns
//!
//! This module provides orchestration patterns and integration with Songbird
//! for consistent task management and coordination across all primals.

use std::collections::HashMap;
use std::time::Duration;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::traits::{PrimalInfo, PrimalState, PrimalError};

/// Orchestration error types
#[derive(Debug, thiserror::Error)]
pub enum OrchestrationError {
    #[error("Task execution failed: {0}")]
    TaskExecution(String),
    #[error("Scheduling error: {0}")]
    Scheduling(String),
    #[error("Communication error: {0}")]
    Communication(String),
    #[error("Coordination error: {0}")]
    Coordination(String),
    #[error("Service discovery error: {0}")]
    ServiceDiscovery(String),
    #[error("Health check error: {0}")]
    HealthCheck(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Configuration error: {0}")]
    Configuration(String),
    #[error("Timeout error: {0}")]
    Timeout(String),
    #[error("Resource error: {0}")]
    Resource(String),
    #[error("Dependency error: {0}")]
    Dependency(String),
    #[error("Other error: {0}")]
    Other(String),
}

/// Orchestration provider trait
#[async_trait]
pub trait OrchestrationProvider: Send + Sync {
    /// Register a primal with the orchestration system
    async fn register_primal(&self, primal_info: &PrimalInfo) -> Result<(), OrchestrationError>;
    
    /// Unregister a primal from the orchestration system
    async fn unregister_primal(&self, primal_id: &str) -> Result<(), OrchestrationError>;
    
    /// Update primal state
    async fn update_primal_state(&self, primal_id: &str, state: PrimalState) -> Result<(), OrchestrationError>;
    
    /// Schedule a task
    async fn schedule_task(&self, task: OrchestrationTask) -> Result<TaskId, OrchestrationError>;
    
    /// Cancel a task
    async fn cancel_task(&self, task_id: &TaskId) -> Result<(), OrchestrationError>;
    
    /// Get task status
    async fn get_task_status(&self, task_id: &TaskId) -> Result<TaskStatus, OrchestrationError>;
    
    /// Discover services
    async fn discover_services(&self, service_type: &str) -> Result<Vec<ServiceInfo>, OrchestrationError>;
    
    /// Report health status
    async fn report_health(&self, primal_id: &str, health: HealthReport) -> Result<(), OrchestrationError>;
    
    /// Get cluster status
    async fn get_cluster_status(&self) -> Result<ClusterStatus, OrchestrationError>;
}

/// Task identifier
pub type TaskId = Uuid;

/// Orchestration task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationTask {
    /// Task ID
    pub id: TaskId,
    
    /// Task name
    pub name: String,
    
    /// Task type
    pub task_type: String,
    
    /// Target primal
    pub target_primal: Option<String>,
    
    /// Task payload
    pub payload: serde_json::Value,
    
    /// Task priority
    pub priority: TaskPriority,
    
    /// Retry policy
    pub retry_policy: RetryPolicy,
    
    /// Timeout
    pub timeout: Duration,
    
    /// Dependencies
    pub dependencies: Vec<TaskId>,
    
    /// Scheduling constraints
    pub constraints: TaskConstraints,
    
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    
    /// Scheduled timestamp
    pub scheduled_at: Option<DateTime<Utc>>,
}

/// Task priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Retry policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// Maximum number of retries
    pub max_retries: u32,
    
    /// Backoff strategy
    pub backoff: BackoffStrategy,
    
    /// Retry only on specific errors
    pub retry_on: Vec<String>,
}

/// Backoff strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackoffStrategy {
    /// Fixed delay
    Fixed(Duration),
    
    /// Exponential backoff
    Exponential {
        initial: Duration,
        max: Duration,
        multiplier: f64,
    },
    
    /// Linear backoff
    Linear {
        initial: Duration,
        increment: Duration,
    },
}

/// Task constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskConstraints {
    /// Required resources
    pub required_resources: HashMap<String, u64>,
    
    /// Node affinity
    pub node_affinity: Option<String>,
    
    /// Anti-affinity rules
    pub anti_affinity: Vec<String>,
    
    /// Placement constraints
    pub placement: Vec<PlacementConstraint>,
}

/// Placement constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacementConstraint {
    /// Constraint type
    pub constraint_type: String,
    
    /// Constraint value
    pub value: String,
    
    /// Constraint operator
    pub operator: ConstraintOperator,
}

/// Constraint operator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintOperator {
    Equal,
    NotEqual,
    In,
    NotIn,
    Exists,
    NotExists,
}

/// Task status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatus {
    /// Task ID
    pub task_id: TaskId,
    
    /// Current state
    pub state: TaskState,
    
    /// Result (if completed)
    pub result: Option<serde_json::Value>,
    
    /// Error (if failed)
    pub error: Option<String>,
    
    /// Progress (0-100)
    pub progress: u8,
    
    /// Execution history
    pub execution_history: Vec<TaskExecution>,
    
    /// Next retry time
    pub next_retry: Option<DateTime<Utc>>,
    
    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
}

/// Task state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskState {
    /// Task is queued
    Queued,
    
    /// Task is scheduled
    Scheduled,
    
    /// Task is running
    Running,
    
    /// Task completed successfully
    Completed,
    
    /// Task failed
    Failed,
    
    /// Task was cancelled
    Cancelled,
    
    /// Task is retrying
    Retrying,
}

/// Task execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecution {
    /// Execution ID
    pub id: Uuid,
    
    /// Start time
    pub started_at: DateTime<Utc>,
    
    /// End time
    pub ended_at: Option<DateTime<Utc>>,
    
    /// Execution node
    pub node: String,
    
    /// Exit code
    pub exit_code: Option<i32>,
    
    /// Error message
    pub error: Option<String>,
    
    /// Logs
    pub logs: Vec<String>,
}

/// Service information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// Service ID
    pub id: String,
    
    /// Service name
    pub name: String,
    
    /// Service type
    pub service_type: String,
    
    /// Service version
    pub version: String,
    
    /// Service endpoint
    pub endpoint: Url,
    
    /// Service metadata
    pub metadata: HashMap<String, String>,
    
    /// Health status
    pub health: ServiceHealth,
    
    /// Last seen
    pub last_seen: DateTime<Utc>,
}

/// Service health status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceHealth {
    Healthy,
    Unhealthy,
    Unknown,
}

/// Health report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    /// Overall health status
    pub status: ServiceHealth,
    
    /// Health checks
    pub checks: HashMap<String, HealthCheck>,
    
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Check name
    pub name: String,
    
    /// Check status
    pub status: ServiceHealth,
    
    /// Check message
    pub message: String,
    
    /// Check duration
    pub duration: Duration,
}

/// Cluster status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterStatus {
    /// Cluster ID
    pub cluster_id: String,
    
    /// Total nodes
    pub total_nodes: u32,
    
    /// Healthy nodes
    pub healthy_nodes: u32,
    
    /// Total services
    pub total_services: u32,
    
    /// Healthy services
    pub healthy_services: u32,
    
    /// Total tasks
    pub total_tasks: u32,
    
    /// Running tasks
    pub running_tasks: u32,
    
    /// Resource usage
    pub resource_usage: ResourceUsage,
    
    /// Last updated
    pub updated_at: DateTime<Utc>,
}

/// Resource usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// CPU usage (percentage)
    pub cpu_percent: f64,
    
    /// Memory usage (bytes)
    pub memory_bytes: u64,
    
    /// Total memory (bytes)
    pub memory_total: u64,
    
    /// Disk usage (bytes)
    pub disk_bytes: u64,
    
    /// Total disk (bytes)
    pub disk_total: u64,
    
    /// Network bytes sent
    pub network_bytes_sent: u64,
    
    /// Network bytes received
    pub network_bytes_received: u64,
}

/// Songbird integration for orchestration
#[derive(Debug)]
pub struct SongbirdIntegration {
    endpoint: Url,
    client: reqwest::Client,
    cluster_id: String,
}

impl SongbirdIntegration {
    /// Create a new Songbird integration
    pub fn new(endpoint: Url, cluster_id: String) -> Self {
        Self {
            endpoint,
            client: reqwest::Client::new(),
            cluster_id,
        }
    }
    
    /// Get the cluster ID
    pub fn cluster_id(&self) -> &str {
        &self.cluster_id
    }
    
    /// Get the endpoint
    pub fn endpoint(&self) -> &Url {
        &self.endpoint
    }
}

#[async_trait]
impl OrchestrationProvider for SongbirdIntegration {
    async fn register_primal(&self, primal_info: &PrimalInfo) -> Result<(), OrchestrationError> {
        let url = self.endpoint.join("/primals/register")
            .map_err(|e| OrchestrationError::Configuration(e.to_string()))?;
        
        let response = self.client
            .post(url)
            .json(&serde_json::json!({
                "cluster_id": self.cluster_id,
                "primal_info": primal_info
            }))
            .send()
            .await
            .map_err(|e| OrchestrationError::Network(e.to_string()))?;
        
        if response.status().is_success() {
            Ok(())
        } else {
            Err(OrchestrationError::Communication(format!("HTTP {}", response.status())))
        }
    }
    
    async fn unregister_primal(&self, primal_id: &str) -> Result<(), OrchestrationError> {
        let url = self.endpoint.join(&format!("/primals/{}/unregister", primal_id))
            .map_err(|e| OrchestrationError::Configuration(e.to_string()))?;
        
        let response = self.client
            .post(url)
            .json(&serde_json::json!({
                "cluster_id": self.cluster_id
            }))
            .send()
            .await
            .map_err(|e| OrchestrationError::Network(e.to_string()))?;
        
        if response.status().is_success() {
            Ok(())
        } else {
            Err(OrchestrationError::Communication(format!("HTTP {}", response.status())))
        }
    }
    
    async fn update_primal_state(&self, primal_id: &str, state: PrimalState) -> Result<(), OrchestrationError> {
        let url = self.endpoint.join(&format!("/primals/{}/state", primal_id))
            .map_err(|e| OrchestrationError::Configuration(e.to_string()))?;
        
        let response = self.client
            .put(url)
            .json(&serde_json::json!({
                "cluster_id": self.cluster_id,
                "state": state
            }))
            .send()
            .await
            .map_err(|e| OrchestrationError::Network(e.to_string()))?;
        
        if response.status().is_success() {
            Ok(())
        } else {
            Err(OrchestrationError::Communication(format!("HTTP {}", response.status())))
        }
    }
    
    async fn schedule_task(&self, task: OrchestrationTask) -> Result<TaskId, OrchestrationError> {
        let url = self.endpoint.join("/tasks/schedule")
            .map_err(|e| OrchestrationError::Configuration(e.to_string()))?;
        
        let response = self.client
            .post(url)
            .json(&serde_json::json!({
                "cluster_id": self.cluster_id,
                "task": task
            }))
            .send()
            .await
            .map_err(|e| OrchestrationError::Network(e.to_string()))?;
        
        if response.status().is_success() {
            let result: serde_json::Value = response.json().await
                .map_err(|e| OrchestrationError::Network(e.to_string()))?;
            let task_id = result.get("task_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| OrchestrationError::Scheduling("Missing task_id field".to_string()))?;
            task_id.parse()
                .map_err(|e| OrchestrationError::Scheduling(format!("Invalid task_id: {}", e)))
        } else {
            Err(OrchestrationError::Scheduling(format!("HTTP {}", response.status())))
        }
    }
    
    async fn cancel_task(&self, task_id: &TaskId) -> Result<(), OrchestrationError> {
        let url = self.endpoint.join(&format!("/tasks/{}/cancel", task_id))
            .map_err(|e| OrchestrationError::Configuration(e.to_string()))?;
        
        let response = self.client
            .post(url)
            .json(&serde_json::json!({
                "cluster_id": self.cluster_id
            }))
            .send()
            .await
            .map_err(|e| OrchestrationError::Network(e.to_string()))?;
        
        if response.status().is_success() {
            Ok(())
        } else {
            Err(OrchestrationError::TaskExecution(format!("HTTP {}", response.status())))
        }
    }
    
    async fn get_task_status(&self, task_id: &TaskId) -> Result<TaskStatus, OrchestrationError> {
        let url = self.endpoint.join(&format!("/tasks/{}/status", task_id))
            .map_err(|e| OrchestrationError::Configuration(e.to_string()))?;
        
        let response = self.client
            .get(url)
            .header("X-Cluster-ID", &self.cluster_id)
            .send()
            .await
            .map_err(|e| OrchestrationError::Network(e.to_string()))?;
        
        if response.status().is_success() {
            let task_status: TaskStatus = response.json().await
                .map_err(|e| OrchestrationError::Network(e.to_string()))?;
            Ok(task_status)
        } else {
            Err(OrchestrationError::TaskExecution(format!("HTTP {}", response.status())))
        }
    }
    
    async fn discover_services(&self, service_type: &str) -> Result<Vec<ServiceInfo>, OrchestrationError> {
        let url = self.endpoint.join(&format!("/services/discover?type={}", service_type))
            .map_err(|e| OrchestrationError::Configuration(e.to_string()))?;
        
        let response = self.client
            .get(url)
            .header("X-Cluster-ID", &self.cluster_id)
            .send()
            .await
            .map_err(|e| OrchestrationError::Network(e.to_string()))?;
        
        if response.status().is_success() {
            let services: Vec<ServiceInfo> = response.json().await
                .map_err(|e| OrchestrationError::Network(e.to_string()))?;
            Ok(services)
        } else {
            Err(OrchestrationError::ServiceDiscovery(format!("HTTP {}", response.status())))
        }
    }
    
    async fn report_health(&self, primal_id: &str, health: HealthReport) -> Result<(), OrchestrationError> {
        let url = self.endpoint.join(&format!("/primals/{}/health", primal_id))
            .map_err(|e| OrchestrationError::Configuration(e.to_string()))?;
        
        let response = self.client
            .post(url)
            .json(&serde_json::json!({
                "cluster_id": self.cluster_id,
                "health": health
            }))
            .send()
            .await
            .map_err(|e| OrchestrationError::Network(e.to_string()))?;
        
        if response.status().is_success() {
            Ok(())
        } else {
            Err(OrchestrationError::HealthCheck(format!("HTTP {}", response.status())))
        }
    }
    
    async fn get_cluster_status(&self) -> Result<ClusterStatus, OrchestrationError> {
        let url = self.endpoint.join("/cluster/status")
            .map_err(|e| OrchestrationError::Configuration(e.to_string()))?;
        
        let response = self.client
            .get(url)
            .header("X-Cluster-ID", &self.cluster_id)
            .send()
            .await
            .map_err(|e| OrchestrationError::Network(e.to_string()))?;
        
        if response.status().is_success() {
            let cluster_status: ClusterStatus = response.json().await
                .map_err(|e| OrchestrationError::Network(e.to_string()))?;
            Ok(cluster_status)
        } else {
            Err(OrchestrationError::Coordination(format!("HTTP {}", response.status())))
        }
    }
}

/// Mock orchestration provider for testing
#[cfg(test)]
#[derive(Debug, Default)]
pub struct MockOrchestrationProvider {
    pub should_succeed: bool,
    pub tasks: HashMap<TaskId, TaskStatus>,
    pub services: Vec<ServiceInfo>,
}

#[cfg(test)]
impl MockOrchestrationProvider {
    pub fn new() -> Self {
        Self {
            should_succeed: true,
            tasks: HashMap::new(),
            services: Vec::new(),
        }
    }
    
    pub fn with_success(mut self, should_succeed: bool) -> Self {
        self.should_succeed = should_succeed;
        self
    }
    
    pub fn add_service(mut self, service: ServiceInfo) -> Self {
        self.services.push(service);
        self
    }
}

#[async_trait]
#[cfg(test)]
impl OrchestrationProvider for MockOrchestrationProvider {
    async fn register_primal(&self, _primal_info: &PrimalInfo) -> Result<(), OrchestrationError> {
        if self.should_succeed {
            Ok(())
        } else {
            Err(OrchestrationError::Communication("Mock failure".to_string()))
        }
    }
    
    async fn unregister_primal(&self, _primal_id: &str) -> Result<(), OrchestrationError> {
        if self.should_succeed {
            Ok(())
        } else {
            Err(OrchestrationError::Communication("Mock failure".to_string()))
        }
    }
    
    async fn update_primal_state(&self, _primal_id: &str, _state: PrimalState) -> Result<(), OrchestrationError> {
        if self.should_succeed {
            Ok(())
        } else {
            Err(OrchestrationError::Communication("Mock failure".to_string()))
        }
    }
    
    async fn schedule_task(&self, task: OrchestrationTask) -> Result<TaskId, OrchestrationError> {
        if self.should_succeed {
            Ok(task.id)
        } else {
            Err(OrchestrationError::Scheduling("Mock failure".to_string()))
        }
    }
    
    async fn cancel_task(&self, _task_id: &TaskId) -> Result<(), OrchestrationError> {
        if self.should_succeed {
            Ok(())
        } else {
            Err(OrchestrationError::TaskExecution("Mock failure".to_string()))
        }
    }
    
    async fn get_task_status(&self, task_id: &TaskId) -> Result<TaskStatus, OrchestrationError> {
        if self.should_succeed {
            Ok(TaskStatus {
                task_id: *task_id,
                state: TaskState::Completed,
                result: Some(serde_json::json!({"success": true})),
                error: None,
                progress: 100,
                execution_history: Vec::new(),
                next_retry: None,
                updated_at: Utc::now(),
            })
        } else {
            Err(OrchestrationError::TaskExecution("Mock failure".to_string()))
        }
    }
    
    async fn discover_services(&self, _service_type: &str) -> Result<Vec<ServiceInfo>, OrchestrationError> {
        if self.should_succeed {
            Ok(self.services.clone())
        } else {
            Err(OrchestrationError::ServiceDiscovery("Mock failure".to_string()))
        }
    }
    
    async fn report_health(&self, _primal_id: &str, _health: HealthReport) -> Result<(), OrchestrationError> {
        if self.should_succeed {
            Ok(())
        } else {
            Err(OrchestrationError::HealthCheck("Mock failure".to_string()))
        }
    }
    
    async fn get_cluster_status(&self) -> Result<ClusterStatus, OrchestrationError> {
        if self.should_succeed {
            Ok(ClusterStatus {
                cluster_id: "test-cluster".to_string(),
                total_nodes: 3,
                healthy_nodes: 3,
                total_services: 5,
                healthy_services: 5,
                total_tasks: 10,
                running_tasks: 2,
                resource_usage: ResourceUsage {
                    cpu_percent: 50.0,
                    memory_bytes: 1024 * 1024 * 1024,
                    memory_total: 2048 * 1024 * 1024,
                    disk_bytes: 10 * 1024 * 1024 * 1024,
                    disk_total: 100 * 1024 * 1024 * 1024,
                    network_bytes_sent: 1000000,
                    network_bytes_received: 2000000,
                },
                updated_at: Utc::now(),
            })
        } else {
            Err(OrchestrationError::Coordination("Mock failure".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::PrimalType;
    
    #[tokio::test]
    async fn test_mock_orchestration_provider() {
        let provider = MockOrchestrationProvider::new()
            .with_success(true);
        
        // Test primal registration
        let primal_info = PrimalInfo {
            name: "test-primal".to_string(),
            version: "1.0.0".to_string(),
            instance_id: Uuid::new_v4(),
            primal_type: PrimalType::Coordinator,
            description: "Test primal".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            tags: vec!["test".to_string()],
            capabilities: vec!["test-capability".to_string()],
        };
        
        assert!(provider.register_primal(&primal_info).await.is_ok());
        assert!(provider.update_primal_state(&primal_info.instance_id.to_string(), PrimalState::Running).await.is_ok());
        
        // Test task scheduling
        let task = OrchestrationTask {
            id: Uuid::new_v4(),
            name: "test-task".to_string(),
            task_type: "test".to_string(),
            target_primal: None,
            payload: serde_json::json!({"key": "value"}),
            priority: TaskPriority::Normal,
            retry_policy: RetryPolicy {
                max_retries: 3,
                backoff: BackoffStrategy::Fixed(Duration::from_secs(1)),
                retry_on: vec!["error".to_string()],
            },
            timeout: Duration::from_secs(30),
            dependencies: Vec::new(),
            constraints: TaskConstraints {
                required_resources: HashMap::new(),
                node_affinity: None,
                anti_affinity: Vec::new(),
                placement: Vec::new(),
            },
            created_at: Utc::now(),
            scheduled_at: None,
        };
        
        let task_id = provider.schedule_task(task).await.expect("should succeed");
        let task_status = provider.get_task_status(&task_id).await.expect("should succeed");
        assert_eq!(task_status.state, TaskState::Completed);
        
        // Test service discovery
        let services = provider.discover_services("test").await.expect("should succeed");
        assert_eq!(services.len(), 0);
        
        // Test cluster status
        let cluster_status = provider.get_cluster_status().await.expect("should succeed");
        assert_eq!(cluster_status.cluster_id, "test-cluster");
        assert_eq!(cluster_status.total_nodes, 3);
        assert_eq!(cluster_status.healthy_nodes, 3);
    }
} 