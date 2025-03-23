use serde_json::{json, Value};
use crate::context::{
    ContextPlugin, 
    ContextTransformation, 
    create_context_plugin, 
    create_custom_context_plugin
};
use crate::plugin::Plugin;

#[tokio::test]
async fn test_default_context_plugin() {
    // Create default plugin
    let plugin = create_context_plugin();
    
    // Check metadata
    assert_eq!(plugin.metadata().name, "Context Plugin");
    assert!(plugin.metadata().capabilities.contains(&"context.transform".to_string()));
    assert!(plugin.metadata().capabilities.contains(&"context.validate".to_string()));
    
    // Initialize plugin
    assert!(plugin.initialize().await.is_ok());
    
    // Get transformations
    let transformations = plugin.get_transformations();
    assert!(!transformations.is_empty());
    assert_eq!(transformations[0].id, "context.standard");
    
    // Test transformation
    let data = json!({
        "data": { "key": "value" }
    });
    
    let result = plugin.transform("context.standard", data.clone()).await;
    assert!(result.is_ok());
    
    let result_value = result.unwrap();
    if let Value::Object(obj) = &result_value {
        assert!(obj.contains_key("result"));
        assert!(obj.contains_key("metadata"));
    } else {
        panic!("Expected object result, got: {:?}", result_value);
    }
    
    // Test validation
    let schema = json!({
        "data": { "required": true }
    });
    
    let validation = plugin.validate(&schema, &data);
    assert!(validation.is_ok());
    assert!(validation.unwrap());
    
    // Shutdown plugin
    assert!(plugin.shutdown().await.is_ok());
}

#[tokio::test]
async fn test_custom_context_plugin() {
    // Create custom transformations
    let transformations = vec![
        ContextTransformation {
            id: "custom.transform".to_string(),
            name: "Custom Transformation".to_string(),
            description: "Custom transformation for testing".to_string(),
            input_schema: json!({ "type": "object" }),
            output_schema: json!({ "type": "object" }),
        }
    ];
    
    // Create custom plugin
    let plugin = create_custom_context_plugin(
        "Custom Context Plugin",
        "Custom plugin for testing",
        transformations
    );
    
    // Check metadata
    assert_eq!(plugin.metadata().name, "Custom Context Plugin");
    assert!(plugin.metadata().capabilities.contains(&"context.transform".to_string()));
    
    // Initialize plugin
    assert!(plugin.initialize().await.is_ok());
    
    // Get transformations
    let plugin_transformations = plugin.get_transformations();
    assert!(!plugin_transformations.is_empty());
    assert_eq!(plugin_transformations[0].id, "custom.transform");
    
    // Test transformation
    let data = json!({
        "data": { "key": "value" }
    });
    
    let result = plugin.transform("custom.transform", data.clone()).await;
    assert!(result.is_ok());
    
    // Test transformation support check
    assert!(plugin.supports_transformation("custom.transform"));
    assert!(!plugin.supports_transformation("nonexistent.transform"));
    
    // Shutdown plugin
    assert!(plugin.shutdown().await.is_ok());
} 