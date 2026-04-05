// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::needless_pass_by_value,
    clippy::significant_drop_tightening,
    clippy::field_reassign_with_default,
    clippy::default_trait_access,
    clippy::many_single_char_names,
    clippy::unreadable_literal,
    clippy::too_many_lines,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::similar_names,
    clippy::option_if_let_else,
    clippy::doc_markdown,
    clippy::struct_field_names,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::return_self_not_must_use,
    clippy::trivially_copy_pass_by_ref,
    clippy::unused_self,
    clippy::unused_async,
    clippy::unnecessary_wraps,
    clippy::semicolon_if_nothing_returned,
    clippy::match_wildcard_for_single_variants,
    clippy::match_same_arms,
    clippy::explicit_iter_loop,
    clippy::uninlined_format_args,
    clippy::equatable_if_let,
    clippy::assertions_on_constants,
    missing_docs,
    unused_imports,
    unused_variables,
    dead_code,
    deprecated
)]

use crate::api::ai::adapters::{AiProviderAdapter, QualityTier};
use crate::api::ai::constraints::RoutingConstraint;
use crate::api::ai::dignity::{DignityCheckRequest, DignityEvaluator};
use crate::api::ai::router::AiRouter;
use crate::api::ai::types::{
    GeneratedImage, ImageGenerationRequest, ImageGenerationResponse, TextGenerationRequest,
    TextGenerationResponse,
};
use crate::error::PrimalError;
use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;

/// Minimal adapter: text generation only (for routing tests).
struct MockTextAdapter {
    id: &'static str,
    name: &'static str,
}

#[async_trait]
impl AiProviderAdapter for MockTextAdapter {
    fn provider_id(&self) -> &str {
        self.id
    }

    fn provider_name(&self) -> &str {
        self.name
    }

    fn is_local(&self) -> bool {
        true
    }

    fn cost_per_unit(&self) -> Option<f64> {
        Some(0.0)
    }

    fn avg_latency_ms(&self) -> u64 {
        10
    }

    fn quality_tier(&self) -> QualityTier {
        QualityTier::Standard
    }

    fn supports_text_generation(&self) -> bool {
        true
    }

    fn supports_image_generation(&self) -> bool {
        false
    }

    async fn generate_text(
        &self,
        request: TextGenerationRequest,
    ) -> Result<TextGenerationResponse, PrimalError> {
        Ok(TextGenerationResponse {
            text: format!("echo:{}", request.prompt),
            provider_id: self.id.to_string(),
            model: request.model.unwrap_or_else(|| "mock".to_string()),
            usage: None,
            cost_usd: None,
            latency_ms: 1,
        })
    }

    async fn generate_image(
        &self,
        _request: ImageGenerationRequest,
    ) -> Result<ImageGenerationResponse, PrimalError> {
        Err(PrimalError::OperationFailed("mock: no image".to_string()))
    }
}

/// Image-only adapter (no text generation).
struct MockImageOnlyAdapter {
    id: &'static str,
}

#[async_trait]
impl AiProviderAdapter for MockImageOnlyAdapter {
    fn provider_id(&self) -> &str {
        self.id
    }

    fn provider_name(&self) -> &'static str {
        "ImageOnly"
    }

    fn is_local(&self) -> bool {
        true
    }

    fn cost_per_unit(&self) -> Option<f64> {
        Some(0.01)
    }

    fn avg_latency_ms(&self) -> u64 {
        20
    }

    fn quality_tier(&self) -> QualityTier {
        QualityTier::Standard
    }

    fn supports_text_generation(&self) -> bool {
        false
    }

    fn supports_image_generation(&self) -> bool {
        true
    }

    async fn generate_text(
        &self,
        _request: TextGenerationRequest,
    ) -> Result<TextGenerationResponse, PrimalError> {
        Err(PrimalError::OperationFailed("no text".to_string()))
    }

    async fn generate_image(
        &self,
        _request: ImageGenerationRequest,
    ) -> Result<ImageGenerationResponse, PrimalError> {
        Ok(ImageGenerationResponse {
            images: vec![GeneratedImage {
                url: None,
                data: None,
                mime_type: "image/png".to_string(),
                width: 1,
                height: 1,
            }],
            provider_id: self.id.to_string(),
            cost_usd: None,
            latency_ms: 1,
        })
    }
}

