use std::sync::{Arc, Mutex};

// Mock versions of the callback structs for testing
// AIClientCallbacks
struct AIClientCallbacks {
    mcp_service: Option<Box<dyn Fn(&str) -> Result<String, String> + Send + Sync>>,
    log_event: Option<Box<dyn Fn(&str) -> Result<(), String> + Send + Sync>>,
    track_usage: Option<Box<dyn Fn(i32, i32, i32) -> Result<(), String> + Send + Sync>>,
    check_rate_limit: Option<Box<dyn Fn() -> Result<bool, String> + Send + Sync>>,
}

impl Default for AIClientCallbacks {
    fn default() -> Self {
        Self {
            mcp_service: None,
            log_event: None,
            track_usage: None,
            check_rate_limit: None,
        }
    }
}

// ContextManagerCallbacks
struct ContextManagerCallbacks {
    mcp_service: Option<Box<dyn Fn(&str) -> Result<String, String> + Send + Sync>>,
    log_event: Option<Box<dyn Fn(&str) -> Result<(), String> + Send + Sync>>,
}

impl Default for ContextManagerCallbacks {
    fn default() -> Self {
        Self {
            mcp_service: None,
            log_event: None,
        }
    }
}

// ToolCallbacks
struct ToolCallbacks {
    add_message: Option<Box<dyn Fn(&str, &str, &str) -> Result<String, String> + Send + Sync>>,
    get_conversation: Option<Box<dyn Fn(&str) -> Result<Vec<String>, String> + Send + Sync>>,
    send_mcp_message: Option<Box<dyn Fn(&str) -> Result<String, String> + Send + Sync>>,
}

impl Default for ToolCallbacks {
    fn default() -> Self {
        Self {
            add_message: None,
            get_conversation: None,
            send_mcp_message: None,
        }
    }
}

fn main() {
    println!("Testing callback structs for thread-safety compliance...");
    
    // Test AIClientCallbacks
    test_ai_client_callbacks();
    
    // Test ContextManagerCallbacks
    test_context_manager_callbacks();
    
    // Test ToolCallbacks
    test_tool_callbacks();
    
    println!("All tests passed successfully!");
}

fn test_ai_client_callbacks() {
    println!("Testing AIClientCallbacks...");
    
    // Create callbacks with default implementation
    let callbacks = AIClientCallbacks::default();
    
    // Ensure no callbacks are set
    assert!(callbacks.mcp_service.is_none());
    assert!(callbacks.log_event.is_none());
    assert!(callbacks.track_usage.is_none());
    assert!(callbacks.check_rate_limit.is_none());
    
    // Test that we can put it in a thread-safe container
    let callbacks_container = Arc::new(Mutex::new(callbacks));
    
    // Lock the mutex, proving that we can mutate the callbacks
    let mut callbacks_mut = callbacks_container.lock().unwrap();
    
    // Add a dummy callback
    callbacks_mut.log_event = Some(Box::new(move |_msg| {
        println!("Log event called");
        Ok(())
    }));
    
    println!("AIClientCallbacks test passed!");
}

fn test_context_manager_callbacks() {
    println!("Testing ContextManagerCallbacks...");
    
    // Create callbacks with default implementation
    let callbacks = ContextManagerCallbacks::default();
    
    // Ensure no callbacks are set
    assert!(callbacks.mcp_service.is_none());
    assert!(callbacks.log_event.is_none());
    
    // Test that we can put it in a thread-safe container
    let callbacks_container = Arc::new(Mutex::new(callbacks));
    
    // Lock the mutex, proving that we can mutate the callbacks
    let mut callbacks_mut = callbacks_container.lock().unwrap();
    
    // Add a dummy callback
    callbacks_mut.log_event = Some(Box::new(move |_msg| {
        println!("Log event called");
        Ok(())
    }));
    
    println!("ContextManagerCallbacks test passed!");
}

fn test_tool_callbacks() {
    println!("Testing ToolCallbacks...");
    
    // Create callbacks with default implementation
    let callbacks = ToolCallbacks::default();
    
    // Ensure no callbacks are set
    assert!(callbacks.add_message.is_none());
    assert!(callbacks.get_conversation.is_none());
    assert!(callbacks.send_mcp_message.is_none());
    
    // Test that we can put it in a thread-safe container and move it
    let callbacks_container = Arc::new(Mutex::new(Some(callbacks)));
    
    // Lock the mutex, take the callbacks to prove ownership movement works
    let callbacks_opt = callbacks_container.lock().unwrap().take();
    assert!(callbacks_opt.is_some());
    
    println!("ToolCallbacks test passed!");
}
