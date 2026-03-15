// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Workflow Management Engine
//!
//! This module provides comprehensive workflow management capabilities including
//! workflow definition, execution, scheduling, state management, and monitoring
//! for complex AI workflows and service orchestration.
//!
//! This module has been refactored to use a modular structure. The main implementation
//! is now in the `workflow` module, with types organized separately.

// Re-export everything from the workflow module for backward compatibility
pub use super::workflow::*;

// Additional backward compatibility exports
pub use super::workflow::types::*;
pub use super::workflow::WorkflowManagementEngine;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tokio::sync::Arc;
    
    #[tokio::test]
    async fn test_workflow_management_engine_creation() {
        let config = WorkflowManagementConfig::default();
        let event_broadcaster = Arc::new(crate::events::EventBroadcaster::new());
        let service_composition = Arc::new(crate::service_composition::ServiceCompositionEngine::new());
        let ai_coordinator = Arc::new(crate::coordinator::AICoordinator::new());
        
        let _engine = WorkflowManagementEngine::new(
            config,
            event_broadcaster,
            service_composition,
            ai_coordinator,
        );
    }
    
    #[tokio::test]
    async fn test_workflow_definition_creation() {
        let workflow = WorkflowDefinition {
            id: "test-workflow".to_string(),
            name: "Test Workflow".to_string(),
            description: "A test workflow".to_string(),
            version: "1.0.0".to_string(),
            steps: vec![],
            config: WorkflowConfig {
                execution_strategy: ExecutionStrategy::Sequential,
                timeout: std::time::Duration::from_secs(300),
                retry: RetryConfig {
                    max_attempts: 3,
                    delay: std::time::Duration::from_secs(1),
                    backoff_strategy: BackoffStrategy::Exponential,
                    conditions: vec![],
                },
                resources: ResourceLimits {
                    max_cpu: 2.0,
                    max_memory: 1024 * 1024 * 1024, // 1GB
                    max_storage: 10 * 1024 * 1024 * 1024, // 10GB
                    max_network: 100 * 1024 * 1024, // 100MB
                    custom: HashMap::new(),
                },
                monitoring: MonitoringConfig {
                    metrics_enabled: true,
                    logging_enabled: true,
                    tracing_enabled: true,
                    alerts: vec![],
                },
                error_handling: ErrorHandlingConfig {
                    strategy: ErrorHandlingStrategy::Retry,
                    recovery_actions: vec![],
                    notifications: vec![],
                },
                security: SecurityConfig {
                    auth_required: false,
                    authorization: vec![],
                    encryption: EncryptionConfig {
                        enabled: false,
                        algorithm: "AES256".to_string(),
                        key_management: "local".to_string(),
                    },
                    access_control: AccessControlConfig {
                        enabled: false,
                        rules: vec![],
                        rbac: false,
                    },
                },
                scaling: ScalingConfig {
                    auto_scaling: false,
                    min_instances: 1,
                    max_instances: 10,
                    metrics: vec![],
                },
            },
            metadata: HashMap::new(),
            parameters: vec![],
            outputs: vec![],
            triggers: vec![],
            dependencies: vec![],
            constraints: vec![],
            created_at: chrono::Utc::now(),
            modified_at: chrono::Utc::now(),
        };
        
        assert_eq!(workflow.id, "test-workflow");
        assert_eq!(workflow.name, "Test Workflow");
        assert_eq!(workflow.version, "1.0.0");
    }
    
    #[tokio::test]
    async fn test_workflow_instance_creation() {
        let instance = WorkflowInstance {
            id: "test-instance".to_string(),
            workflow_id: "test-workflow".to_string(),
            state: WorkflowState::Pending,
            parameters: HashMap::new(),
            outputs: HashMap::new(),
            metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
            step_states: HashMap::new(),
        };
        
        assert_eq!(instance.id, "test-instance");
        assert_eq!(instance.workflow_id, "test-workflow");
        assert_eq!(instance.state, WorkflowState::Pending);
    }
} 