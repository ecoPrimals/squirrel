#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use crate::plugins::plugin::{PluginItem, PluginMetadata, PluginStatus};

    #[test]
    fn test_plugin_creation() {
        let metadata = PluginMetadata {
            name: "test-plugin".to_string(),
            version: "0.1.0".to_string(),
            description: Some("A test plugin".to_string()),
            author: Some("Test Author".to_string()),
            homepage: None,
        };

        let plugin = PluginItem::new(
            metadata.clone(),
            PathBuf::from("/path/to/plugin"),
            PluginStatus::Installed,
        );

        assert_eq!(plugin.metadata().name, "test-plugin");
        assert_eq!(plugin.metadata().version, "0.1.0");
        assert_eq!(plugin.metadata().description, Some("A test plugin".to_string()));
        assert_eq!(plugin.metadata().author, Some("Test Author".to_string()));
        assert_eq!(plugin.metadata().homepage, None);
        assert_eq!(plugin.path(), &PathBuf::from("/path/to/plugin"));
        assert_eq!(plugin.status(), &PluginStatus::Installed);
    }

    #[test]
    fn test_plugin_status_transitions() {
        let metadata = PluginMetadata {
            name: "status-test".to_string(),
            version: "0.1.0".to_string(),
            description: None,
            author: None,
            homepage: None,
        };

        let mut plugin = PluginItem::new(
            metadata,
            PathBuf::from("/path/to/status-test"),
            PluginStatus::Installed,
        );

        // Test status transitions
        assert_eq!(plugin.status(), &PluginStatus::Installed);
        
        plugin.set_status(PluginStatus::Enabled);
        assert_eq!(plugin.status(), &PluginStatus::Enabled);
        
        plugin.set_status(PluginStatus::Disabled);
        assert_eq!(plugin.status(), &PluginStatus::Disabled);
        
        plugin.set_status(PluginStatus::Failed("Test error".to_string()));
        match plugin.status() {
            PluginStatus::Failed(err) => assert_eq!(err, "Test error"),
            _ => panic!("Expected Failed status"),
        }
    }
} 