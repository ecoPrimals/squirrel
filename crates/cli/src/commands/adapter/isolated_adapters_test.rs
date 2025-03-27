// Tests for the adapter module using isolated tests
// These tests are completely isolated from the rest of the codebase and
// only test the adapter pattern concepts using simplified mock implementations.

use crate::commands::adapter::completely_isolated_tests::{
    SimpleAdapter, SimpleTestCommand, McpAdapter, PluginAdapter, SimplePlugin
};
use std::sync::Arc;

#[test]
fn test_simple_adapter() {
    let mut adapter = SimpleAdapter::new();
    let command = Arc::new(SimpleTestCommand::new(
        "test", 
        "A test command", 
        "Test executed"
    ));
    
    // Register and check
    adapter.register_command(command).unwrap();
    assert_eq!(adapter.list_commands().unwrap(), vec!["test"]);
    
    // Execute without args
    let result = adapter.execute_command("test", vec![]).unwrap();
    assert_eq!(result, "Test executed");
    
    // Execute with args
    let result = adapter.execute_command("test", vec!["arg1".to_string(), "arg2".to_string()]).unwrap();
    assert_eq!(result, "Test executed: arg1 arg2");
    
    // Get help
    let help = adapter.get_help("test").unwrap();
    assert_eq!(help, "Help for test: A test command");
    
    // Error cases
    let result = adapter.execute_command("nonexistent", vec![]);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Command 'nonexistent' not found");
}

#[test]
fn test_mcp_adapter() {
    let mut adapter = McpAdapter::new();
    let command = Arc::new(SimpleTestCommand::new(
        "admin_cmd", 
        "An admin command", 
        "Admin command executed"
    ));
    
    // Register and set as admin command
    adapter.register_command(command).unwrap();
    adapter.add_admin_command("admin_cmd");
    
    // Execute with admin user
    let result = adapter.execute_command("admin_cmd", vec!["admin".to_string()]).unwrap();
    assert_eq!(result, "Admin command executed");
    
    // Execute with admin user and args
    let result = adapter.execute_command(
        "admin_cmd", 
        vec!["admin".to_string(), "arg1".to_string()]
    ).unwrap();
    assert_eq!(result, "Admin command executed: arg1");
    
    // Execute with unauthorized user
    let result = adapter.execute_command("admin_cmd", vec!["guest".to_string()]);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "User 'guest' is not authorized for admin commands");
    
    // Execute without auth
    let result = adapter.execute_command("admin_cmd", vec![]);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Authentication required for admin commands");
}

#[test]
fn test_plugin_adapter() {
    let mut adapter = PluginAdapter::new();
    let mut plugin = SimplePlugin::new("test_plugin");
    
    let command = Arc::new(SimpleTestCommand::new(
        "plugin_cmd", 
        "A plugin command", 
        "Plugin command executed"
    ));
    
    plugin.add_command(command);
    adapter.register_plugin(Box::new(plugin)).unwrap();
    
    // Execute without args
    let result = adapter.execute_command("plugin_cmd", vec![]).unwrap();
    assert_eq!(result, "Plugin command executed");
    
    // Execute with args
    let result = adapter.execute_command(
        "plugin_cmd", 
        vec!["arg1".to_string(), "arg2".to_string()]
    ).unwrap();
    assert_eq!(result, "Plugin command executed: arg1 arg2");
    
    // Get help
    let help = adapter.get_help("plugin_cmd").unwrap();
    assert_eq!(help, "Help for plugin_cmd: A plugin command");
    
    // List commands
    let commands = adapter.list_commands().unwrap();
    assert_eq!(commands, vec!["plugin_cmd"]);
    
    // Direct command registration should fail
    let command = Arc::new(SimpleTestCommand::new(
        "direct_cmd", 
        "A direct command", 
        "Direct command executed"
    ));
    let result = adapter.register_command(command);
    assert!(result.is_err());
} 