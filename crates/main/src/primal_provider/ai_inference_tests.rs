//! Tests for AI Inference and Provider Selection

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
    fn test_provider_selection_gpt_model() {
        let request = create_test_request("chat", Some("gpt-4".to_string()));
        let provider = AIProviderSelection::select_provider(&request).unwrap();
        assert_eq!(provider, "openai");
    }

    #[test]
    fn test_provider_selection_gpt_35_model() {
        let request = create_test_request("chat", Some("gpt-3.5-turbo".to_string()));
        let provider = AIProviderSelection::select_provider(&request).unwrap();
        assert_eq!(provider, "openai");
    }

    #[test]
    fn test_provider_selection_claude_model() {
        let request = create_test_request("chat", Some("claude-3-opus".to_string()));
        let provider = AIProviderSelection::select_provider(&request).unwrap();
        assert_eq!(provider, "anthropic");
    }

    #[test]
    fn test_provider_selection_claude_sonnet() {
        let request = create_test_request("analysis", Some("claude-3-sonnet".to_string()));
        let provider = AIProviderSelection::select_provider(&request).unwrap();
        assert_eq!(provider, "anthropic");
    }

    #[test]
    fn test_provider_selection_llama_model() {
        let request = create_test_request("chat", Some("llama2".to_string()));
        let provider = AIProviderSelection::select_provider(&request).unwrap();
        assert_eq!(provider, "ollama");
    }

    #[test]
    fn test_provider_selection_mistral_model() {
        let request = create_test_request("chat", Some("mistral".to_string()));
        let provider = AIProviderSelection::select_provider(&request).unwrap();
        assert_eq!(provider, "ollama");
    }

    #[test]
    fn test_provider_selection_openai_in_name() {
        let request = create_test_request("chat", Some("openai-custom".to_string()));
        let provider = AIProviderSelection::select_provider(&request).unwrap();
        assert_eq!(provider, "openai");
    }

    #[test]
    fn test_provider_selection_anthropic_in_name() {
        let request = create_test_request("chat", Some("anthropic-custom".to_string()));
        let provider = AIProviderSelection::select_provider(&request).unwrap();
        assert_eq!(provider, "anthropic");
    }

    #[test]
    fn test_provider_selection_text_generation_task() {
        let request = create_test_request("text_generation", None);
        let provider = AIProviderSelection::select_provider(&request).unwrap();
        // Should use default provider (openai or env var)
        assert!(!provider.is_empty());
    }

    #[test]
    fn test_provider_selection_chat_task() {
        let request = create_test_request("chat", None);
        let provider = AIProviderSelection::select_provider(&request).unwrap();
        assert!(!provider.is_empty());
    }

    #[test]
    fn test_provider_selection_code_generation_task() {
        let request = create_test_request("code_generation", None);
        let provider = AIProviderSelection::select_provider(&request).unwrap();
        assert_eq!(provider, "openai");
    }

    #[test]
    fn test_provider_selection_analysis_task() {
        let request = create_test_request("analysis", None);
        let provider = AIProviderSelection::select_provider(&request).unwrap();
        assert_eq!(provider, "anthropic");
    }

    #[test]
    fn test_provider_selection_reasoning_task() {
        let request = create_test_request("reasoning", None);
        let provider = AIProviderSelection::select_provider(&request).unwrap();
        assert_eq!(provider, "anthropic");
    }

    #[test]
    fn test_provider_selection_local_task() {
        let request = create_test_request("local", None);
        let provider = AIProviderSelection::select_provider(&request).unwrap();
        assert_eq!(provider, "ollama");
    }

    #[test]
    fn test_provider_selection_private_task() {
        let request = create_test_request("private", None);
        let provider = AIProviderSelection::select_provider(&request).unwrap();
        assert_eq!(provider, "ollama");
    }

    #[test]
    fn test_provider_selection_unknown_task_defaults() {
        let request = create_test_request("unknown_task_type", None);
        let provider = AIProviderSelection::select_provider(&request).unwrap();
        assert_eq!(provider, "openai");
    }

    #[test]
    fn test_provider_selection_empty_task_defaults() {
        let request = create_test_request("", None);
        let provider = AIProviderSelection::select_provider(&request).unwrap();
        assert_eq!(provider, "openai");
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
    fn test_provider_selection_model_precedence_over_task() {
        // Model should take precedence over task type
        let request = create_test_request("analysis", Some("gpt-4".to_string()));
        let provider = AIProviderSelection::select_provider(&request).unwrap();
        // Should choose openai based on model, not anthropic based on task
        assert_eq!(provider, "openai");
    }

    #[test]
    fn test_provider_selection_case_sensitive_model() {
        let request = create_test_request("chat", Some("GPT-4".to_string()));
        let provider = AIProviderSelection::select_provider(&request).unwrap();
        // Case sensitive - won't match gpt- prefix
        // Should fall back to task type
        assert!(!provider.is_empty());
    }

    #[test]
    fn test_provider_selection_partial_model_name() {
        let request = create_test_request("chat", Some("my-gpt-model".to_string()));
        let provider = AIProviderSelection::select_provider(&request).unwrap();
        // Contains "gpt-" but doesn't start with it
        // Should fall back to task type
        assert!(!provider.is_empty());
    }

    #[test]
    fn test_ai_inference_request_clone() {
        let request = create_test_request("chat", Some("gpt-4".to_string()));
        let cloned = request.clone();

        assert_eq!(cloned.task_type, request.task_type);
        assert_eq!(cloned.model, request.model);
        assert_eq!(cloned.messages.len(), request.messages.len());
    }

    #[test]
    fn test_provider_selection_all_providers_returned() {
        // Verify all three providers can be selected
        let openai_request = create_test_request("code_generation", None);
        let anthropic_request = create_test_request("analysis", None);
        let ollama_request = create_test_request("local", None);

        let openai = AIProviderSelection::select_provider(&openai_request).unwrap();
        let anthropic = AIProviderSelection::select_provider(&anthropic_request).unwrap();
        let ollama = AIProviderSelection::select_provider(&ollama_request).unwrap();

        assert_eq!(openai, "openai");
        assert_eq!(anthropic, "anthropic");
        assert_eq!(ollama, "ollama");
    }

    #[test]
    fn test_provider_selection_consistent_results() {
        let request = create_test_request("chat", Some("gpt-4".to_string()));

        // Should return consistent results
        let provider1 = AIProviderSelection::select_provider(&request).unwrap();
        let provider2 = AIProviderSelection::select_provider(&request).unwrap();

        assert_eq!(provider1, provider2);
    }
}
