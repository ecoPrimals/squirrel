#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use crate::plugins::manager::PluginManager;
    use crate::plugins::plugin::{PluginMetadata, PluginStatus};
    use crate::plugins::error::PluginError;

    fn create_test_plugin(name: &str, version: &str) -> (PluginMetadata, PathBuf, PluginStatus) {
        let metadata = PluginMetadata {
            name: name.to_string(),
            version: version.to_string(),
            description: Some(format!("Test plugin {}", name)),
            author: Some("Test Author".to_string()),
            homepage: None,
        };

        let path = PathBuf::from(format!("/path/to/{}", name));
        let status = PluginStatus::Installed;

        (metadata, path, status)
    }

    #[test]
    fn test_plugin_manager_add_get() {
        let mut manager = PluginManager::new();
        
        // Add test plugins
        let (metadata1, path1, status1) = create_test_plugin("plugin1", "0.1.0");
        let (metadata2, path2, status2) = create_test_plugin("plugin2", "0.2.0");
        
        // Test adding plugins
        assert!(manager.add_plugin(metadata1.clone(), path1, status1).is_ok());
        assert!(manager.add_plugin(metadata2.clone(), path2, status2).is_ok());
        
        // Test getting plugins
        let retrieved_plugin1 = manager.get_plugin("plugin1").unwrap();
        let retrieved_plugin2 = manager.get_plugin("plugin2").unwrap();
        
        assert_eq!(retrieved_plugin1.metadata().name, "plugin1");
        assert_eq!(retrieved_plugin1.metadata().version, "0.1.0");
        assert_eq!(retrieved_plugin2.metadata().name, "plugin2");
        assert_eq!(retrieved_plugin2.metadata().version, "0.2.0");
        
        // Test getting nonexistent plugin
        assert!(manager.get_plugin("nonexistent").is_err());
    }

    #[test]
    fn test_plugin_manager_list() {
        let mut manager = PluginManager::new();
        
        // Initially, list should be empty
        assert_eq!(manager.list_plugins().len(), 0);
        
        // Add test plugins
        let (metadata1, path1, status1) = create_test_plugin("list-plugin1", "0.1.0");
        let (metadata2, path2, status2) = create_test_plugin("list-plugin2", "0.2.0");
        
        assert!(manager.add_plugin(metadata1, path1, status1).is_ok());
        assert!(manager.add_plugin(metadata2, path2, status2).is_ok());
        
        // Now list should have two plugins
        let plugins = manager.list_plugins();
        assert_eq!(plugins.len(), 2);
        
        // Verify plugin data is correct
        let names: Vec<String> = plugins.iter()
            .map(|p| p.metadata().name.clone())
            .collect();
        
        assert!(names.contains(&"list-plugin1".to_string()));
        assert!(names.contains(&"list-plugin2".to_string()));
    }

    #[test]
    fn test_plugin_manager_remove() {
        let mut manager = PluginManager::new();
        
        // Add a plugin
        let (metadata, path, status) = create_test_plugin("remove-plugin", "0.1.0");
        assert!(manager.add_plugin(metadata, path, status).is_ok());
        
        // Verify it exists
        assert!(manager.get_plugin("remove-plugin").is_ok());
        
        // Remove it
        assert!(manager.remove_plugin("remove-plugin").is_ok());
        
        // Verify it doesn't exist anymore
        assert!(manager.get_plugin("remove-plugin").is_err());
        
        // Try removing it again - should fail
        match manager.remove_plugin("remove-plugin") {
            Err(PluginError::NotFound(_)) => {}, // Expected error
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_plugin_duplicates() {
        let mut manager = PluginManager::new();
        
        // Add original plugin
        let (metadata, path, status) = create_test_plugin("duplicate", "0.1.0");
        assert!(manager.add_plugin(metadata.clone(), path.clone(), status.clone()).is_ok());
        
        // Try adding plugin with same name - should fail
        let duplicate_metadata = PluginMetadata {
            name: "duplicate".to_string(),
            version: "0.2.0".to_string(),
            description: Some("Duplicate plugin".to_string()),
            author: Some("Test Author".to_string()),
            homepage: None,
        };
        
        match manager.add_plugin(duplicate_metadata, path, status) {
            Err(PluginError::AlreadyExists(_)) => {}, // Expected error
            _ => panic!("Expected AlreadyExists error"),
        }
    }
} 