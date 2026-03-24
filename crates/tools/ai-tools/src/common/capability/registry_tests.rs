// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use super::*;
use tempfile::NamedTempFile;

#[test]
fn test_register_model() {
    let mut registry = ModelRegistry::new();

    let capabilities = ModelCapabilities {
        name: "test-model".to_string(),
        provider_id: "test-provider".to_string(),
        version: Some("1.0".to_string()),
        model_types: vec!["LargeLanguageModel".to_string()],
        task_types: vec!["TextGeneration".to_string()],
        max_context_size: Some(4096),
        supports_streaming: true,
        supports_function_calling: false,
        supports_tool_use: false,
        performance: PerformanceConfig::default(),
        resources: ResourceConfig {
            requires_internet: true,
            ..Default::default()
        },
        cost: CostConfig::default(),
        priority: 50,
        handles_sensitive_data: false,
        cost_tier: "Medium".to_string(),
        api_endpoint: None,
    };

    registry.register_model(capabilities);

    let provider_models = registry.get_provider_models("test-provider");
    assert_eq!(provider_models.len(), 1);
    assert_eq!(provider_models[0], "test-model");

    let model_capabilities = registry.get_model_capabilities("test-provider", "test-model");
    assert!(model_capabilities.is_some());

    let capabilities = model_capabilities.expect("Model capabilities should be available in test");
    assert!(capabilities.supports_task(&TaskType::TextGeneration));
    assert!(capabilities.supports_model_type(&ModelType::LargeLanguageModel));
    assert_eq!(capabilities.max_context_size, 4096);
    assert!(capabilities.supports_streaming);
    assert!(!capabilities.supports_function_calling);
    assert!(!capabilities.supports_tool_use);
}

#[test]
fn test_load_save_registry() {
    let mut registry = ModelRegistry::new();

    // Add a test model
    let capabilities = ModelCapabilities {
        name: "test-model".to_string(),
        provider_id: "test-provider".to_string(),
        version: Some("1.0".to_string()),
        model_types: vec!["LargeLanguageModel".to_string()],
        task_types: vec!["TextGeneration".to_string()],
        max_context_size: Some(4096),
        supports_streaming: true,
        supports_function_calling: false,
        supports_tool_use: false,
        performance: PerformanceConfig::default(),
        resources: ResourceConfig::default(),
        cost: CostConfig::default(),
        priority: 50,
        handles_sensitive_data: false,
        cost_tier: "Medium".to_string(),
        api_endpoint: None,
    };

    registry.register_model(capabilities);

    // Save to tempfile
    let temp_file = NamedTempFile::new().expect("Failed to create temporary file for test");
    registry
        .save_to_file(temp_file.path())
        .expect("Failed to save registry to file in test");

    // Load into new registry
    let mut new_registry = ModelRegistry::new();
    new_registry
        .load_from_file(temp_file.path())
        .expect("Failed to load registry from file in test");

    // Verify model was loaded
    let model_capabilities = new_registry.get_model_capabilities("test-provider", "test-model");
    assert!(model_capabilities.is_some());
}

#[test]
fn test_global_registry() {
    let mut registry = ModelRegistry::new();

    // Add a test model
    let capabilities = ModelCapabilities {
        name: "global-test-model".to_string(),
        provider_id: "test-provider".to_string(),
        version: Some("1.0".to_string()),
        model_types: vec!["LargeLanguageModel".to_string()],
        task_types: vec!["TextGeneration".to_string()],
        max_context_size: Some(4096),
        supports_streaming: true,
        supports_function_calling: false,
        supports_tool_use: false,
        performance: PerformanceConfig::default(),
        resources: ResourceConfig::default(),
        cost: CostConfig::default(),
        priority: 50,
        handles_sensitive_data: false,
        cost_tier: "Medium".to_string(),
        api_endpoint: None,
    };

    registry.register_model(capabilities);

    // Set as global
    ModelRegistry::set_global(registry);

    // Get global
    let global = ModelRegistry::global();

    // Verify model exists in global registry
    let model_capabilities = global.get_model_capabilities("test-provider", "global-test-model");
    assert!(model_capabilities.is_some());
}

