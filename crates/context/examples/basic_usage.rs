use std::sync::Arc;
use squirrel_context::{
    create_manager, create_adapter,
    ContextState, ContextError, Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Create a context manager
    let manager = create_manager();
    
    // Create a context adapter
    let adapter = create_adapter(manager.clone());
    
    // Initialize the adapter - this will also activate the default context
    adapter.initialize().await?;
    
    // Get the current context ID
    let current_id = adapter.get_current_context_id()?;
    println!("Current context ID: {}", current_id);
    
    // Get the current context tracker
    let tracker = adapter.get_current_tracker()?;
    
    // Get the current state
    let mut state = tracker.get_state()?;
    println!("Initial state: {:?}", state);
    
    // Update the state
    state.set("key1".to_string(), "value1".to_string());
    state.set("key2".to_string(), "value2".to_string());
    
    // Update the tracker with the new state
    tracker.update_state(state)?;
    
    // Get the updated state
    let updated_state = tracker.get_state()?;
    println!("Updated state: {:?}", updated_state);
    
    // Create a new context
    let new_context_id = "new-context";
    let new_tracker = adapter.create_and_activate_context(new_context_id).await?;
    
    // Verify the current context ID
    let current_id = adapter.get_current_context_id()?;
    println!("New current context ID: {}", current_id);
    
    // Update the new context state
    let mut new_state = new_tracker.get_state()?;
    new_state.set("app".to_string(), "example".to_string());
    new_state.set("version".to_string(), "1.0".to_string());
    
    // Update the tracker with the new state
    new_tracker.update_state(new_state)?;
    
    // Get the updated state
    let updated_state = new_tracker.get_state()?;
    println!("New context state: {:?}", updated_state);
    
    // Create a recovery point for the current context
    let snapshot = adapter.create_recovery_point().await?;
    println!("Created recovery point: {:?}", snapshot);
    
    // Switch back to the default context
    let default_tracker = adapter.switch_context("default").await?;
    
    // Verify the current context ID
    let current_id = adapter.get_current_context_id()?;
    println!("Switched back to context ID: {}", current_id);
    
    // Get the state of the default context
    let default_state = default_tracker.get_state()?;
    println!("Default context state: {:?}", default_state);
    
    // List all context IDs
    let context_ids = adapter.list_context_ids()?;
    println!("All context IDs: {:?}", context_ids);
    
    // List active context IDs
    let active_ids = adapter.list_active_context_ids()?;
    println!("Active context IDs: {:?}", active_ids);
    
    // Deactivate the new context
    adapter.deactivate_context(new_context_id).await?;
    
    // Verify active contexts after deactivation
    let active_ids = adapter.list_active_context_ids()?;
    println!("Active context IDs after deactivation: {:?}", active_ids);
    
    Ok(())
} 