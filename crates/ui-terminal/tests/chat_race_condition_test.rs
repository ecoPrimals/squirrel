use std::sync::Arc;
use std::time::Duration;
use tokio::test;
use tokio::time::sleep;
use std::sync::Mutex;
use ui_terminal::app::chat::{ChatApp, ChatResponse, OpenAIService, OpenAIError};
use ui_terminal::error::Error;
use ui_terminal::widgets::chat::ChatState;
use crate::mocks::MockDashboardService;
use async_trait::async_trait;
use std::env;

mod mocks;

/// Mock OpenAI service for testing
#[derive(Debug)]
struct MockOpenAIService {
    responses: Vec<String>,
    response_delay: Duration,
    split_responses: bool,
    use_real_api: bool,
}

impl MockOpenAIService {
    fn new() -> Self {
        Self {
            responses: vec![
                "First part of the response.".to_string(),
                "Second part of the response.".to_string(),
                "Third part of the response.".to_string(),
            ],
            response_delay: Duration::from_millis(10),
            split_responses: false,
            use_real_api: true,
        }
    }
    
    /// Configure to send split responses (simulates OpenAI streaming where chunks might arrive separately)
    fn with_split_responses(mut self) -> Self {
        self.split_responses = true;
        self
    }
    
    /// Configure the delay between responses
    fn with_delay(mut self, delay: Duration) -> Self {
        self.response_delay = delay;
        self
    }
    
    /// Send a message to the OpenAI API
    async fn send_message(&self, _message: &str, _history: &[(String, bool)]) -> Result<String, String> {
        // Simulate processing delay
        sleep(self.response_delay).await;
        
        if self.split_responses {
            // Combine all parts into one response (this is the expected behavior)
            Ok(self.responses.join(" "))
        } else {
            // Return just the first response (mimicking partial response)
            Ok(self.responses[0].clone())
        }
    }
    
    fn is_real_api_enabled(&self) -> bool {
        self.use_real_api
    }
    
    fn enable_real_api(&mut self, enable: bool) {
        self.use_real_api = enable;
    }
}

/// Test the race condition that occurs with the OpenAI service by using
/// our mock service directly through the ChatApp
#[tokio::test]
async fn test_openai_service_race_condition() {
    // Setup a mock service
    let mock_service = Arc::new(MockDashboardService::new());
    
    // Create a ChatApp instance
    let mut app = ChatApp::new(mock_service);
    
    // Create our race condition service with the OpenAIService trait methods
    let race_service = MockOpenAIService::new().with_split_responses();
    
    // Manually construct a response to simulate what would happen with the OpenAI service
    let response_content = race_service.responses.join(" ");
    
    // Add a user message to the chat first
    app.state.add_user_message("Test message".to_string());
    
    // Add a thinking message that will be replaced
    app.state.add_ai_message("Thinking...".to_string());
    app.has_temp_message = true;
    
    // Capture the tx sender
    let tx = app.tx.clone();
    
    // Send the response
    let response = ChatResponse {
        content: response_content,
        is_error: false,
        id: generate_unique_id(),
    };
    tx.send(response).await.unwrap();
    
    // Wait for the async task to complete
    sleep(Duration::from_millis(100)).await;
    
    // Process received messages
    app.process_received_messages();
    
    // Check that we have exactly two messages: the user message and the AI response
    assert_eq!(app.state.messages.len(), 2, "Should have user message and AI response");
    
    // Verify the messages
    if app.state.messages.len() >= 2 {
        // First message should be from the user
        assert_eq!(app.state.messages[0].is_user, true, "First message should be from user");
        assert_eq!(app.state.messages[0].content, "Test message", "User message content should match");
        
        // Second message should be from AI with the combined response
        assert_eq!(app.state.messages[1].is_user, false, "Second message should be from AI");
        assert_eq!(
            app.state.messages[1].content, 
            "First part of the response. Second part of the response. Third part of the response.", 
            "AI response should contain all parts"
        );
    }
}