#[test]
fn test_get_cost_tier_string_mapping() {
    let cases = [
        ("free", CostTier::Free),
        ("Low", CostTier::Low),
        ("MEDIUM", CostTier::Medium),
        ("high", CostTier::High),
        ("weird", CostTier::High),
    ];
    for (tier_str, expected) in cases {
        let m = ModelCapabilities {
            name: "m".to_string(),
            provider_id: "p".to_string(),
            version: None,
            model_types: vec![],
            task_types: vec![],
            max_context_size: None,
            supports_streaming: false,
            supports_function_calling: false,
            supports_tool_use: false,
            performance: PerformanceConfig::default(),
            resources: ResourceConfig::default(),
            cost: CostConfig::default(),
            priority: 50,
            handles_sensitive_data: false,
            cost_tier: tier_str.to_string(),
            api_endpoint: None,
        };
        assert_eq!(m.get_cost_tier(), expected);
    }
}

#[test]
fn test_to_ai_capabilities_covers_types_and_performance_branches() {
    let m = ModelCapabilities {
        name: "custom".to_string(),
        provider_id: "pv".to_string(),
        version: None,
        model_types: vec!["Embedding".to_string(), "CustomModelX".to_string()],
        task_types: vec!["TextEmbedding".to_string(), "CustomTaskY".to_string()],
        max_context_size: None,
        supports_streaming: true,
        supports_function_calling: true,
        supports_tool_use: true,
        performance: PerformanceConfig {
            avg_latency_ms: Some(10),
            requests_per_second: Some(1.5),
            success_rate: Some(0.9),
            avg_tokens_per_second: Some(100.0),
            max_throughput_rps: Some(2.0),
            max_batch_size: Some(8),
            quality_score: Some(80),
        },
        resources: ResourceConfig {
            min_memory_mb: 512,
            min_gpu_memory_mb: Some(1024),
            min_cpu_cores: Some(4),
            requires_gpu: true,
            requires_internet: false,
            load_time_ms: Some(100),
            requires_specific_hardware: true,
            hardware_requirements: Some("gpu".to_string()),
        },
        cost: CostConfig {
            cost_per_1k_input_tokens: Some(0.1),
            cost_per_1k_output_tokens: Some(0.2),
            cost_per_request: Some(0.01),
            has_fixed_cost: true,
            is_free: false,
        },
        priority: 40,
        handles_sensitive_data: true,
        cost_tier: "Low".to_string(),
        api_endpoint: Some("http://x".to_string()),
    };
    let ai = m.to_ai_capabilities();
    assert!(ai.supports_model_type(&ModelType::Custom("CustomModelX".to_string())));
    assert!(ai.supports_task(&TaskType::Custom("CustomTaskY".to_string())));
    assert_eq!(ai.performance_metrics.max_batch_size, Some(8));
    assert_eq!(ai.resource_requirements.min_memory_mb, 512);
    assert!(ai.cost_metrics.has_fixed_cost);
}

#[test]
fn test_from_file_nested_models_json() {
    let dir = tempfile::tempdir().expect("should succeed");
    let path = dir.path().join("reg.json");
    let json = r#"{
        "models": {
            "prov": {
                "mid": {
                    "name": "mid",
                    "provider_id": "prov",
                    "version": null,
                    "model_types": ["LargeLanguageModel"],
                    "task_types": ["TextGeneration"],
                    "max_context_size": 2048,
                    "supports_streaming": false,
                    "supports_function_calling": false,
                    "supports_tool_use": false,
                    "performance": {},
                    "resources": {},
                    "cost": {},
                    "priority": 50,
                    "handles_sensitive_data": false,
                    "cost_tier": "Medium",
                    "api_endpoint": null
                }
            }
        }
    }"#;
    std::fs::write(&path, json).expect("should succeed");
    let reg = ModelRegistry::from_file(&path).expect("nested models file should parse");
    assert!(reg.get_provider_models("prov").contains(&"mid".to_string()));
}

