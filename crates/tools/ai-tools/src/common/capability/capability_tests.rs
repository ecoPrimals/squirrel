// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use super::*;

// --- SecurityLevel tests ---
#[test]
fn test_security_level_serde() {
    let levels = vec![
        SecurityLevel::Low,
        SecurityLevel::Medium,
        SecurityLevel::High,
        SecurityLevel::Critical,
    ];
    for level in levels {
        let json = serde_json::to_string(&level).expect("should succeed");
        let deserialized: SecurityLevel = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(deserialized, level);
    }
}

#[test]
fn test_security_level_hash() {
    let mut set = HashSet::new();
    set.insert(SecurityLevel::Low);
    set.insert(SecurityLevel::High);
    set.insert(SecurityLevel::Low); // duplicate
    assert_eq!(set.len(), 2);
}

// --- ModelType tests ---
#[test]
fn test_model_type_serde() {
    let types = vec![
        ModelType::LargeLanguageModel,
        ModelType::ChatModel,
        ModelType::Embedding,
        ModelType::ImageGeneration,
        ModelType::ImageUnderstanding,
        ModelType::AudioTranscription,
        ModelType::AudioGeneration,
        ModelType::MultiModal,
        ModelType::Custom("my-model".to_string()),
    ];
    for mt in types {
        let json = serde_json::to_string(&mt).expect("should succeed");
        let deserialized: ModelType = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(deserialized, mt);
    }
}

// --- TaskType tests ---
#[test]
fn test_task_type_serde() {
    let types = vec![
        TaskType::TextGeneration,
        TaskType::CodeGeneration,
        TaskType::Translation,
        TaskType::Summarization,
        TaskType::QuestionAnswering,
        TaskType::ChatCompletion,
        TaskType::FunctionCalling,
        TaskType::ImageGeneration,
        TaskType::Other,
        TaskType::Custom("my-task".to_string()),
    ];
    for tt in types {
        let json = serde_json::to_string(&tt).expect("should succeed");
        let deserialized: TaskType = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(deserialized, tt);
    }
}

// --- CostTier tests ---
#[test]
fn test_cost_tier_default() {
    let tier = CostTier::default();
    assert_eq!(tier, CostTier::Medium);
}

#[test]
fn test_cost_tier_ordering() {
    assert!(CostTier::Free < CostTier::Low);
    assert!(CostTier::Low < CostTier::Medium);
    assert!(CostTier::Medium < CostTier::High);
}

#[test]
fn test_cost_tier_serde() {
    let tiers = vec![
        CostTier::Free,
        CostTier::Low,
        CostTier::Medium,
        CostTier::High,
    ];
    for tier in tiers {
        let json = serde_json::to_string(&tier).expect("should succeed");
        let deserialized: CostTier = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(deserialized, tier);
    }
}

// --- PerformanceMetrics tests ---
#[test]
fn test_performance_metrics_default() {
    let metrics = PerformanceMetrics::default();
    assert!(metrics.avg_latency_ms.is_none());
    assert!(metrics.requests_per_second.is_none());
    assert!(metrics.success_rate.is_none());
}

#[test]
fn test_performance_metrics_serde() {
    let metrics = PerformanceMetrics {
        avg_latency_ms: Some(100),
        requests_per_second: Some(50.0),
        success_rate: Some(0.99),
        avg_tokens_per_second: Some(100.0),
        max_throughput_rps: Some(200.0),
        max_batch_size: Some(32),
        quality_score: Some(95),
    };
    let json = serde_json::to_string(&metrics).expect("should succeed");
    let deserialized: PerformanceMetrics = serde_json::from_str(&json).expect("should succeed");
    assert_eq!(deserialized.avg_latency_ms, Some(100));
    assert_eq!(deserialized.quality_score, Some(95));
}

// --- ResourceRequirements tests ---
#[test]
fn test_resource_requirements_default() {
    let req = ResourceRequirements::default();
    assert_eq!(req.min_memory_mb, 0);
    assert!(!req.requires_gpu);
    assert!(!req.requires_internet);
}

