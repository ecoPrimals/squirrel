// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Tests for compute client types

use super::*;
use std::time::Duration;

// ============================================================================
// CONFIGURATION TESTS
// ============================================================================

#[test]
fn test_compute_client_config_default() {
    let config = ComputeClientConfig::default();

    assert_eq!(config.operation_timeout, Duration::from_secs(3600));
    assert_eq!(config.max_retries, 3);
    assert_eq!(config.preferred_capabilities.len(), 2);
    assert_eq!(config.resource_requirements.cpu_cores, 2);
    assert_eq!(config.resource_requirements.memory_gb, 4);
}

#[test]
fn test_compute_capability_preference() {
    let pref = ComputeCapabilityPreference {
        capability: ComputeCapabilityType::CpuIntensive {
            cores: 8,
            memory_gb: 16,
            architecture: "x86_64".to_string(),
        },
        weight: 0.9,
        required: true,
    };

    assert!((pref.weight - 0.9).abs() < f64::EPSILON);
    assert!(pref.required);
}

#[test]
fn test_compute_capability_types() {
    let cpu = ComputeCapabilityType::CpuIntensive {
        cores: 4,
        memory_gb: 8,
        architecture: "arm64".to_string(),
    };

    let gpu = ComputeCapabilityType::GpuAccelerated {
        gpu_memory_gb: 16,
        cuda_support: true,
        frameworks: vec!["pytorch".to_string()],
    };

    let container = ComputeCapabilityType::ContainerRuntime {
        orchestrators: vec!["k8s".to_string()],
        isolation_level: "container".to_string(),
    };

    let serverless = ComputeCapabilityType::ServerlessExecution {
        languages: vec!["rust".to_string(), "python".to_string()],
        cold_start_ms: 100,
    };

    let ml = ComputeCapabilityType::MachineLearning {
        frameworks: vec!["tensorflow".to_string()],
        training_support: true,
        inference_support: true,
    };

    let hpc = ComputeCapabilityType::HighPerformanceComputing {
        interconnect: "infiniband".to_string(),
        parallel_processing: true,
        distributed_compute: true,
    };

    // All variants created successfully
    assert!(matches!(cpu, ComputeCapabilityType::CpuIntensive { .. }));
    assert!(matches!(gpu, ComputeCapabilityType::GpuAccelerated { .. }));
    assert!(matches!(
        container,
        ComputeCapabilityType::ContainerRuntime { .. }
    ));
    assert!(matches!(
        serverless,
        ComputeCapabilityType::ServerlessExecution { .. }
    ));
    assert!(matches!(ml, ComputeCapabilityType::MachineLearning { .. }));
    assert!(matches!(
        hpc,
        ComputeCapabilityType::HighPerformanceComputing { .. }
    ));
}

#[test]
fn test_resource_requirements() {
    let resources = ResourceRequirements {
        cpu_cores: 4,
        memory_gb: 8,
        gpu_units: Some(2),
        storage_gb: 100,
        max_execution_time: Duration::from_secs(600),
        network_bandwidth_mbps: Some(1000.0),
    };

    assert_eq!(resources.cpu_cores, 4);
    assert_eq!(resources.memory_gb, 8);
    assert_eq!(resources.gpu_units, Some(2));
    assert_eq!(resources.storage_gb, 100);
}

#[test]
fn test_isolation_levels() {
    let levels = [
        IsolationLevel::Process,
        IsolationLevel::Container,
        IsolationLevel::VirtualMachine,
        IsolationLevel::Hardware,
    ];

    assert_eq!(levels.len(), 4);
}

#[test]
fn test_encryption_requirements() {
    let encryption = EncryptionRequirements {
        data_at_rest: true,
        data_in_transit: true,
        data_in_use: false,
    };

    assert!(encryption.data_at_rest);
    assert!(encryption.data_in_transit);
    assert!(!encryption.data_in_use);
}