#[test]
fn test_save_non_json_extension_roundtrip() {
    let mut registry = ModelRegistry::new();
    let capabilities = ModelCapabilities {
        name: "roundtrip".to_string(),
        provider_id: "rp".to_string(),
        version: Some("1".to_string()),
        model_types: vec!["LargeLanguageModel".to_string()],
        task_types: vec!["TextGeneration".to_string()],
        max_context_size: Some(512),
        supports_streaming: false,
        supports_function_calling: false,
        supports_tool_use: false,
        performance: PerformanceConfig::default(),
        resources: ResourceConfig::default(),
        cost: CostConfig::default(),
        priority: 50,
        handles_sensitive_data: false,
        cost_tier: "Medium".to_string(),
        api_endpoint: None,
    };
    registry.register_model(capabilities);

    let dir = tempfile::tempdir().expect("should succeed");
    let path = dir.path().join("registry.dat");
    registry.save_to_file(&path).expect("should succeed");
    let loaded = ModelRegistry::from_file(&path).expect("should succeed");
    assert!(loaded.get_model_capabilities("rp", "roundtrip").is_some());
}

#[test]
fn test_update_global_runs_closure() {
    ModelRegistry::update_global(|r| {
        r.register_model(ModelCapabilities {
            name: "ug".to_string(),
            provider_id: "ugp".to_string(),
            version: None,
            model_types: vec![],
            task_types: vec![],
            max_context_size: None,
            supports_streaming: false,
            supports_function_calling: false,
            supports_tool_use: false,
            performance: PerformanceConfig::default(),
            resources: ResourceConfig::default(),
            cost: CostConfig::default(),
            priority: 1,
            handles_sensitive_data: false,
            cost_tier: "Free".to_string(),
            api_endpoint: None,
        });
    });
    let g = ModelRegistry::global();
    assert!(g.get_model_capabilities("ugp", "ug").is_some());
}

#[test]
fn test_load_from_available_paths_loads_first_existing_file() {
    let dir = tempfile::tempdir().expect("should succeed");
    // Use non-`.json` extension so `load_from_file` uses direct `ModelRegistry` serde merge
    // (nested `.json` + `models` key path returns early without updating `self`).
    let path = dir.path().join("models.registry");
    let mut reg = ModelRegistry::new();
    reg.add_config_path(dir.path().join("missing.registry"));
    reg.add_config_path(path.clone());
    std::fs::write(
        &path,
        r#"{"models":{"p":{"m":{"name":"m","provider_id":"p","version":null,"model_types":["LargeLanguageModel"],"task_types":["TextGeneration"],"max_context_size":128,"supports_streaming":false,"supports_function_calling":false,"supports_tool_use":false,"performance":{},"resources":{},"cost":{},"priority":50,"handles_sensitive_data":false,"cost_tier":"Medium","api_endpoint":null}}}}"#,
    )
    .expect("should succeed");
    reg.load_from_available_paths().expect("should succeed");
    assert!(reg.get_model_capabilities("p", "m").is_some());
}

#[test]
fn test_to_ai_capabilities_model_and_task_type_variants() {
    let m = ModelCapabilities {
        name: "x".to_string(),
        provider_id: "p".to_string(),
        version: None,
        model_types: vec![
            "ImageGeneration".to_string(),
            "AudioTranscription".to_string(),
            "AudioGeneration".to_string(),
        ],
        task_types: vec![
            "ImageGeneration".to_string(),
            "DataAnalysis".to_string(),
            "FunctionExecution".to_string(),
        ],
        max_context_size: Some(100),
        supports_streaming: false,
        supports_function_calling: false,
        supports_tool_use: false,
        performance: PerformanceConfig::default(),
        resources: ResourceConfig::default(),
        cost: CostConfig::default(),
        priority: 50,
        handles_sensitive_data: false,
        cost_tier: "Medium".to_string(),
        api_endpoint: None,
    };
    let ai = m.to_ai_capabilities();
    assert!(ai.supports_model_type(&ModelType::ImageGeneration));
    assert!(ai.supports_task(&TaskType::DataAnalysis));
}

#[test]
fn test_get_providers_lists_keys() {
    let mut r = ModelRegistry::new();
    r.register_model(ModelCapabilities {
        name: "a".to_string(),
        provider_id: "pa".to_string(),
        version: None,
        model_types: vec![],
        task_types: vec![],
        max_context_size: None,
        supports_streaming: false,
        supports_function_calling: false,
        supports_tool_use: false,
        performance: PerformanceConfig::default(),
        resources: ResourceConfig::default(),
        cost: CostConfig::default(),
        priority: 1,
        handles_sensitive_data: false,
        cost_tier: "Low".to_string(),
        api_endpoint: None,
    });
    assert!(r.get_providers().contains(&"pa".to_string()));
}