/// Test to simulate rapid multiple responses that should be deduplicated
#[tokio::test]
async fn test_rapid_response_deduplication() {
    // Setup a mock service
    let mock_service = Arc::new(MockDashboardService::new());
    
    // Create a ChatApp instance
    let mut app = ChatApp::new(mock_service);
    
    // Capture the tx sender
    let tx = app.tx.clone();
    
    // Add a user message
    app.state.add_user_message("User question".to_string());
    
    // Add a thinking message
    app.state.add_ai_message("Thinking...".to_string());
    app.has_temp_message = true;
    
    // Simulate multiple partial responses arriving quickly from OpenAI
    // Each with the exact same text but coming in separate chunks
    let response_parts = [
        "This is a complete",
        "This is a complete response",
        "This is a complete response from OpenAI."
    ];
    
    // Send the responses in rapid succession
    for (i, part) in response_parts.iter().enumerate() {
        let msg = ChatResponse {
            content: part.to_string(),
            is_error: false,
            // Use different IDs to simulate the real-world issue
            id: generate_unique_id(),
        };
        
        // Small delay to simulate network timing
        sleep(Duration::from_millis(5)).await;
        
        let tx_clone = tx.clone();
        tx_clone.send(msg).await.unwrap();
        
        // Process after each message to simulate how the UI updates
        if i < response_parts.len() - 1 {
            sleep(Duration::from_millis(5)).await;
            app.process_received_messages();
        }
    }
    
    // Final processing
    sleep(Duration::from_millis(10)).await;
    app.process_received_messages();
    
    // Check message count - should be user message + final AI response
    assert_eq!(app.state.messages.len(), 2, "Should only have user message and AI response");
    
    // Verify the last message is the complete response
    if app.state.messages.len() >= 2 {
        assert_eq!(app.state.messages[1].content, "This is a complete response from OpenAI.", 
            "Final message should be the complete response");
    }
}

/// Simulates the issue observed in logs where we get multiple chunks of the same message
#[tokio::test]
async fn test_fragmented_response_handling() {
    // Setup a mock service
    let mock_service = Arc::new(MockDashboardService::new());
    
    // Create a ChatApp instance
    let mut app = ChatApp::new(mock_service);
    
    // Capture the tx sender
    let tx = app.tx.clone();
    
    // Add a user message
    app.state.add_user_message("Write me a poem".to_string());
    
    // Add thinking message
    app.state.add_ai_message("Thinking about your message: \"Write me a poem\"...".to_string());
    app.has_temp_message = true;
    
    // The poetic response that will come in fragments
    let poem = "Of course, here is a short poem for you:\n\nIn a world of chaos and noise,\nFind solace in simple joys.\nNature's beauty, love so true,\nPeace within, always anew.\n\nI hope you enjoyed it!";
    
    // Simulate the fragmented response pattern seen in the logs
    // These are the actual fragments observed in the manually added selection
    let fragments = [
        "Of course, here is a short poem for you:\n\n",
        "Of course, here is a short poem for you:\n\nIn a world of chaos and noise,\n",
        "Of course, here is a short poem for you:\n\nIn a world of chaos and noise,\nFind solace in simple joys.\n",
        "Of course, here is a short poem for you:\n\nIn a world of chaos and noise,\nFind solace in simple joys.\nNature's beauty, love so true,\n",
        "Of course, here is a short poem for you:\n\nIn a world of chaos and noise,\nFind solace in simple joys.\nNature's beauty, love so true,\nPeace within, always anew.\n\n",
        poem,  // Complete poem
    ];
    
    // Send the fragmented responses with different IDs
    for fragment in fragments {
        let msg = ChatResponse {
            content: fragment.to_string(),
            is_error: false,
            id: generate_unique_id(),
        };
        
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            tx_clone.send(msg).await.unwrap();
        });
        
        // Small delay between fragments
        sleep(Duration::from_millis(5)).await;
    }
    
    // Wait for messages to be processed
    sleep(Duration::from_millis(50)).await;
    
    // Process all messages
    app.process_received_messages();
    
    // Check that we have exactly two messages: user question and the complete AI response
    assert_eq!(app.state.messages.len(), 2, "Should have user message and final AI response");
    
    // The final message should be the complete poem
    if app.state.messages.len() >= 2 {
        assert_eq!(app.state.messages[1].content, poem, "Final message should be the complete poem");
    }
    
    // Ensure the temporary message flag is cleared
    assert_eq!(app.has_temp_message, false, "Temporary message flag should be cleared");
}

// Helper to generate unique IDs for the tests
fn generate_unique_id() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    use rand::Rng;
    
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    let random_component = rand::thread_rng().gen::<u16>() as u64;
    now ^ (random_component << 48)
} 