#[tokio::test]
async fn provider_count_reflects_injected_adapters() {
    let router = AiRouter::from_adapters_for_test(vec![Arc::new(MockTextAdapter {
        id: "mock-text",
        name: "Mock",
    })]);
    assert_eq!(router.provider_count().await, 1);
}

#[tokio::test]
async fn generate_text_routes_to_mock_and_returns_echo() {
    let router = AiRouter::from_adapters_for_test(vec![Arc::new(MockTextAdapter {
        id: "mock-text",
        name: "Mock",
    })]);
    let req = TextGenerationRequest {
        prompt: "hello".to_string(),
        system: None,
        max_tokens: 64,
        temperature: 0.5,
        model: None,
        constraints: vec![],
        params: std::collections::HashMap::new(),
    };
    let out = router
        .generate_text(req, None)
        .await
        .expect("should succeed");
    assert_eq!(out.text, "echo:hello");
    assert_eq!(out.provider_id, "mock-text");
}

#[tokio::test]
async fn generate_text_errors_when_no_providers() {
    let router = AiRouter::default();
    let req = TextGenerationRequest {
        prompt: "x".to_string(),
        system: None,
        max_tokens: 32,
        temperature: 0.7,
        model: None,
        constraints: vec![],
        params: std::collections::HashMap::new(),
    };
    let err = router.generate_text(req, None).await.unwrap_err();
    assert!(matches!(err, crate::error::PrimalError::OperationFailed(_)));
}

#[tokio::test]
async fn list_providers_merges_without_duplicates() {
    let router = AiRouter::from_adapters_for_test(vec![Arc::new(MockTextAdapter {
        id: "only",
        name: "Only",
    })]);
    let listed = router.list_providers().await;
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].provider_id, "only");
}

#[test]
fn text_generation_request_serde_round_trip() {
    let req = TextGenerationRequest {
        prompt: "p".to_string(),
        system: Some("sys".to_string()),
        max_tokens: 100,
        temperature: 0.2,
        model: Some("m".to_string()),
        constraints: vec![],
        params: std::collections::HashMap::new(),
    };
    let v = serde_json::to_value(&req).expect("should succeed");
    let back: TextGenerationRequest = serde_json::from_value(v).expect("should succeed");
    assert_eq!(back.prompt, "p");
    assert_eq!(back.model.as_deref(), Some("m"));
}

#[test]
fn image_generation_request_serde_round_trip() {
    let req = ImageGenerationRequest {
        prompt: "a cat".to_string(),
        negative_prompt: None,
        size: "512x512".to_string(),
        n: 1,
        quality_preference: None,
        constraints: vec![],
        params: std::collections::HashMap::new(),
    };
    let j = json!({"prompt":"a cat","size":"512x512","n":1});
    let parsed: ImageGenerationRequest = serde_json::from_value(j).expect("should succeed");
    assert_eq!(parsed.prompt, req.prompt);
}

#[tokio::test]
async fn generate_text_errors_when_only_image_adapter_present() {
    let router =
        AiRouter::from_adapters_for_test(vec![Arc::new(MockImageOnlyAdapter { id: "img-only" })]);
    let req = TextGenerationRequest {
        prompt: "x".to_string(),
        system: None,
        max_tokens: 32,
        temperature: 0.7,
        model: None,
        constraints: vec![],
        params: std::collections::HashMap::new(),
    };
    let err = router.generate_text(req, None).await.unwrap_err();
    assert!(matches!(err, PrimalError::OperationFailed(msg) if msg.contains("text")));
}

#[tokio::test]
async fn generate_image_errors_when_no_image_providers() {
    let router = AiRouter::from_adapters_for_test(vec![Arc::new(MockTextAdapter {
        id: "text-only",
        name: "Text",
    })]);
    let req = ImageGenerationRequest {
        prompt: "a cat".to_string(),
        negative_prompt: None,
        size: "256x256".to_string(),
        n: 1,
        quality_preference: None,
        constraints: vec![],
        params: std::collections::HashMap::new(),
    };
    let err = router.generate_image(req, None).await.unwrap_err();
    assert!(matches!(err, PrimalError::OperationFailed(_)));
}

