// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Plugin Directory Loading Tests
//!
//! Comprehensive tests for plugin loading from directory functionality

#[cfg(test)]
mod plugin_loading_tests {
    use squirrel_plugins::{DefaultPluginManager, PluginManagerTrait};
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// Helper to create a test plugin manifest (TOML)
    fn create_test_manifest_toml(dir: &PathBuf, name: &str, version: &str) -> std::io::Result<()> {
        let manifest = format!(
            r#"
[plugin]
id = "{}"
name = "{}"
version = "{}"
description = "Test plugin"
author = "Test Author"
homepage = "https://example.com"
repository = "https://github.com/test/test"

[plugin.capabilities]
security = false
monitoring = false
storage = false

[[plugin.dependencies]]
"#,
            uuid::Uuid::new_v4(),
            name,
            version
        );

        fs::write(dir.join("plugin.toml"), manifest)?;
        Ok(())
    }

    /// Helper to create a test plugin manifest (JSON)
    fn create_test_manifest_json(dir: &PathBuf, name: &str, version: &str) -> std::io::Result<()> {
        let manifest = serde_json::json!({
            "id": uuid::Uuid::new_v4().to_string(),
            "name": name,
            "version": version,
            "description": "Test plugin",
            "author": "Test Author",
            "homepage": "https://example.com",
            "repository": "https://github.com/test/test",
            "capabilities": {
                "security": false,
                "monitoring": false,
                "storage": false
            },
            "dependencies": []
        });

        fs::write(
            dir.join("plugin.json"),
            serde_json::to_string_pretty(&manifest)?,
        )?;
        Ok(())
    }

    #[tokio::test]
    async fn test_load_plugins_from_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let manager = DefaultPluginManager::new();

        let result = manager
            .load_plugins(temp_dir.path().to_str().unwrap())
            .await;

