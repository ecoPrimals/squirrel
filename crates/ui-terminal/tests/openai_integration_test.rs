use std::env;
use ui_terminal::app::chat::openai::OpenAIService;
use secrecy::ExposeSecret;

/// Helper function to create a test service with API key
async fn create_test_service_with_key() -> Option<OpenAIService> {
    // Check if we have a real API key
    let api_key = std::env::var("OPENAI_API_KEY").ok()
        .or_else(|| {
            match squirrel_ai_tools::config::Config::load() {
                Ok(config) => {
                    // Use proper access to the SecretString
                    let api_key = config.openai_api_key.expose_secret();
                    Some(api_key.0.clone())
                },
                Err(_) => None,
            }
        });
    
    // If we have an API key, create a real service
    if let Some(key) = api_key {
        // Temporarily set the env var to ensure our service gets created with the real key
        env::set_var("OPENAI_API_KEY", key);
        
        // Create service with real API enabled
        let mut service = OpenAIService::new_with_model("gpt-3.5-turbo").unwrap();
        service.enable_real_api(true);
        
        Some(service)
    } else {
        println!("No API key available for testing.");
        None
    }
}

#[tokio::test]
async fn test_openai_service_creation() {
    // Test creating the service with different models
    let service = OpenAIService::new();
    assert!(service.is_ok());
    
    let service = OpenAIService::new_with_model("gpt-4");
    assert!(service.is_ok());
}

#[tokio::test]
async fn test_mock_response_generation() {
    // Test the mock response functionality
    let service = OpenAIService::new().unwrap();
    
    // Test without conversation history
    let conversation: Vec<(String, bool)> = vec![];
    let response = service.send_message("Hello, AI!", &conversation).await;
    
    assert!(response.is_ok());
    let content = response.unwrap();
    assert!(!content.is_empty());
    assert!(content.contains("Hello") || content.contains("hello"));
    
    // Test with conversation history
    let conversation = vec![
        ("Hello, AI!".to_string(), true),
        ("Hello! I'm your AI assistant. How can I help you today?".to_string(), false),
    ];
    
    let response = service.send_message("Thanks for the info", &conversation).await;
    
    assert!(response.is_ok());
    let content = response.unwrap();
    assert!(!content.is_empty());
    // Should contain contextual response since we have history
    assert!(content.contains("welcome") || content.contains("conversation"));
}

#[tokio::test]
#[ignore = "Requires real OpenAI API key and makes external API calls"]
async fn test_real_api_connectivity() {
    // This test requires a real API key from env var or config
    // We'll skip by default to avoid external API calls during testing
    
    if let Some(service) = create_test_service_with_key().await {
        // Make a simple API request
        let conversation: Vec<(String, bool)> = vec![];
        let response = service.send_message("Hello, are you connected?", &conversation).await;
        
        assert!(response.is_ok(), "Failed to get response from OpenAI API: {:?}", response.err());
        let content = response.unwrap();
        
        // The response should be non-empty
        assert!(!content.is_empty());
        println!("OpenAI API response: {}", content);
    } else {
        // Skip the test when no API key is available
        println!("Skipping real API test - no API key available");
    }
}

#[tokio::test]
#[ignore = "Requires real OpenAI API key and makes external API calls"]
async fn test_conversation_history_with_real_api() {
    // This test verifies that conversation history is correctly used with the API
    
    if let Some(service) = create_test_service_with_key().await {
        // Create a conversation with context
        let conversation = vec![
            ("My name is TestUser".to_string(), true),
            ("Hello TestUser! How can I help you today?".to_string(), false),
        ];
        
        // The AI should remember the name from the conversation history
        let response = service.send_message("What's my name?", &conversation).await;
        
        assert!(response.is_ok());
        let content = response.unwrap();
        
        // The response should contain the name from the context
        assert!(content.contains("TestUser"), "Response should contain the user's name from context");
        println!("Conversation history test response: {}", content);
    } else {
        // Skip the test when no API key is available
        println!("Skipping conversation history test - no API key available");
    }
} 