#[test]
fn test_network_security_levels() {
    let basic = NetworkSecurityLevel::Basic;
    let vpn = NetworkSecurityLevel::VpnProtected;
    let private = NetworkSecurityLevel::PrivateNetwork;
    let airgapped = NetworkSecurityLevel::AirGapped;

    assert!(matches!(basic, NetworkSecurityLevel::Basic));
    assert!(matches!(vpn, NetworkSecurityLevel::VpnProtected));
    assert!(matches!(private, NetworkSecurityLevel::PrivateNetwork));
    assert!(matches!(airgapped, NetworkSecurityLevel::AirGapped));
}

#[test]
fn test_compute_security_requirements() {
    let security = ComputeSecurityRequirements {
        isolation_level: IsolationLevel::Container,
        trusted_execution: true,
        encryption_requirements: EncryptionRequirements {
            data_at_rest: true,
            data_in_transit: true,
            data_in_use: false,
        },
        network_security: NetworkSecurityLevel::VpnProtected,
    };

    assert!(security.trusted_execution);
    assert!(matches!(
        security.isolation_level,
        IsolationLevel::Container
    ));
}

// ============================================================================
// REQUEST/RESPONSE TESTS
// ============================================================================

#[test]
fn test_compute_operations() {
    let execute = ComputeOperation::Execute {
        language: "rust".to_string(),
        entrypoint: "main".to_string(),
    };

    let train = ComputeOperation::TrainModel {
        framework: "pytorch".to_string(),
        model_type: "transformer".to_string(),
    };

    let inference = ComputeOperation::RunInference {
        model_id: "model-123".to_string(),
        batch_size: 32,
    };

    let batch = ComputeOperation::BatchProcess {
        job_type: "etl".to_string(),
        parallelism: 10,
    };

    let stream = ComputeOperation::StreamProcess {
        stream_source: "kafka://topic".to_string(),
        processing_window: Duration::from_secs(60),
    };

    let custom = ComputeOperation::CustomWorkload {
        workload_type: "custom".to_string(),
        configuration: std::collections::HashMap::new(),
    };

    assert!(matches!(execute, ComputeOperation::Execute { .. }));
    assert!(matches!(train, ComputeOperation::TrainModel { .. }));
    assert!(matches!(inference, ComputeOperation::RunInference { .. }));
    assert!(matches!(batch, ComputeOperation::BatchProcess { .. }));
    assert!(matches!(stream, ComputeOperation::StreamProcess { .. }));
    assert!(matches!(custom, ComputeOperation::CustomWorkload { .. }));
}

#[test]
fn test_compute_payload() {
    let payload = ComputePayload {
        code: Some("fn main() {}".to_string()),
        input_data: Some(vec![1, 2, 3, 4]),
        environment: std::collections::HashMap::from([("VAR1".to_string(), "value1".to_string())]),
        dependencies: vec!["tokio".to_string(), "serde".to_string()],
        parameters: std::collections::HashMap::new(),
    };

    assert!(payload.code.is_some());
    assert!(payload.input_data.is_some());
    assert_eq!(payload.dependencies.len(), 2);
}

#[test]
fn test_workload_characteristics() {
    let workload = WorkloadCharacteristics {
        cpu_intensity: 0.8,
        memory_intensity: 0.6,
        io_intensity: 0.3,
        gpu_requirement: 0.9,
        parallelizability: 0.7,
    };

    assert!((workload.cpu_intensity - 0.8).abs() < f64::EPSILON);
    assert!((workload.gpu_requirement - 0.9).abs() < f64::EPSILON);
}

#[test]
fn test_compute_priorities() {
    let priorities = [
        ComputePriority::Low,
        ComputePriority::Normal,
        ComputePriority::High,
        ComputePriority::Critical,
    ];

    assert_eq!(priorities.len(), 4);
}