#[test]
fn test_resource_requirements_serde() {
    let req = ResourceRequirements {
        min_memory_mb: 4096,
        min_gpu_memory_mb: Some(8192),
        min_cpu_cores: Some(4),
        requires_gpu: true,
        requires_internet: false,
        load_time_ms: Some(5000),
        requires_specific_hardware: true,
        hardware_requirements: Some("NVIDIA A100".to_string()),
    };
    let json = serde_json::to_string(&req).expect("should succeed");
    let deserialized: ResourceRequirements = serde_json::from_str(&json).expect("should succeed");
    assert_eq!(deserialized.min_memory_mb, 4096);
    assert!(deserialized.requires_gpu);
}

// --- CostMetrics tests ---
#[test]
fn test_cost_metrics_default() {
    let metrics = CostMetrics::default();
    assert!(!metrics.has_fixed_cost);
    assert!(!metrics.is_free);
}

#[test]
fn test_cost_metrics_serde() {
    let metrics = CostMetrics {
        cost_per_1k_input_tokens: Some(0.01),
        cost_per_1k_output_tokens: Some(0.03),
        cost_per_request: Some(0.001),
        has_fixed_cost: true,
        is_free: false,
    };
    let json = serde_json::to_string(&metrics).expect("should succeed");
    let deserialized: CostMetrics = serde_json::from_str(&json).expect("should succeed");
    assert!(deserialized.has_fixed_cost);
    assert!(!deserialized.is_free);
}

// --- SecurityRequirements tests ---
#[test]
fn test_security_requirements_default() {
    let req = SecurityRequirements::default();
    assert!(!req.requires_encryption);
    assert!(!req.contains_sensitive_data);
    assert_eq!(req.security_level, SecurityLevel::Medium);
    assert!(!req.requires_audit_logging);
    assert!(req.geo_restrictions.is_none());
}

#[test]
fn test_security_requirements_serde() {
    let req = SecurityRequirements {
        requires_encryption: true,
        contains_sensitive_data: true,
        security_level: SecurityLevel::Critical,
        requires_audit_logging: true,
        geo_restrictions: Some(GeoConstraints {
            allowed_regions: Some(vec!["US".to_string(), "EU".to_string()]),
            blocked_regions: None,
            data_residency: Some("US".to_string()),
        }),
    };
    let json = serde_json::to_string(&req).expect("should succeed");
    let deserialized: SecurityRequirements = serde_json::from_str(&json).expect("should succeed");
    assert!(deserialized.requires_encryption);
    assert_eq!(deserialized.security_level, SecurityLevel::Critical);
    assert!(deserialized.geo_restrictions.is_some());
}

// --- GeoConstraints tests ---
#[test]
fn test_geo_constraints_serde() {
    let geo = GeoConstraints {
        allowed_regions: Some(vec!["US".to_string()]),
        blocked_regions: Some(vec!["CN".to_string()]),
        data_residency: Some("EU".to_string()),
    };
    let json = serde_json::to_string(&geo).expect("should succeed");
    let deserialized: GeoConstraints = serde_json::from_str(&json).expect("should succeed");
    assert_eq!(
        deserialized
            .allowed_regions
            .as_ref()
            .expect("should succeed")
            .len(),
        1
    );
}

// --- AITask tests ---
#[test]
fn test_ai_task_default() {
    let task = AITask::default();
    assert_eq!(task.task_type, TaskType::TextGeneration);
    assert!(task.required_model_type.is_none());
    assert!(!task.requires_streaming);
    assert!(!task.requires_function_calling);
    assert_eq!(task.priority, 50);
}

#[test]
fn test_ai_task_serde() {
    let task = AITask {
        task_type: TaskType::CodeGeneration,
        required_model_type: Some(ModelType::LargeLanguageModel),
        min_context_size: Some(4096),
        requires_streaming: true,
        requires_function_calling: true,
        requires_tool_use: false,
        security_requirements: SecurityRequirements::default(),
        complexity_score: Some(75),
        priority: 90,
    };
    let json = serde_json::to_string(&task).expect("should succeed");
    let deserialized: AITask = serde_json::from_str(&json).expect("should succeed");
    assert_eq!(deserialized.task_type, TaskType::CodeGeneration);
    assert_eq!(deserialized.priority, 90);
    assert!(deserialized.requires_streaming);
}

// --- AICapabilities tests ---
#[test]
fn test_ai_capabilities_default() {
    let caps = AICapabilities::default();
    assert!(caps.supported_model_types.is_empty());
    assert!(caps.supported_task_types.is_empty());
    assert_eq!(caps.max_context_size, 0);
    assert!(!caps.supports_streaming);
}