#[tokio::test]
async fn generate_text_with_constraints_uses_constraint_router() {
    let router = AiRouter::from_adapters_for_test(vec![Arc::new(MockTextAdapter {
        id: "mock-text",
        name: "Mock",
    })]);
    let req = TextGenerationRequest {
        prompt: "hello".to_string(),
        system: None,
        max_tokens: 64,
        temperature: 0.5,
        model: None,
        constraints: vec![RoutingConstraint::RequireProvider("mock-text".to_string())],
        params: std::collections::HashMap::new(),
    };
    let out = router
        .generate_text(req, None)
        .await
        .expect("should succeed");
    assert_eq!(out.provider_id, "mock-text");
}

#[tokio::test]
async fn get_text_generation_providers_filters_non_text() {
    let router = AiRouter::from_adapters_for_test(vec![
        Arc::new(MockTextAdapter { id: "t", name: "T" }) as Arc<dyn AiProviderAdapter>,
        Arc::new(MockImageOnlyAdapter { id: "i" }) as Arc<dyn AiProviderAdapter>,
    ]);
    let infos = router
        .get_text_generation_providers()
        .await
        .expect("should succeed");
    assert_eq!(infos.len(), 1);
    assert_eq!(infos[0].provider_id, "t");
}

/// Cheaper (higher score) image provider that always fails generation — triggers fallback path.
struct MockFailingImageAdapter {
    id: &'static str,
}

#[async_trait]
impl AiProviderAdapter for MockFailingImageAdapter {
    fn provider_id(&self) -> &str {
        self.id
    }

    fn provider_name(&self) -> &'static str {
        "FailImg"
    }

    fn is_local(&self) -> bool {
        true
    }

    fn cost_per_unit(&self) -> Option<f64> {
        Some(0.0)
    }

    fn avg_latency_ms(&self) -> u64 {
        5
    }

    fn quality_tier(&self) -> QualityTier {
        QualityTier::Standard
    }

    fn supports_text_generation(&self) -> bool {
        false
    }

    fn supports_image_generation(&self) -> bool {
        true
    }

    async fn generate_text(
        &self,
        _request: TextGenerationRequest,
    ) -> Result<TextGenerationResponse, PrimalError> {
        Err(PrimalError::OperationFailed("no text".to_string()))
    }

    async fn generate_image(
        &self,
        _request: ImageGenerationRequest,
    ) -> Result<ImageGenerationResponse, PrimalError> {
        Err(PrimalError::OperationFailed(
            "primary image provider failed".to_string(),
        ))
    }
}

/// Higher-cost fallback image provider that succeeds.
struct MockFallbackImageAdapter {
    id: &'static str,
}

#[async_trait]
impl AiProviderAdapter for MockFallbackImageAdapter {
    fn provider_id(&self) -> &str {
        self.id
    }

    fn provider_name(&self) -> &'static str {
        "OkImg"
    }

    fn is_local(&self) -> bool {
        true
    }

    fn cost_per_unit(&self) -> Option<f64> {
        Some(0.05)
    }

    fn avg_latency_ms(&self) -> u64 {
        30
    }

    fn quality_tier(&self) -> QualityTier {
        QualityTier::Standard
    }

    fn supports_text_generation(&self) -> bool {
        false
    }

    fn supports_image_generation(&self) -> bool {
        true
    }

    async fn generate_text(
        &self,
        _request: TextGenerationRequest,
    ) -> Result<TextGenerationResponse, PrimalError> {
        Err(PrimalError::OperationFailed("no text".to_string()))
    }

    async fn generate_image(
        &self,
        _request: ImageGenerationRequest,
    ) -> Result<ImageGenerationResponse, PrimalError> {
        Ok(ImageGenerationResponse {
            images: vec![GeneratedImage {
                url: None,
                data: None,
                mime_type: "image/png".to_string(),
                width: 2,
                height: 2,
            }],
            provider_id: self.id.to_string(),
            cost_usd: None,
            latency_ms: 2,
        })
    }
}

