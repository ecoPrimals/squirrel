// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Tests for Universal Compute Client

#[cfg(test)]
mod tests {
    use super::super::client::UniversalComputeClient;
    use super::super::types::{ComputeClientConfig, ComputePriority};
    use crate::universal::PrimalContext;
    use crate::universal_primal_ecosystem::UniversalPrimalEcosystem;
    use std::sync::Arc;

    fn test_context() -> PrimalContext {
        PrimalContext::default()
    }

    #[test]
    fn test_compute_client_new() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let config = ComputeClientConfig::default();
        let client = UniversalComputeClient::new(ecosystem, config, test_context());
        assert!(client.get_compute_config().operation_timeout.as_secs() > 0);
    }

    #[tokio::test]
    async fn test_compute_client_initialize() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let config = ComputeClientConfig::default();
        let client = UniversalComputeClient::new(ecosystem, config, test_context());
        client
            .initialize()
            .await
            .expect("initialize should succeed");
    }

    #[tokio::test]
    async fn test_compute_client_execute_code_no_providers() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let config = ComputeClientConfig::default();
        let client = UniversalComputeClient::new(ecosystem, config, test_context());
        client.initialize().await.expect("initialize");
        let result = client
            .execute_code(
                "python",
                "print('hello')".to_string(),
                ComputePriority::Normal,
            )
            .await;
        assert!(result.is_err());
    }

    #[test]
    fn test_compute_client_get_config() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let config = ComputeClientConfig::default();
        let client = UniversalComputeClient::new(ecosystem, config.clone(), test_context());
        let retrieved = client.get_compute_config();
        assert_eq!(retrieved.operation_timeout, config.operation_timeout);
    }

    #[test]
    fn test_apply_configuration_defaults() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let config = ComputeClientConfig::default();
        let client = UniversalComputeClient::new(ecosystem, config, test_context());
        use super::super::types::{
            AIComputeContext, ComputeOperation, ComputePayload, ResourceRequirements,
            UniversalComputeRequest, WorkloadCharacteristics,
        };
        use std::collections::HashMap;

        let mut request = UniversalComputeRequest {
            request_id: uuid::Uuid::new_v4(),
            operation: ComputeOperation::Execute {
                language: "python".to_string(),
                entrypoint: "main".to_string(),
            },
            payload: ComputePayload {
                code: Some("print(1)".to_string()),
                input_data: None,
                environment: HashMap::new(),
                dependencies: vec![],
                parameters: HashMap::new(),
            },
            resources: ResourceRequirements {
                cpu_cores: 0,
                memory_gb: 0,
                gpu_units: None,
                storage_gb: 10,
                max_execution_time: std::time::Duration::from_secs(0),
                network_bandwidth_mbps: None,
            },
            security: crate::compute_client::ComputeSecurityRequirements::default(),
            ai_context: AIComputeContext {
                workload_characteristics: WorkloadCharacteristics {
                    cpu_intensity: 0.5,
                    memory_intensity: 0.3,
                    io_intensity: 0.2,
                    gpu_requirement: 0.0,
                    parallelizability: 0.5,
                },
                priority: ComputePriority::Normal,
                deadline: None,
                cost_performance_preference:
                    super::super::types::CostPerformancePreference::Balanced,
            },
            metadata: HashMap::new(),
        };

        client.apply_configuration_defaults(&mut request);

        assert!(request.resources.cpu_cores > 0 || request.resources.memory_gb > 0);
        assert!(request.resources.max_execution_time.as_secs() > 0);
    }
}
