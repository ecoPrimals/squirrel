// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Tests for AI Inference and Provider Selection
//!
//! TRUE PRIMAL: Provider selection is capability-based, not vendor-based.
//! All tests verify agnostic behavior -- the selector returns "auto" and
//! delegates concrete provider resolution to the AI router's capability
//! discovery at runtime.

#[cfg(test)]
mod tests {
    use super::super::ai_inference::{AIInferenceRequest, AIProviderSelection};
    use std::collections::HashMap;

    // Helper to create a basic AI inference request
    fn create_test_request(task_type: &str, model: Option<String>) -> AIInferenceRequest {
        AIInferenceRequest {
            task_type: task_type.to_string(),
            messages: vec![serde_json::json!({"role": "user", "content": "test message"})],
            model,
            parameters: HashMap::new(),
        }
    }

    #[test]
    fn test_provider_selection_with_model_returns_auto() {
        // When a model is specified, provider selection returns "auto"
        // and lets the AI router resolve the model to a discovered provider
        let request = create_test_request("chat", Some("gpt-4".to_string()));
        let provider = AIProviderSelection::select_provider(&request).unwrap();
        assert_eq!(provider, "auto");
    }

    #[test]
    fn test_provider_selection_any_model_returns_auto() {
        // Any model name should return "auto" -- no vendor name mapping
        for model in &[
            "gpt-4",
            "claude-3-opus",
            "llama2",
            "mistral",
            "custom-model",
        ] {
            let request = create_test_request("chat", Some(model.to_string()));
            let provider = AIProviderSelection::select_provider(&request).unwrap();
            assert_eq!(provider, "auto", "Model '{}' should return 'auto'", model);
        }
    }

    #[test]
    fn test_provider_selection_no_model_returns_auto() {
        // Without a model, provider selection uses env or defaults to "auto"
        let request = create_test_request("chat", None);
        let provider = AIProviderSelection::select_provider(&request).unwrap();
        assert_eq!(provider, "auto");
    }

    #[test]
    fn test_provider_selection_text_generation_task() {
        let request = create_test_request("text_generation", None);
        let provider = AIProviderSelection::select_provider(&request).unwrap();
        assert_eq!(provider, "auto");
    }

    #[test]
    fn test_provider_selection_local_task() {
        // "local" task type prefers local providers, but still returns "auto"
        // unless AI_DEFAULT_PROVIDER is set
        let request = create_test_request("local", None);
        let provider = AIProviderSelection::select_provider(&request).unwrap();
        assert_eq!(provider, "auto");
    }

    #[test]
    fn test_provider_selection_private_task() {
        let request = create_test_request("private", None);
        let provider = AIProviderSelection::select_provider(&request).unwrap();
        assert_eq!(provider, "auto");
    }

    #[test]
    fn test_provider_selection_unknown_task_defaults() {
        let request = create_test_request("unknown_task_type", None);
        let provider = AIProviderSelection::select_provider(&request).unwrap();
        assert_eq!(provider, "auto");
    }

    #[test]
    fn test_provider_selection_empty_task_defaults() {
        let request = create_test_request("", None);
        let provider = AIProviderSelection::select_provider(&request).unwrap();
        assert_eq!(provider, "auto");
    }

    #[test]
    fn test_ai_inference_request_creation() {
        let request = create_test_request("chat", Some("gpt-4".to_string()));

        assert_eq!(request.task_type, "chat");
        assert_eq!(request.model, Some("gpt-4".to_string()));
        assert_eq!(request.messages.len(), 1);
        assert!(request.parameters.is_empty());
    }

    #[test]
    fn test_ai_inference_request_with_parameters() {
        let mut params = HashMap::new();
        params.insert("temperature".to_string(), serde_json::json!(0.7));
        params.insert("max_tokens".to_string(), serde_json::json!(1000));

        let request = AIInferenceRequest {
            task_type: "chat".to_string(),
            messages: vec![],
            model: Some("gpt-4".to_string()),
            parameters: params.clone(),
        };

        assert_eq!(request.parameters.len(), 2);
        assert_eq!(
            request.parameters.get("temperature"),
            Some(&serde_json::json!(0.7))
        );
    }

    #[test]
    fn test_ai_inference_request_multiple_messages() {
        let messages = vec![
            serde_json::json!({"role": "system", "content": "You are helpful"}),
            serde_json::json!({"role": "user", "content": "Hello"}),
            serde_json::json!({"role": "assistant", "content": "Hi there"}),
        ];

        let request = AIInferenceRequest {
            task_type: "chat".to_string(),
            messages: messages.clone(),
            model: None,
            parameters: HashMap::new(),
        };

        assert_eq!(request.messages.len(), 3);
    }

    #[test]
    fn test_ai_inference_request_serialization() {
        let request = create_test_request("chat", Some("gpt-4".to_string()));

        let json = serde_json::to_string(&request);
        assert!(json.is_ok(), "Request should be serializable");

        let json_str = json.unwrap();
        let deserialized: Result<AIInferenceRequest, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok(), "Request should be deserializable");
    }

    #[test]
    fn test_provider_selection_consistent_results() {
        let request = create_test_request("chat", Some("gpt-4".to_string()));

        // Should return consistent results
        let provider1 = AIProviderSelection::select_provider(&request).unwrap();
        let provider2 = AIProviderSelection::select_provider(&request).unwrap();

        assert_eq!(provider1, provider2);
    }

    #[test]
    fn test_provider_selection_empty_model_returns_auto() {
        let request = create_test_request("chat", Some("".to_string()));
        let provider = AIProviderSelection::select_provider(&request).unwrap();
        assert_eq!(provider, "auto");
    }

    #[test]
    fn test_ai_inference_request_clone() {
        let request = create_test_request("chat", Some("test-model".to_string()));
        let cloned = request.clone();

        assert_eq!(cloned.task_type, request.task_type);
        assert_eq!(cloned.model, request.model);
        assert_eq!(cloned.messages.len(), request.messages.len());
    }
}