#[test]
fn test_cost_performance_preferences() {
    let min_cost = CostPerformancePreference::MinimizeCost;
    let balanced = CostPerformancePreference::Balanced;
    let max_perf = CostPerformancePreference::MaximizePerformance;
    let custom = CostPerformancePreference::Custom {
        cost_weight: 0.3,
        performance_weight: 0.7,
    };

    assert!(matches!(min_cost, CostPerformancePreference::MinimizeCost));
    assert!(matches!(balanced, CostPerformancePreference::Balanced));
    assert!(matches!(
        max_perf,
        CostPerformancePreference::MaximizePerformance
    ));
    assert!(matches!(custom, CostPerformancePreference::Custom { .. }));
}

#[test]
fn test_ai_compute_context() {
    use chrono::Utc;

    let context = AIComputeContext {
        workload_characteristics: WorkloadCharacteristics {
            cpu_intensity: 0.5,
            memory_intensity: 0.5,
            io_intensity: 0.5,
            gpu_requirement: 0.5,
            parallelizability: 0.5,
        },
        priority: ComputePriority::Normal,
        deadline: Some(Utc::now()),
        cost_performance_preference: CostPerformancePreference::Balanced,
    };

    assert!(matches!(context.priority, ComputePriority::Normal));
    assert!(context.deadline.is_some());
}

#[test]
fn test_universal_compute_request() {
    let request = UniversalComputeRequest {
        request_id: uuid::Uuid::new_v4(),
        operation: ComputeOperation::Execute {
            language: "rust".to_string(),
            entrypoint: "main".to_string(),
        },
        payload: ComputePayload {
            code: Some("fn main() {}".to_string()),
            input_data: None,
            environment: std::collections::HashMap::new(),
            dependencies: vec![],
            parameters: std::collections::HashMap::new(),
        },
        resources: ResourceRequirements {
            cpu_cores: 2,
            memory_gb: 4,
            gpu_units: None,
            storage_gb: 10,
            max_execution_time: Duration::from_secs(300),
            network_bandwidth_mbps: None,
        },
        security: ComputeSecurityRequirements {
            isolation_level: IsolationLevel::Container,
            trusted_execution: false,
            encryption_requirements: EncryptionRequirements {
                data_at_rest: true,
                data_in_transit: true,
                data_in_use: false,
            },
            network_security: NetworkSecurityLevel::Basic,
        },
        ai_context: AIComputeContext {
            workload_characteristics: WorkloadCharacteristics {
                cpu_intensity: 0.7,
                memory_intensity: 0.5,
                io_intensity: 0.3,
                gpu_requirement: 0.0,
                parallelizability: 0.6,
            },
            priority: ComputePriority::Normal,
            deadline: None,
            cost_performance_preference: CostPerformancePreference::Balanced,
        },
        metadata: std::collections::HashMap::new(),
    };

    assert!(matches!(
        request.operation,
        ComputeOperation::Execute { .. }
    ));
    assert_eq!(request.resources.cpu_cores, 2);
}

#[test]
fn test_compute_results() {
    let results = ComputeResults {
        output_data: Some(vec![5, 6, 7, 8]),
        return_code: 0,
        stdout: "Success".to_string(),
        stderr: String::new(),
        metadata: std::collections::HashMap::new(),
    };

    assert_eq!(results.return_code, 0);
    assert!(results.output_data.is_some());
}

#[test]
fn test_resource_utilization() {
    let utilization = ResourceUtilization {
        cpu_utilization: 0.75,
        memory_utilization: 0.60,
        gpu_utilization: Some(0.85),
        network_utilization: Some(0.40),
    };

    assert!((utilization.cpu_utilization - 0.75).abs() < f64::EPSILON);
    assert!((utilization.gpu_utilization.expect("should succeed") - 0.85).abs() < f64::EPSILON);
}

#[test]
fn test_cost_breakdown() {
    let cost = CostBreakdown {
        cpu_cost: 0.50,
        memory_cost: 0.25,
        gpu_cost: Some(2.00),
        storage_cost: 0.10,
        network_cost: 0.05,
        total_cost: 2.90,
    };

    assert!((cost.total_cost - 2.90).abs() < f64::EPSILON);
    assert_eq!(cost.gpu_cost, Some(2.00));
}

