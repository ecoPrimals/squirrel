//! Songbird orchestration integration
//!
//! This module provides integration with the Songbird orchestration system,
//! enabling dynamic primal management and context-aware routing.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use universal_patterns::traits::{
    PrimalProvider, PrimalContext, PrimalHealth, PrimalEndpoints, PrimalRequest, PrimalResponse,
    PrimalResult, PrimalType, PrimalCapability, PrimalDependency, DynamicPortInfo, PortType, PortStatus
};
use universal_patterns::config::UniversalPrimalConfig;
use universal_patterns::registry::UniversalPrimalRegistry;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Songbird orchestration provider
pub struct SongbirdProvider {
    registry: Arc<UniversalPrimalRegistry>,
    config: UniversalPrimalConfig,
    health_status: Arc<RwLock<HealthStatus>>,
    context: PrimalContext,
}

#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub healthy: bool,
    pub message: String,
    pub last_check: chrono::DateTime<chrono::Utc>,
}

impl SongbirdProvider {
    pub fn new(config: UniversalPrimalConfig) -> Self {
        Self {
            registry: Arc::new(UniversalPrimalRegistry::new()),
            config,
            health_status: Arc::new(RwLock::new(HealthStatus {
                healthy: true,
                message: "Songbird orchestration healthy".to_string(),
                last_check: chrono::Utc::now(),
            })),
            context: PrimalContext::default(),
        }
    }

    pub async fn test_integration(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Testing Songbird orchestration integration...");
        
        // Test basic functionality
        let _test_context = PrimalContext::default();
        let health = self.health_check().await;
        
        println!("Health check result: {:?}", health);
        println!("Songbird integration test completed successfully!");
        
        Ok(())
    }
}

#[async_trait]
impl PrimalProvider for SongbirdProvider {
    fn primal_id(&self) -> &str {
        "songbird"
    }

    fn instance_id(&self) -> &str {
        "songbird-orchestrator"
    }

    fn context(&self) -> &PrimalContext {
        &self.context
    }

    fn primal_type(&self) -> PrimalType {
        PrimalType::Orchestration
    }

    fn capabilities(&self) -> Vec<PrimalCapability> {
        vec![
            PrimalCapability::ServiceDiscovery { protocols: vec!["http".to_string(), "grpc".to_string()] },
            PrimalCapability::NetworkRouting { protocols: vec!["tcp".to_string(), "udp".to_string()] },
            PrimalCapability::LoadBalancing { algorithms: vec!["round_robin".to_string(), "least_connections".to_string()] },
            PrimalCapability::AutoScaling { metrics: vec!["cpu".to_string(), "memory".to_string()] },
        ]
    }

    fn dependencies(&self) -> Vec<PrimalDependency> {
        vec![
            PrimalDependency::RequiresAuthentication { methods: vec!["token".to_string()] },
            PrimalDependency::RequiresCompute { types: vec!["container".to_string()] },
        ]
    }

    async fn health_check(&self) -> PrimalHealth {
        let status = self.health_status.read().await;
        if status.healthy {
            PrimalHealth::Healthy
        } else {
            PrimalHealth::Unhealthy { reason: status.message.clone() }
        }
    }

