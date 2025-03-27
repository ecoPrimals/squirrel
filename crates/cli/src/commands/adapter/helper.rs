use std::sync::Arc;
use tokio::sync::Mutex;
use log::{debug, info};

use commands::CommandRegistry;
use crate::commands::register_commands;
use crate::commands::adapter::error::{AdapterError, AdapterResult};
use crate::commands::adapter::registry::CommandRegistryAdapter;
use crate::commands::adapter::mcp::{AuthManager, BasicAuthProvider, McpCommandAdapter};
use crate::commands::adapter::plugins::CommandsPluginAdapter;
use crate::commands::adapter::UserRole;
use crate::commands::adapter::CommandAdapterTrait;

/// Helper result type for adapter operations
pub type AdapterHelperResult<T> = Result<T, AdapterError>;

/// Create a new registry adapter with all commands registered
pub fn create_initialized_registry_adapter() -> AdapterHelperResult<Arc<CommandRegistryAdapter>> {
    debug!("Creating initialized registry adapter");
    
    // Create command registry
    let mut registry = CommandRegistry::new();
    
    // Register built-in commands
    register_commands(&mut registry);
    
    // Create Arc-wrapped registry
    let registry_arc = Arc::new(Mutex::new(registry));
    
    // Create adapter
    let adapter = Arc::new(CommandRegistryAdapter::new(registry_arc));
    
    info!("Created initialized registry adapter");
    Ok(adapter)
}

/// Create a new registry adapter with no commands registered
pub fn create_empty_registry_adapter() -> AdapterHelperResult<Arc<CommandRegistryAdapter>> {
    debug!("Creating empty registry adapter");
    
    // Create command registry
    let registry = CommandRegistry::new();
    
    // Create Arc-wrapped registry
    let registry_arc = Arc::new(Mutex::new(registry));
    
    // Create adapter
    let adapter = Arc::new(CommandRegistryAdapter::new(registry_arc));
    
    info!("Created empty registry adapter");
    Ok(adapter)
}

/// Create a new MCP adapter with basic auth
pub fn create_mcp_adapter() -> AdapterResult<Arc<McpCommandAdapter>> {
    let registry_adapter = create_empty_registry_adapter().map_err(|e| {
        AdapterError::Internal(format!("Failed to create registry adapter: {}", e))
    })?;
    
    // Create auth provider
    let mut auth_provider = BasicAuthProvider::new();
    auth_provider
        .add_user("admin", "admin", UserRole::Admin)
        .add_user("user", "password", UserRole::RegularUser)
        .add_command_permission("admin", vec![UserRole::Admin]);
    
    let auth_provider = Arc::new(auth_provider);
    let auth_manager = Arc::new(AuthManager::with_provider(auth_provider));
    
    let adapter = Arc::new(McpCommandAdapter::new(auth_manager, registry_adapter));
    Ok(adapter)
}

/// Create a new plugin adapter
pub fn create_plugin_adapter(registry_adapter: Arc<CommandRegistryAdapter>) -> AdapterHelperResult<Arc<CommandsPluginAdapter>> {
    debug!("Creating plugin adapter");
    
    // Get registry from adapter
    let registry = registry_adapter.get_registry();
    
    // Create plugin adapter
    let adapter = Arc::new(CommandsPluginAdapter::new(registry));
    
    info!("Created plugin adapter");
    Ok(adapter)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_initialized_registry_adapter() {
        // Create registry adapter
        let adapter = create_initialized_registry_adapter().unwrap();
        
        // Check if registry has commands
        let commands = adapter.list_commands().await.unwrap();
        assert!(!commands.is_empty());
        
        // Check for some expected commands
        assert!(commands.contains(&"help".to_string()));
        assert!(commands.contains(&"version".to_string()));
    }
    
    #[tokio::test]
    async fn test_create_empty_registry_adapter() {
        // Create registry adapter
        let adapter = create_empty_registry_adapter().unwrap();
        
        // Check if registry has no commands
        let commands = adapter.list_commands().await.unwrap();
        assert!(commands.is_empty());
    }
    
    #[tokio::test]
    async fn test_create_mcp_adapter() {
        // Create MCP adapter
        let mcp_adapter = create_mcp_adapter().unwrap();
        
        // Just check that we got a valid Arc back
        assert!(Arc::strong_count(&mcp_adapter) >= 1);
    }
    
    #[tokio::test]
    async fn test_create_plugin_adapter() {
        // Create registry adapter
        let registry_adapter = create_initialized_registry_adapter().unwrap();
        
        // Create plugin adapter
        let plugin_adapter = create_plugin_adapter(registry_adapter).unwrap();
        
        // Just check that we got a valid Arc back
        assert!(Arc::strong_count(&plugin_adapter) >= 1);
    }
} 