#[test]
fn test_compute_performance_metrics() {
    let metrics = ComputePerformanceMetrics {
        execution_time: Duration::from_secs(120),
        queue_time: Duration::from_secs(10),
        resource_utilization: ResourceUtilization {
            cpu_utilization: 0.8,
            memory_utilization: 0.7,
            gpu_utilization: None,
            network_utilization: None,
        },
        cost_breakdown: CostBreakdown {
            cpu_cost: 1.0,
            memory_cost: 0.5,
            gpu_cost: None,
            storage_cost: 0.1,
            network_cost: 0.05,
            total_cost: 1.65,
        },
        provider_health: 0.95,
    };

    assert_eq!(metrics.execution_time, Duration::from_secs(120));
    assert!((metrics.provider_health - 0.95).abs() < f64::EPSILON);
}

#[test]
fn test_workload_analysis() {
    let analysis = WorkloadAnalysis {
        patterns: vec!["cpu-bound".to_string(), "batch".to_string()],
        efficiency_score: 0.85,
        bottlenecks: vec!["memory".to_string()],
        recommendations: vec!["increase memory".to_string()],
    };

    assert_eq!(analysis.patterns.len(), 2);
    assert!((analysis.efficiency_score - 0.85).abs() < f64::EPSILON);
}

#[test]
fn test_ai_compute_insights() {
    let insights = AIComputeInsights {
        confidence_score: 0.92,
        performance_optimizations: vec!["use caching".to_string()],
        cost_optimizations: vec!["use spot instances".to_string()],
        alternative_providers: vec!["provider-2".to_string()],
        workload_analysis: WorkloadAnalysis {
            patterns: vec!["io-bound".to_string()],
            efficiency_score: 0.75,
            bottlenecks: vec![],
            recommendations: vec![],
        },
    };

    assert!((insights.confidence_score - 0.92).abs() < f64::EPSILON);
    assert_eq!(insights.performance_optimizations.len(), 1);
}

#[test]
fn test_universal_compute_response() {
    let response = UniversalComputeResponse {
        request_id: uuid::Uuid::new_v4(),
        success: true,
        results: Some(ComputeResults {
            output_data: None,
            return_code: 0,
            stdout: "Done".to_string(),
            stderr: String::new(),
            metadata: std::collections::HashMap::new(),
        }),
        provider_id: "provider-1".to_string(),
        performance: ComputePerformanceMetrics {
            execution_time: Duration::from_secs(60),
            queue_time: Duration::from_secs(5),
            resource_utilization: ResourceUtilization {
                cpu_utilization: 0.7,
                memory_utilization: 0.6,
                gpu_utilization: None,
                network_utilization: None,
            },
            cost_breakdown: CostBreakdown {
                cpu_cost: 0.5,
                memory_cost: 0.3,
                gpu_cost: None,
                storage_cost: 0.1,
                network_cost: 0.05,
                total_cost: 0.95,
            },
            provider_health: 0.98,
        },
        ai_insights: AIComputeInsights {
            confidence_score: 0.88,
            performance_optimizations: vec![],
            cost_optimizations: vec![],
            alternative_providers: vec![],
            workload_analysis: WorkloadAnalysis {
                patterns: vec![],
                efficiency_score: 0.80,
                bottlenecks: vec![],
                recommendations: vec![],
            },
        },
        error: None,
    };

    assert!(response.success);
    assert!(response.results.is_some());
    assert!(response.error.is_none());
}

#[test]
fn test_serialization() {
    let config = ComputeClientConfig::default();
    let json = serde_json::to_string(&config).expect("should succeed");
    let deserialized: ComputeClientConfig = serde_json::from_str(&json).expect("should succeed");

    assert_eq!(config.max_retries, deserialized.max_retries);
}
