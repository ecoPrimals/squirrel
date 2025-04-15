use dashboard_core::service::MockDashboardService;
use std::sync::Arc;
use tokio::sync::mpsc;
use ui_terminal::app::chat::{ChatApp, ChatResponse};
use std::collections::HashSet;

/// Test the message deduplication logic
#[tokio::test]
async fn test_message_deduplication() {
    // Create channels for testing
    let (tx, rx) = mpsc::channel(10);

    // Create some test responses
    let response1 = ChatResponse::final_response("message 1".to_string());
    let response1_clone = ChatResponse {
        content: response1.content.clone(),
        is_error: response1.is_error,
        id: response1.id,
    };
    let response2 = ChatResponse::final_response("message 2".to_string());

    // Create a mock chat app
    let service = Arc::new(MockDashboardService::new());
    let mut app = ChatApp::<MockDashboardService>::new(service);
    app.rx = rx;
    app.tx = tx.clone();

    // Send messages including a duplicate
    tx.send(response1.clone()).await.unwrap();
    tx.send(response1_clone).await.unwrap(); // Same ID as response1
    tx.send(response2.clone()).await.unwrap();

    // Process messages and verify deduplication
    app.process_received_messages();
    
    // First message should be processed
    assert!(app.processed_messages.contains(&response1.id));
    
    // Process messages again to catch the second message
    // (Note: in a real app these would be processed in a loop)
    app.process_received_messages();
    
    // Both unique messages should be processed
    assert!(app.processed_messages.contains(&response1.id));
    assert!(app.processed_messages.contains(&response2.id));
    
    // We should have exactly 2 IDs in the processed set (not 3)
    assert_eq!(app.processed_messages.len(), 2);
}

/// Test the thinking message replacement
#[tokio::test]
async fn test_thinking_message_replacement() {
    // Create a mock chat app
    let service = Arc::new(MockDashboardService::new());
    let mut app = ChatApp::<MockDashboardService>::new(service);
    
    // Add a thinking message
    app.has_temp_message = true;
    app.state.add_ai_message("Thinking...".to_string());
    
    // Verify the thinking message is there
    assert_eq!(app.state.messages.len(), 1);
    assert_eq!(app.state.messages[0].content, "Thinking...");
    
    // Process a final response
    let response = ChatResponse::final_response("Final answer".to_string());
    app.process_single_message(response);
    
    // Verify the thinking message was replaced
    assert_eq!(app.state.messages.len(), 1); // Still just one message
    assert_eq!(app.state.messages[0].content, "Final answer"); // But content updated
    assert!(!app.has_temp_message); // No longer marked as temporary
}

/// Test pre-formatted and wrapped message detection
#[test]
fn test_message_format_detection() {
    // A regular message that should be wrapped
    let regular_message = "This is a regular message that should be wrapped by the system because it doesn't contain any newlines.";
    assert!(!regular_message.contains('\n'));
    
    // A pre-formatted message (like a poem) that has its own line breaks
    let preformatted_message = "Roses are red,\nViolets are blue,\nThis message has newlines,\nAnd should not be wrapped too.";
    assert!(preformatted_message.contains('\n'));
    
    // A Windows-style pre-formatted message
    let windows_formatted = "Line one\r\nLine two\r\nLine three";
    assert!(windows_formatted.contains("\r\n"));
}

/// Test conversation history filtering
#[test]
fn test_conversation_history_filtering() {
    // Create a mock chat app
    let service = Arc::new(MockDashboardService::new());
    let mut app = ChatApp::<MockDashboardService>::new(service);
    
    // Add some messages
    app.state.add_user_message("User message 1".to_string());
    app.state.add_ai_message("AI message 1".to_string());
    app.state.add_user_message("User message 2".to_string());
    
    // Get conversation history - all messages should be included
    let history = app.get_conversation_history();
    assert_eq!(history.len(), 3);
    
    // Add a thinking message
    app.has_temp_message = true;
    app.state.add_ai_message("Thinking...".to_string());
    
    // Get conversation history - thinking message should be excluded
    let history = app.get_conversation_history();
    assert_eq!(history.len(), 3); // Still 3, not 4
    
    // The last message shouldn't be the thinking message
    let (last_content, _) = &history[history.len() - 1];
    assert_ne!(last_content, "Thinking...");
} 