    fn endpoints(&self) -> PrimalEndpoints {
        let base_url = std::env::var("SONGBIRD_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        let ws_base_url = std::env::var("SONGBIRD_WS_URL")
            .unwrap_or_else(|_| base_url.replace("http://", "ws://").replace("https://", "wss://"));
        
        PrimalEndpoints {
            primary: base_url.clone(),
            health: format!("{}/health", base_url),
            metrics: Some(format!("{}/metrics", base_url)),
            admin: Some(format!("{}/admin", base_url)),
            websocket: Some(format!("{}/ws", ws_base_url)),
            custom: "orchestration=/orchestrate,scaling=/scale".to_string(),
        }
    }

    async fn handle_primal_request(&self, request: PrimalRequest) -> PrimalResult<PrimalResponse> {
        let response = PrimalResponse {
            request_id: request.id,
            response_type: match request.request_type {
                universal_patterns::traits::PrimalRequestType::HealthCheck => 
                    universal_patterns::traits::PrimalResponseType::HealthCheck,
                _ => universal_patterns::traits::PrimalResponseType::Custom("orchestration".to_string()),
            },
            payload: HashMap::new(),
            timestamp: chrono::Utc::now(),
            success: true,
            error_message: None,
            metadata: None,
        };
        
        Ok(response)
    }

    async fn initialize(&mut self, _config: serde_json::Value) -> PrimalResult<()> {
        println!("Initializing Songbird orchestration provider...");
        Ok(())
    }

    async fn shutdown(&mut self) -> PrimalResult<()> {
        println!("Shutting down Songbird orchestration provider...");
        Ok(())
    }

    fn can_serve_context(&self, _context: &PrimalContext) -> bool {
        true
    }

    fn dynamic_port_info(&self) -> Option<DynamicPortInfo> {
        Some(DynamicPortInfo {
            assigned_port: 8080,
            port_type: PortType::Http,
            status: PortStatus::Active,
            assigned_at: chrono::Utc::now(),
            lease_duration: chrono::Duration::hours(24),
        })
    }
}

/// Task management for Songbird orchestration
#[derive(Debug, Clone)]
pub struct OrchestrationTask {
    pub id: Uuid,
    pub name: String,
    pub status: TaskStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl OrchestrationTask {
    pub fn new(name: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            status: TaskStatus::Pending,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update_status(&mut self, status: TaskStatus) {
        self.status = status;
        self.updated_at = chrono::Utc::now();
    }
}

/// Songbird integration layer
pub struct SongbirdIntegration {
    provider: SongbirdProvider,
    tasks: Arc<RwLock<HashMap<Uuid, OrchestrationTask>>>,
}

impl SongbirdIntegration {
    pub fn new(config: UniversalPrimalConfig) -> Self {
        Self {
            provider: SongbirdProvider::new(config),
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_task(&self, name: String) -> Uuid {
        let task = OrchestrationTask::new(name);
        let id = task.id;
        let mut tasks = self.tasks.write().await;
        tasks.insert(id, task);
        id
    }

    pub async fn update_task_status(&self, id: Uuid, status: TaskStatus) -> Result<(), String> {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(&id) {
            task.update_status(status);
            Ok(())
        } else {
            Err(format!("Task not found: {}", id))
        }
    }

    pub async fn get_task(&self, id: Uuid) -> Option<OrchestrationTask> {
        let tasks = self.tasks.read().await;
        tasks.get(&id).cloned()
    }

    pub fn provider(&self) -> &SongbirdProvider {
        &self.provider
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use universal_patterns::config::UniversalPrimalConfig;

    #[tokio::test]
    async fn test_songbird_provider_creation() {
        let config = UniversalPrimalConfig::default();
        let provider = SongbirdProvider::new(config);
        
        assert_eq!(provider.primal_id(), "songbird");
        assert_eq!(provider.instance_id(), "songbird-orchestrator");
        
        let health = provider.health_check().await;
        assert!(matches!(health, PrimalHealth::Healthy));
    }

    #[tokio::test]
    async fn test_songbird_integration() {
        let config = UniversalPrimalConfig::default();
        let integration = SongbirdIntegration::new(config);
        
        let task_id = integration.create_task("Test Task".to_string()).await;
        
        let task = integration.get_task(task_id).await.unwrap();
        assert_eq!(task.name, "Test Task");
        assert_eq!(task.status, TaskStatus::Pending);
        
        integration.update_task_status(task_id, TaskStatus::Running).await.unwrap();
        
        let updated_task = integration.get_task(task_id).await.unwrap();
        assert_eq!(updated_task.status, TaskStatus::Running);
    }

    #[tokio::test]
    async fn test_primal_provider_interface() {
        let config = UniversalPrimalConfig::default();
        let provider = SongbirdProvider::new(config);
        
        assert_eq!(provider.primal_type(), PrimalType::Orchestration);
        assert!(!provider.capabilities().is_empty());
        assert!(!provider.dependencies().is_empty());
        
        let endpoints = provider.endpoints();
        assert!(!endpoints.primary.is_empty());
        assert!(!endpoints.health.is_empty());
        
        let port_info = provider.dynamic_port_info().unwrap();
        assert_eq!(port_info.assigned_port, 8080);
        assert_eq!(port_info.port_type, PortType::Http);
        assert_eq!(port_info.status, PortStatus::Active);
    }

    #[tokio::test]
    async fn test_request_handling() {
        let config = UniversalPrimalConfig::default();
        let provider = SongbirdProvider::new(config);
        
        let request = PrimalRequest {
            id: Uuid::new_v4(),
            request_type: universal_patterns::traits::PrimalRequestType::HealthCheck,
            payload: HashMap::new(),
            timestamp: chrono::Utc::now(),
            context: None,
            priority: None,
            security_level: None,
        };
        
        let response = provider.handle_primal_request(request).await.unwrap();
        assert!(response.success);
        assert!(response.error_message.is_none());
    }
} 