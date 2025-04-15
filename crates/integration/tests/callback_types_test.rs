use std::sync::{Arc, Mutex};

use squirrel_integration::{
    ai_agent::types::AIClientCallbacks,
    context_mcp::types::ContextManagerCallbacks,
    mcp_ai_tools::adapter::ToolCallbacks,
};

#[test]
fn test_callback_default_impls() {
    // Create callbacks with default implementations
    let ai_client_callbacks = AIClientCallbacks::default();
    let context_manager_callbacks = ContextManagerCallbacks::default();
    let tool_callbacks = ToolCallbacks::default();
    
    // Ensure we can create these without any Clone issues
    assert!(ai_client_callbacks.mcp_service.is_none());
    assert!(ai_client_callbacks.log_event.is_none());
    assert!(ai_client_callbacks.track_usage.is_none());
    assert!(ai_client_callbacks.check_rate_limit.is_none());
    
    assert!(context_manager_callbacks.mcp_service.is_none());
    assert!(context_manager_callbacks.log_event.is_none());
    
    assert!(tool_callbacks.add_message.is_none());
    assert!(tool_callbacks.get_conversation.is_none());
    assert!(tool_callbacks.send_mcp_message.is_none());
}

#[test]
fn test_tool_callbacks_with_option_fields() {
    // We should be able to create callbacks without cloning issues
    let add_message = Box::new(|_: &str, _: &str, _| Ok(String::new())) as Box<dyn Fn(&str, &str, _) -> _ + Send + Sync>;
    let get_conversation = Box::new(|_: &str| Ok(Vec::new())) as Box<dyn Fn(&str) -> _ + Send + Sync>;
    let send_mcp_message = Box::new(|_: &str| Ok(String::new())) as Box<dyn Fn(&str) -> _ + Send + Sync>;
    
    let callbacks = ToolCallbacks {
        add_message: Some(add_message),
        get_conversation: Some(get_conversation),
        send_mcp_message: Some(send_mcp_message),
    };
    
    // We can move callbacks into a container without cloning
    let callbacks_container = Arc::new(Mutex::new(Some(callbacks)));
    let callbacks_clone = callbacks_container.lock().unwrap().take();
    
    assert!(callbacks_clone.is_some());
} 