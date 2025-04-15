// crates/ui-terminal/tests/chat_message_deduplication_test.rs

use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

use ui_terminal::app::chat::{ChatApp, ChatResponse};
use crate::mocks::MockDashboardService;

mod mocks;

/// Test ChatApp's ability to handle and deduplicate messages
#[tokio::test]
async fn test_chat_message_deduplication() {
    // Create a mock service
    let mock_service = Arc::new(MockDashboardService::new());
    
    // Create a ChatApp instance
    let mut app = ChatApp::new(mock_service);
    
    // Capture the tx sender
    let tx = app.tx.clone();
    
    // Send multiple messages with the same ID to simulate duplicates
    let test_id = 12345;
    let test_content = "Test response message".to_string();
    
    // Create a duplicate message (same ID, same content)
    let duplicate_message = ChatResponse {
        content: test_content.clone(),
        is_error: false,
        id: test_id,
    };
    
    // Send the same message multiple times
    for _ in 0..3 {
        let msg = duplicate_message.clone();
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            tx_clone.send(msg).await.unwrap();
        });
    }
    
    // Give the messages time to be sent
    sleep(Duration::from_millis(50)).await;
    
    // Process received messages
    app.process_received_messages();
    
    // Check that only one message was added to the chat (deduplication worked)
    assert_eq!(app.state.messages.len(), 1, "Only one message should be added despite sending duplicates");
    
    // Verify the content of the message
    if let Some(message) = app.state.messages.first() {
        assert_eq!(message.content, test_content, "Message content should match what was sent");
        assert_eq!(message.is_user, false, "Message should be marked as from AI, not user");
    }
    
    // Verify the message is marked as processed
    assert!(app.processed_messages.contains(&test_id), "Message ID should be in the processed set");
}

/// Test that messages with different IDs but same content are treated as separate messages
#[tokio::test]
async fn test_different_id_messages() {
    // Create a mock service
    let mock_service = Arc::new(MockDashboardService::new());
    
    // Create a ChatApp instance
    let mut app = ChatApp::new(mock_service);
    
    // Capture the tx sender
    let tx = app.tx.clone();
    
    // Create message with different IDs but same content
    let content = "Same content, different IDs".to_string();
    
    let message1 = ChatResponse {
        content: content.clone(),
        is_error: false,
        id: 1001,
    };
    
    let message2 = ChatResponse {
        content: content.clone(),
        is_error: false,
        id: 1002,
    };
    
    // Send messages with different IDs
    tx.send(message1).await.unwrap();
    
    // Process first message
    sleep(Duration::from_millis(50)).await;
    app.process_received_messages();
    
    // Send second message
    tx.send(message2).await.unwrap();
    
    // Process second message
    sleep(Duration::from_millis(50)).await;
    app.process_received_messages();
    
    // Check that we have two messages (because they had different IDs)
    assert_eq!(app.state.messages.len(), 2, "Messages with different IDs should be processed as separate");
}

/// Test that temporary messages are properly replaced
#[tokio::test]
async fn test_temporary_message_replacement() {
    // Create a mock service
    let mock_service = Arc::new(MockDashboardService::new());
    
    // Create a ChatApp instance
    let mut app = ChatApp::new(mock_service);
    
    // Add a "thinking" message
    let thinking_msg = "Thinking about your message...".to_string();
    app.state.add_ai_message(thinking_msg.clone());
    app.has_temp_message = true;
    
    // Capture the tx sender
    let tx = app.tx.clone();
    
    // Create a response message that should replace the temporary one
    let response = ChatResponse {
        content: "This is the actual response".to_string(),
        is_error: false,
        id: 2001,
    };
    
    // Send the response
    tx.send(response.clone()).await.unwrap();
    
    // Give time for the message to be sent
    sleep(Duration::from_millis(50)).await;
    
    // Process the message
    app.process_received_messages();
    
    // Check that we still have only one message (the temp was replaced)
    assert_eq!(app.state.messages.len(), 1, "The temporary message should be replaced, not added to");
    
    // Check that the content is the response content
    if let Some(message) = app.state.messages.first() {
        assert_eq!(message.content, response.content, "The message content should be updated to the response");
    }
    
    // Check that we no longer have a temporary message
    assert_eq!(app.has_temp_message, false, "The temporary message flag should be cleared");
}

/// Test handling multiple simultaneous messages
#[tokio::test]
async fn test_multiple_messages_handling() {
    // Create a mock service
    let mock_service = Arc::new(MockDashboardService::new());
    
    // Create a ChatApp instance
    let mut app = ChatApp::new(mock_service);
    
    // Capture the tx sender
    let tx = app.tx.clone();
    
    // Send multiple different messages in quick succession
    for i in 1..5 {
        let msg = ChatResponse {
            content: format!("Message {}", i),
            is_error: false,
            id: i * 1000,
        };
        
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            tx_clone.send(msg).await.unwrap();
        });
    }
    
    // Give messages time to be sent
    sleep(Duration::from_millis(100)).await;
    
    // Process received messages
    app.process_received_messages();
    
    // Only the last message should be processed
    assert_eq!(app.state.messages.len(), 1, "Only the last message should be processed");
    
    // The last message should be Message 4
    if let Some(message) = app.state.messages.first() {
        assert_eq!(message.content, "Message 4", "The latest message should be processed");
    }
}

/// Test error response handling
#[tokio::test]
async fn test_error_response_handling() {
    // Create a mock service
    let mock_service = Arc::new(MockDashboardService::new());
    
    // Create a ChatApp instance
    let mut app = ChatApp::new(mock_service);
    
    // Capture the tx sender
    let tx = app.tx.clone();
    
    // Add a "thinking" message
    let thinking_msg = "Thinking about your message...".to_string();
    app.state.add_ai_message(thinking_msg.clone());
    app.has_temp_message = true;
    
    // Create an error response
    let error_msg = "API connection failed";
    let response = ChatResponse {
        content: format!("Error: {}", error_msg),
        is_error: true,
        id: 3001,
    };
    
    // Send the error response
    tx.send(response).await.unwrap();
    
    // Process the message
    sleep(Duration::from_millis(50)).await;
    app.process_received_messages();
    
    // Check that the temporary message was replaced with the error
    assert_eq!(app.state.messages.len(), 1, "The error should replace the temp message");
    
    // Verify the content contains the error
    if let Some(message) = app.state.messages.first() {
        assert!(message.content.contains("Error:"), "The message should be formatted as an error");
        assert!(message.content.contains(error_msg), "The error message should contain the actual error text");
    }
} 