        assert!(result.is_ok());
        let plugin_ids = result.unwrap();
        assert_eq!(plugin_ids.len(), 0);
    }

    #[tokio::test]
    async fn test_load_plugins_nonexistent_directory() {
        let manager = DefaultPluginManager::new();

        let result = manager.load_plugins("/nonexistent/directory/path").await;

        // Should return Ok with empty vec (graceful handling)
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_load_single_plugin_toml() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_dir = temp_dir.path().join("test-plugin");
        fs::create_dir(&plugin_dir).unwrap();

        create_test_manifest_toml(&plugin_dir, "test-plugin", "1.0.0").unwrap();

        let manager = DefaultPluginManager::new();
        let result = manager
            .load_plugins(temp_dir.path().to_str().unwrap())
            .await;

        assert!(result.is_ok());
        let plugin_ids = result.unwrap();
        assert_eq!(plugin_ids.len(), 1);
    }

    #[tokio::test]
    async fn test_load_single_plugin_json() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_dir = temp_dir.path().join("test-plugin-json");
        fs::create_dir(&plugin_dir).unwrap();

        create_test_manifest_json(&plugin_dir, "test-plugin-json", "2.0.0").unwrap();

        let manager = DefaultPluginManager::new();
        let result = manager
            .load_plugins(temp_dir.path().to_str().unwrap())
            .await;

        assert!(result.is_ok());
        let plugin_ids = result.unwrap();
        assert_eq!(plugin_ids.len(), 1);
    }

    #[tokio::test]
    async fn test_load_multiple_plugins() {
        let temp_dir = TempDir::new().unwrap();

        // Create multiple plugin directories
        for i in 1..=5 {
            let plugin_dir = temp_dir.path().join(format!("plugin-{}", i));
            fs::create_dir(&plugin_dir).unwrap();

            if i % 2 == 0 {
                create_test_manifest_toml(&plugin_dir, &format!("plugin-{}", i), "1.0.0").unwrap();
            } else {
                create_test_manifest_json(&plugin_dir, &format!("plugin-{}", i), "1.0.0").unwrap();
            }
        }

        let manager = DefaultPluginManager::new();
        let result = manager
            .load_plugins(temp_dir.path().to_str().unwrap())
            .await;

        assert!(result.is_ok());
        let plugin_ids = result.unwrap();
        assert_eq!(plugin_ids.len(), 5);
    }

    #[tokio::test]
    async fn test_load_plugins_with_invalid_manifest() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_dir = temp_dir.path().join("invalid-plugin");
        fs::create_dir(&plugin_dir).unwrap();

        // Create invalid TOML
        fs::write(plugin_dir.join("plugin.toml"), "invalid toml content {{{").unwrap();

        let manager = DefaultPluginManager::new();
        let result = manager
            .load_plugins(temp_dir.path().to_str().unwrap())
            .await;

        // Should succeed but skip invalid plugins (logged as warnings)
        assert!(result.is_ok());
        let plugin_ids = result.unwrap();
        assert_eq!(plugin_ids.len(), 0);
    }

    #[tokio::test]
    async fn test_load_plugins_mixed_valid_invalid() {
        let temp_dir = TempDir::new().unwrap();

        // Valid plugin 1
        let plugin_dir_1 = temp_dir.path().join("valid-plugin-1");
        fs::create_dir(&plugin_dir_1).unwrap();
        create_test_manifest_toml(&plugin_dir_1, "valid-1", "1.0.0").unwrap();

        // Invalid plugin
        let plugin_dir_2 = temp_dir.path().join("invalid-plugin");
        fs::create_dir(&plugin_dir_2).unwrap();
        fs::write(plugin_dir_2.join("plugin.toml"), "bad content").unwrap();

        // Valid plugin 2
        let plugin_dir_3 = temp_dir.path().join("valid-plugin-2");
        fs::create_dir(&plugin_dir_3).unwrap();
        create_test_manifest_json(&plugin_dir_3, "valid-2", "2.0.0").unwrap();

        let manager = DefaultPluginManager::new();
        let result = manager
            .load_plugins(temp_dir.path().to_str().unwrap())
            .await;

        assert!(result.is_ok());
        let plugin_ids = result.unwrap();
        // Should load 2 valid plugins, skip 1 invalid
        assert_eq!(plugin_ids.len(), 2);
    }

    #[tokio::test]
    async fn test_load_plugins_no_manifest() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_dir = temp_dir.path().join("no-manifest-plugin");
        fs::create_dir(&plugin_dir).unwrap();

        // Create directory but no manifest
        fs::write(plugin_dir.join("readme.txt"), "No manifest here").unwrap();

        let manager = DefaultPluginManager::new();
        let result = manager
            .load_plugins(temp_dir.path().to_str().unwrap())
            .await;

        assert!(result.is_ok());
        let plugin_ids = result.unwrap();
        // Should skip directories without manifests
        assert_eq!(plugin_ids.len(), 0);
    }

    #[tokio::test]
    async fn test_load_plugins_directory_is_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("not-a-directory.txt");
        fs::write(&file_path, "This is a file").unwrap();

        let manager = DefaultPluginManager::new();
        let result = manager.load_plugins(file_path.to_str().unwrap()).await;

        // Should return error since it's not a directory
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_load_plugins_verify_registration() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_dir = temp_dir.path().join("registered-plugin");
        fs::create_dir(&plugin_dir).unwrap();
        create_test_manifest_toml(&plugin_dir, "registered-plugin", "1.0.0").unwrap();

        let manager = DefaultPluginManager::new();
        let result = manager
            .load_plugins(temp_dir.path().to_str().unwrap())
            .await;

        assert!(result.is_ok());
        let plugin_ids = result.unwrap();
        assert_eq!(plugin_ids.len(), 1);

        // Verify plugin is actually registered
        let plugin_id = plugin_ids[0];
        let status = PluginManagerTrait::get_plugin_status(&manager, plugin_id).await;
        assert!(status.is_ok());
    }

    #[tokio::test]
    async fn test_load_plugins_nested_directories() {
        let temp_dir = TempDir::new().unwrap();

        // Create nested structure
        let plugin_dir = temp_dir.path().join("parent").join("child");
        fs::create_dir_all(&plugin_dir).unwrap();
        create_test_manifest_toml(&plugin_dir, "nested-plugin", "1.0.0").unwrap();

        let manager = DefaultPluginManager::new();

        // Load from parent - should not recurse into subdirectories
        let result = manager
            .load_plugins(temp_dir.path().join("parent").to_str().unwrap())
            .await;

        assert!(result.is_ok());
        let plugin_ids = result.unwrap();
        // Should find the plugin in child directory
        assert_eq!(plugin_ids.len(), 1);
    }

    #[tokio::test]
    async fn test_load_plugins_special_characters_in_name() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_dir = temp_dir.path().join("special-plugin");
        fs::create_dir(&plugin_dir).unwrap();

        // Plugin with special characters
        create_test_manifest_toml(&plugin_dir, "plugin-with-special_chars.v2", "1.0.0").unwrap();

        let manager = DefaultPluginManager::new();
        let result = manager
            .load_plugins(temp_dir.path().to_str().unwrap())
            .await;

        assert!(result.is_ok());
        let plugin_ids = result.unwrap();
        assert_eq!(plugin_ids.len(), 1);
    }
}
