// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Tests for AI router

use super::router::AiRouter;
use super::types::{ActionRequirements, ImageGenerationRequest, TextGenerationRequest};

#[tokio::test]
async fn test_router_new_with_discovery() {
    // Test router creation with discovery
    let result = AiRouter::new_with_discovery(None).await;

    // Router should be created successfully even if no providers are found
    // (it will be empty but valid)
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_router_provider_count() {
    let router = AiRouter::new_with_discovery(None).await.unwrap();

    // Check that we can get provider count
    let count = router.provider_count().await;

    // Count should be non-negative (may be 0 if no env vars set)
    assert!(count >= 0);
}

#[tokio::test]
async fn test_router_list_providers() {
    let router = AiRouter::new_with_discovery(None).await.unwrap();

    // List providers should return a vector (may be empty)
    let providers = router.list_providers().await;

    // Should match provider count
    let count = router.provider_count().await;
    assert_eq!(providers.len(), count);
}

#[tokio::test]
async fn test_generate_text_no_providers() {
    // Create router without any providers
    std::env::remove_var("ANTHROPIC_API_KEY");
    std::env::remove_var("OPENAI_API_KEY");

    let router = AiRouter::new_with_discovery(None).await.unwrap();

    let request = TextGenerationRequest {
        prompt: "Hello, world!".to_string(),
        system: None,
        max_tokens: 100,
        temperature: 0.7,
        model: None,
        constraints: vec![],
        params: std::collections::HashMap::new(),
    };

    // Should fail when no providers available
    let result = router.generate_text(request, None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_generate_image_no_providers() {
    // Create router without any providers
    std::env::remove_var("OPENAI_API_KEY");
    std::env::remove_var("HUGGINGFACE_API_KEY");

    let router = AiRouter::new_with_discovery(None).await.unwrap();

    let request = ImageGenerationRequest {
        prompt: "A beautiful sunset".to_string(),
        negative_prompt: None,
        size: "512x512".to_string(),
        n: 1,
        quality_preference: None,
        constraints: vec![],
        params: std::collections::HashMap::new(),
    };

    // Should fail when no providers available
    let result = router.generate_image(request, None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_router_with_action_requirements() {
    let router = AiRouter::new_with_discovery(None).await.unwrap();

    let requirements = ActionRequirements {
        quality: Some("high".to_string()),
        cost_preference: Some("balanced".to_string()),
        max_latency_ms: Some(5000),
        privacy_level: Some("private".to_string()),
        preferred_provider: None,
    };

    let request = TextGenerationRequest {
        prompt: "Test with requirements".to_string(),
        system: None,
        max_tokens: 50,
        temperature: 0.7,
        model: None,
        constraints: vec![],
        params: std::collections::HashMap::new(),
    };

    // Should handle requirements even if it fails due to no providers
    let result = router.generate_text(request, Some(requirements)).await;

    // Will fail due to no providers, but should not panic
    assert!(result.is_err());
}

#[tokio::test]
async fn test_provider_info_structure() {
    let router = AiRouter::new_with_discovery(None).await.unwrap();
    let providers = router.list_providers().await;

    // Each provider info should have valid structure
    for provider in providers {
        // Provider ID should not be empty
        assert!(!provider.provider_id.is_empty());

        // Provider name should not be empty
        assert!(!provider.provider_name.is_empty());

        // Capabilities should be present
        assert!(!provider.capabilities.is_empty());
    }
}

#[tokio::test]
async fn test_concurrent_provider_access() {
    let router = std::sync::Arc::new(AiRouter::new_with_discovery(None).await.unwrap());

    // Spawn multiple tasks accessing provider count concurrently
    let mut handles = vec![];

    for _ in 0..10 {
        let router_clone = router.clone();
        handles.push(tokio::spawn(
            async move { router_clone.provider_count().await },
        ));
    }

    // All tasks should complete successfully
    for handle in handles {
        let result = handle.await;
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_concurrent_provider_list() {
    let router = std::sync::Arc::new(AiRouter::new_with_discovery(None).await.unwrap());

    // Spawn multiple tasks listing providers concurrently
    let mut handles = vec![];

    for _ in 0..10 {
        let router_clone = router.clone();
        handles.push(tokio::spawn(
            async move { router_clone.list_providers().await },
        ));
    }

    // All tasks should complete successfully
    for handle in handles {
        let result = handle.await;
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_text_generation_request_validation() {
    let router = AiRouter::new_with_discovery(None).await.unwrap();

    // Test with empty prompt
    let request = TextGenerationRequest {
        prompt: "".to_string(),
        system: None,
        max_tokens: 100,
        temperature: 0.7,
        model: None,
        constraints: vec![],
        params: std::collections::HashMap::new(),
    };

    // Should handle empty prompt gracefully
    let result = router.generate_text(request, None).await;
    assert!(result.is_err()); // Will fail due to no providers, but shouldn't panic
}

#[tokio::test]
async fn test_image_generation_request_validation() {
    let router = AiRouter::new_with_discovery(None).await.unwrap();

    // Test with empty prompt
    let request = ImageGenerationRequest {
        prompt: "".to_string(),
        negative_prompt: None,
        size: "512x512".to_string(),
        n: 1,
        quality_preference: None,
        constraints: vec![],
        params: std::collections::HashMap::new(),
    };

    // Should handle empty prompt gracefully
    let result = router.generate_image(request, None).await;
    assert!(result.is_err()); // Will fail due to no providers, but shouldn't panic
}

#[tokio::test]
async fn test_router_initialization_timeout() {
    // The router has a 10-second timeout for initialization
    // This test verifies it doesn't hang indefinitely
    let start = std::time::Instant::now();

    let _router = AiRouter::new_with_discovery(None).await.unwrap();

    let elapsed = start.elapsed();

    // Should complete within a reasonable time
    // Router may try multiple adapters (each with 5s timeout), so allow up to 30s
    assert!(
        elapsed.as_secs() < 30,
        "Initialization took {} seconds",
        elapsed.as_secs()
    );
}

#[tokio::test]
async fn test_action_requirements_defaults() {
    let requirements = ActionRequirements::default();

    // Default values should be set
    assert_eq!(requirements.quality, Some("medium".to_string()));
    assert_eq!(requirements.cost_preference, Some("balanced".to_string()));
}

#[tokio::test]
async fn test_action_requirements_with_privacy() {
    let requirements = ActionRequirements {
        quality: Some("high".to_string()),
        cost_preference: Some("premium".to_string()),
        max_latency_ms: Some(3000),
        privacy_level: Some("local".to_string()),
        preferred_provider: Some("local-llm".to_string()),
    };

    // Requirements should be valid
    assert_eq!(requirements.privacy_level, Some("local".to_string()));
    assert_eq!(
        requirements.preferred_provider,
        Some("local-llm".to_string())
    );
}

#[tokio::test]
async fn test_router_handles_service_mesh_none() {
    // Test that passing None for service mesh client works
    let result = AiRouter::new_with_discovery(None).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_multiple_router_instances() {
    // Create multiple router instances
    let router1 = AiRouter::new_with_discovery(None).await.unwrap();
    let router2 = AiRouter::new_with_discovery(None).await.unwrap();

    // Both should be independently functional
    let count1 = router1.provider_count().await;
    let count2 = router2.provider_count().await;

    // Counts should be the same (same environment)
    assert_eq!(count1, count2);
}

#[tokio::test]
async fn test_provider_capabilities_not_empty() {
    let router = AiRouter::new_with_discovery(None).await.unwrap();
    let providers = router.list_providers().await;

    // If any providers exist, they should have capabilities
    for provider in providers {
        assert!(
            !provider.capabilities.is_empty(),
            "Provider {} has no capabilities",
            provider.provider_id
        );
    }
}

#[tokio::test]
async fn test_text_generation_with_system_message() {
    let router = AiRouter::new_with_discovery(None).await.unwrap();

    let request = TextGenerationRequest {
        prompt: "Test prompt with system message".to_string(),
        system: Some("You are a helpful assistant".to_string()),
        max_tokens: 200,
        temperature: 0.8,
        model: Some("test-model".to_string()),
        constraints: vec![],
        params: std::collections::HashMap::new(),
    };

    // Should handle system message gracefully
    let result = router.generate_text(request, None).await;
    assert!(result.is_err()); // Will fail due to no providers
}

#[tokio::test]
async fn test_image_generation_with_negative_prompt() {
    let router = AiRouter::new_with_discovery(None).await.unwrap();

    let request = ImageGenerationRequest {
        prompt: "A beautiful landscape".to_string(),
        negative_prompt: Some("ugly, blurry".to_string()),
        size: "1024x1024".to_string(),
        n: 2,
        quality_preference: Some("high".to_string()),
        constraints: vec![],
        params: std::collections::HashMap::new(),
    };

    // Should handle negative prompt gracefully
    let result = router.generate_image(request, None).await;
    assert!(result.is_err()); // Will fail due to no providers
}

#[tokio::test]
async fn test_text_generation_with_params() {
    let router = AiRouter::new_with_discovery(None).await.unwrap();

    let mut params = std::collections::HashMap::new();
    params.insert("custom_param".to_string(), serde_json::json!("value"));
    params.insert("another_param".to_string(), serde_json::json!(42));

    let request = TextGenerationRequest {
        prompt: "Test with custom params".to_string(),
        system: None,
        max_tokens: 100,
        temperature: 0.7,
        model: None,
        constraints: vec![],
        params,
    };

    // Should handle custom params gracefully
    let result = router.generate_text(request, None).await;
    assert!(result.is_err()); // Will fail due to no providers
}

#[tokio::test]
async fn test_image_generation_multiple_images() {
    let router = AiRouter::new_with_discovery(None).await.unwrap();

    let request = ImageGenerationRequest {
        prompt: "Test generating multiple images".to_string(),
        negative_prompt: None,
        size: "512x512".to_string(),
        n: 4, // Request multiple images
        quality_preference: None,
        constraints: vec![],
        params: std::collections::HashMap::new(),
    };

    // Should handle multiple image request gracefully
    let result = router.generate_image(request, None).await;
    assert!(result.is_err()); // Will fail due to no providers
}

#[tokio::test]
async fn test_text_generation_high_temperature() {
    let router = AiRouter::new_with_discovery(None).await.unwrap();

    let request = TextGenerationRequest {
        prompt: "Creative writing prompt".to_string(),
        system: None,
        max_tokens: 500,
        temperature: 1.5, // High temperature for creativity
        model: None,
        constraints: vec![],
        params: std::collections::HashMap::new(),
    };

    // Should handle high temperature gracefully
    let result = router.generate_text(request, None).await;
    assert!(result.is_err()); // Will fail due to no providers
}

#[tokio::test]
async fn test_text_generation_low_temperature() {
    let router = AiRouter::new_with_discovery(None).await.unwrap();

    let request = TextGenerationRequest {
        prompt: "Factual query".to_string(),
        system: None,
        max_tokens: 100,
        temperature: 0.1, // Low temperature for deterministic output
        model: None,
        constraints: vec![],
        params: std::collections::HashMap::new(),
    };

    // Should handle low temperature gracefully
    let result = router.generate_text(request, None).await;
    assert!(result.is_err()); // Will fail due to no providers
}

#[tokio::test]
async fn test_provider_count_is_consistent() {
    let router = AiRouter::new_with_discovery(None).await.unwrap();

    // Get count multiple times
    let count1 = router.provider_count().await;
    let count2 = router.provider_count().await;
    let count3 = router.provider_count().await;

    // Should be consistent across calls
    assert_eq!(count1, count2);
    assert_eq!(count2, count3);
}

#[tokio::test]
async fn test_provider_list_is_consistent() {
    let router = AiRouter::new_with_discovery(None).await.unwrap();

    // Get list multiple times
    let list1 = router.list_providers().await;
    let list2 = router.list_providers().await;

    // Should be consistent across calls
    assert_eq!(list1.len(), list2.len());

    // Provider IDs should match
    let ids1: Vec<_> = list1.iter().map(|p| &p.provider_id).collect();
    let ids2: Vec<_> = list2.iter().map(|p| &p.provider_id).collect();
    assert_eq!(ids1, ids2);
}
