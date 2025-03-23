use serde_json::{json, Value};
use crate::context_adapter::{
    ContextAdapterPlugin, 
    AdapterMetadata, 
    create_context_adapter_plugin, 
    create_custom_context_adapter_plugin
};
use crate::plugin::Plugin;

#[tokio::test]
async fn test_default_context_adapter_plugin() {
    // Create default plugin
    let plugin = create_context_adapter_plugin();
    
    // Check metadata
    assert_eq!(plugin.metadata().name, "Context Adapter Plugin");
    assert!(plugin.metadata().capabilities.contains(&"context.adapter".to_string()));
    assert!(plugin.metadata().capabilities.contains(&"context.format".to_string()));
    
    // Initialize plugin
    assert!(plugin.initialize().await.is_ok());
    
    // Get adapters
    let adapters = plugin.get_adapters();
    assert!(!adapters.is_empty());
    
    // Check for JSON to MCP adapter
    let json_to_mcp = adapters.iter().find(|a| a.id == "json.to.mcp");
    assert!(json_to_mcp.is_some());
    assert_eq!(json_to_mcp.unwrap().source_format, "json");
    assert_eq!(json_to_mcp.unwrap().target_format, "mcp");
    
    // Test conversion
    let data = json!({
        "command": "process",
        "data": { "value": 42 }
    });
    
    let result = plugin.convert("json.to.mcp", data.clone()).await;
    assert!(result.is_ok());
    
    let result_value = result.unwrap();
    if let Value::Object(obj) = &result_value {
        assert!(obj.contains_key("converted_data"));
        assert!(obj.contains_key("metadata"));
    } else {
        panic!("Expected object result, got: {:?}", result_value);
    }
    
    // Test format validation
    assert!(plugin.validate_format("json", &data).unwrap());
    
    // Test compatibility check
    assert!(plugin.check_compatibility("json", "mcp"));
    assert!(plugin.check_compatibility("mcp", "json"));
    assert!(!plugin.check_compatibility("json", "unknown"));
    
    // Shutdown plugin
    assert!(plugin.shutdown().await.is_ok());
}

#[tokio::test]
async fn test_custom_context_adapter_plugin() {
    // Create custom adapters
    let adapters = vec![
        AdapterMetadata {
            id: "custom.adapter".to_string(),
            name: "Custom Adapter".to_string(),
            description: "Custom adapter for testing".to_string(),
            source_format: "custom".to_string(),
            target_format: "standard".to_string(),
        }
    ];
    
    // Create custom plugin
    let plugin = create_custom_context_adapter_plugin(
        "Custom Context Adapter Plugin",
        "Custom plugin for testing",
        adapters
    );
    
    // Check metadata
    assert_eq!(plugin.metadata().name, "Custom Context Adapter Plugin");
    assert!(plugin.metadata().capabilities.contains(&"context.adapter".to_string()));
    
    // Initialize plugin
    assert!(plugin.initialize().await.is_ok());
    
    // Get adapters
    let plugin_adapters = plugin.get_adapters();
    assert!(!plugin_adapters.is_empty());
    assert_eq!(plugin_adapters[0].id, "custom.adapter");
    
    // Test adapter support check
    assert!(plugin.supports_adapter("custom.adapter"));
    assert!(!plugin.supports_adapter("nonexistent.adapter"));
    
    // Shutdown plugin
    assert!(plugin.shutdown().await.is_ok());
} 