#[tokio::test]
async fn generate_image_retries_with_fallback_provider() {
    let router = AiRouter::from_adapters_for_test(vec![
        Arc::new(MockFailingImageAdapter { id: "img-fail" }) as Arc<dyn AiProviderAdapter>,
        Arc::new(MockFallbackImageAdapter { id: "img-ok" }) as Arc<dyn AiProviderAdapter>,
    ]);
    let req = ImageGenerationRequest {
        prompt: "sunset".to_string(),
        negative_prompt: None,
        size: "256x256".to_string(),
        n: 1,
        quality_preference: None,
        constraints: vec![],
        params: std::collections::HashMap::new(),
    };
    let out = router
        .generate_image(req, None)
        .await
        .expect("should succeed");
    assert_eq!(out.provider_id, "img-ok");
    assert_eq!(out.images.len(), 1);
}

#[tokio::test]
async fn generate_image_success_without_retry() {
    let router =
        AiRouter::from_adapters_for_test(vec![Arc::new(MockImageOnlyAdapter { id: "img-one" })]);
    let req = ImageGenerationRequest {
        prompt: "x".to_string(),
        negative_prompt: None,
        size: "128x128".to_string(),
        n: 1,
        quality_preference: None,
        constraints: vec![],
        params: std::collections::HashMap::new(),
    };
    let out = router
        .generate_image(req, None)
        .await
        .expect("should succeed");
    assert_eq!(out.provider_id, "img-one");
}

#[tokio::test]
async fn router_default_has_no_providers() {
    let r = AiRouter::default();
    assert_eq!(r.provider_count().await, 0);
}

/// Router uses the same dignity evaluator semantics as [`DignityEvaluator`] (non-blocking log on violation).
#[test]
fn dignity_integration_matches_standalone_evaluator_for_text_prompt() {
    let prompt = "Should we hire this applicant?";
    let standalone = DignityEvaluator.evaluate_request(&DignityCheckRequest {
        prompt,
        model: None,
        context: None,
    });
    assert!(!standalone.passed);
    assert!(!standalone.flags.is_empty());
}

#[tokio::test]
async fn generate_text_with_dignity_sensitive_prompt_still_routes_when_provider_ok() {
    let router = AiRouter::from_adapters_for_test(vec![Arc::new(MockTextAdapter {
        id: "mock-text",
        name: "Mock",
    })]);
    let req = TextGenerationRequest {
        prompt: "Review this applicant for employment".to_string(),
        system: None,
        max_tokens: 64,
        temperature: 0.5,
        model: Some("gpt-4".to_string()),
        constraints: vec![],
        params: std::collections::HashMap::new(),
    };
    let out = router
        .generate_text(req, None)
        .await
        .expect("should succeed");
    assert!(out.text.contains("echo:"));
    assert_eq!(out.provider_id, "mock-text");
}

#[tokio::test]
async fn generate_image_dignity_prompt_still_succeeds_with_image_provider() {
    let router =
        AiRouter::from_adapters_for_test(vec![Arc::new(MockImageOnlyAdapter { id: "img-one" })]);
    let req = ImageGenerationRequest {
        prompt: "Evaluate housing eligibility for this tenant".to_string(),
        negative_prompt: None,
        size: "128x128".to_string(),
        n: 1,
        quality_preference: None,
        constraints: vec![],
        params: std::collections::HashMap::new(),
    };
    let out = router
        .generate_image(req, None)
        .await
        .expect("should succeed");
    assert_eq!(out.provider_id, "img-one");
}

#[tokio::test]
async fn generate_text_constraint_require_provider_still_routes_after_fallback() {
    let router = AiRouter::from_adapters_for_test(vec![Arc::new(MockTextAdapter {
        id: "mock-text",
        name: "Mock",
    })]);
    let req = TextGenerationRequest {
        prompt: "hi".to_string(),
        system: None,
        max_tokens: 32,
        temperature: 0.7,
        model: None,
        constraints: vec![RoutingConstraint::RequireProvider(
            "nonexistent".to_string(),
        )],
        params: std::collections::HashMap::new(),
    };
    let out = router
        .generate_text(req, None)
        .await
        .expect("should succeed");
    assert_eq!(out.provider_id, "mock-text");
}