#[test]
fn test_ai_capabilities_new() {
    let caps = AICapabilities::new();
    assert!(caps.supported_model_types.is_empty());
}

#[test]
fn test_ai_capabilities_add_model_type() {
    let mut caps = AICapabilities::new();
    caps.add_model_type(ModelType::LargeLanguageModel);
    caps.add_model_type(ModelType::Embedding);
    assert!(caps.supports_model_type(&ModelType::LargeLanguageModel));
    assert!(caps.supports_model_type(&ModelType::Embedding));
    assert!(!caps.supports_model_type(&ModelType::ImageGeneration));
}

#[test]
fn test_ai_capabilities_add_task_type() {
    let mut caps = AICapabilities::new();
    caps.add_task_type(TaskType::TextGeneration);
    caps.add_task_type(TaskType::CodeGeneration);
    assert!(caps.supports_task(&TaskType::TextGeneration));
    assert!(caps.supports_task(&TaskType::CodeGeneration));
    assert!(!caps.supports_task(&TaskType::Translation));
}

#[test]
fn test_ai_capabilities_builder_methods() {
    let mut caps = AICapabilities::new();
    caps.with_max_context_size(128_000)
        .with_streaming(true)
        .with_function_calling(true)
        .with_tool_use(true)
        .with_image_support(true);

    assert_eq!(caps.max_context_size, 128_000);
    assert!(caps.supports_streaming);
    assert!(caps.supports_function_calling);
    assert!(caps.supports_tool_use);
    assert!(caps.supports_images);
}

#[test]
fn test_ai_capabilities_with_cost_metrics() {
    let mut caps = AICapabilities::new();
    let cost_metrics = CostMetrics {
        cost_per_1k_input_tokens: Some(0.01),
        cost_per_1k_output_tokens: Some(0.03),
        cost_per_request: None,
        has_fixed_cost: false,
        is_free: false,
    };
    caps.with_cost_metrics(cost_metrics);
    assert!(!caps.cost_metrics.is_free);
}

#[test]
fn test_ai_capabilities_with_supported_tasks() {
    let mut caps = AICapabilities::new();
    caps.with_supported_tasks(vec![
        TaskType::TextGeneration,
        TaskType::CodeGeneration,
        TaskType::Summarization,
    ]);
    assert_eq!(caps.supported_task_types.len(), 3);
    assert!(caps.supports_task(&TaskType::Summarization));
}

#[test]
fn test_ai_capabilities_serde() {
    let mut caps = AICapabilities::new();
    caps.add_model_type(ModelType::LargeLanguageModel);
    caps.add_task_type(TaskType::TextGeneration);
    caps.with_max_context_size(4096).with_streaming(true);

    let json = serde_json::to_string(&caps).expect("should succeed");
    let deserialized: AICapabilities = serde_json::from_str(&json).expect("should succeed");
    assert_eq!(deserialized.max_context_size, 4096);
    assert!(deserialized.supports_streaming);
}

// --- RoutingPreferences tests ---
#[test]
fn test_routing_preferences_default() {
    let prefs = RoutingPreferences::default();
    assert_eq!(prefs.priority, 50);
    assert!(prefs.allows_forwarding);
    assert!(!prefs.handles_sensitive_data);
    assert!(prefs.geo_constraints.is_none());
    assert_eq!(prefs.cost_tier, CostTier::Medium);
    assert!(!prefs.prefers_local);
    assert!((prefs.cost_sensitivity - 0.5).abs() < f64::EPSILON);
    assert!((prefs.performance_priority - 0.5).abs() < f64::EPSILON);
}

#[test]
fn test_routing_preferences_serde() {
    let prefs = RoutingPreferences {
        priority: 90,
        allows_forwarding: false,
        handles_sensitive_data: true,
        geo_constraints: None,
        cost_tier: CostTier::High,
        prefers_local: true,
        cost_sensitivity: 0.1,
        performance_priority: 0.9,
    };
    let json = serde_json::to_string(&prefs).expect("should succeed");
    let deserialized: RoutingPreferences = serde_json::from_str(&json).expect("should succeed");
    assert_eq!(deserialized.priority, 90);
    assert!(deserialized.prefers_local);
    assert_eq!(deserialized.cost_tier, CostTier::High);
}
