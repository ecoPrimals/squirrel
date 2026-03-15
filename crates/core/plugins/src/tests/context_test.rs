// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

#[cfg(test)]
mod tests {
    use crate::plugins::context_impl::TestPluginContext;

    #[test]
    fn test_plugin_context() {
        // Create a new context
        let context = TestPluginContext::new();
        
        // Store a value - use String::from() to create a proper String
        context.set("key", String::from("value"));
        
        // Retrieve the value
        let value: Option<String> = context.get("key");
        assert_eq!(value, Some("value".to_string()));
        
        // Remove the value
        let removed = context.remove("key");
        assert!(removed);
        
        // Verify it's gone
        let value: Option<String> = context.get("key");
        assert_eq!(value, None);
    }
} 