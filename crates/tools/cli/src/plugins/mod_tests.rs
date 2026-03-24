// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Tests for the plugin system module functions.

use std::fs;
use std::path::Path;

use super::parse_plugin_metadata;
use crate::plugins::manager::PluginManager;

#[test]
fn test_parse_plugin_metadata_valid() {
    let content = r#"
name = "test-plugin"
version = "1.0.0"
description = "A test plugin"
author = "Test Author"
homepage = "https://example.com"
"#;
    let result = parse_plugin_metadata(content, Path::new("/tmp/plugins/test"));
    assert!(result.is_ok());
    let metadata = result.expect("should succeed");
    assert_eq!(metadata.name, "test-plugin");
    assert_eq!(metadata.version, "1.0.0");
    assert_eq!(metadata.description, Some("A test plugin".to_string()));
    assert_eq!(metadata.author, Some("Test Author".to_string()));
    assert_eq!(metadata.homepage, Some("https://example.com".to_string()));
}

#[test]
fn test_parse_plugin_metadata_minimal() {
    let content = r#"
name = "minimal"
version = "0.1.0"
"#;
    let result = parse_plugin_metadata(content, Path::new("/tmp/plugins/min"));
    assert!(result.is_ok());
    let metadata = result.expect("should succeed");
    assert_eq!(metadata.name, "minimal");
    assert_eq!(metadata.version, "0.1.0");
    assert!(metadata.description.is_none());
    assert!(metadata.author.is_none());
    assert!(metadata.homepage.is_none());
}

#[test]
fn test_parse_plugin_metadata_missing_name() {
    let content = r#"
version = "1.0.0"
description = "No name"
"#;
    let result = parse_plugin_metadata(content, Path::new("/tmp/plugins/noname"));
    assert!(result.is_err());
}

#[test]
fn test_parse_plugin_metadata_missing_version() {
    let content = r#"
name = "no-version"
description = "Missing version"
"#;
    let result = parse_plugin_metadata(content, Path::new("/tmp/plugins/nover"));
    assert!(result.is_err());
}

#[test]
fn test_parse_plugin_metadata_empty_content() {
    let result = parse_plugin_metadata("", Path::new("/tmp/plugins/empty"));
    assert!(result.is_err());
}

#[test]
fn test_parse_plugin_metadata_comments_and_empty_lines() {
    let content = r#"
# This is a comment
name = "commented-plugin"

# Another comment
version = "2.0.0"

# description is optional
"#;
    let result = parse_plugin_metadata(content, Path::new("/tmp/plugins/commented"));
    assert!(result.is_ok());
    let metadata = result.expect("should succeed");
    assert_eq!(metadata.name, "commented-plugin");
    assert_eq!(metadata.version, "2.0.0");
}

#[test]
fn test_parse_plugin_metadata_unknown_keys_ignored() {
    let content = r#"
name = "ext-plugin"
version = "1.0.0"
license = "MIT"
unknown_key = "ignored"
"#;
    let result = parse_plugin_metadata(content, Path::new("/tmp/plugins/ext"));
    assert!(result.is_ok());
    let metadata = result.expect("should succeed");
    assert_eq!(metadata.name, "ext-plugin");
}

#[test]
fn test_get_plugin_directories_returns_vec() {
    let dirs = super::get_plugin_directories();
    // Should at least include the HOME-based directory if HOME is set
    // The function always returns a vec, even if empty - verify it doesn't panic
    let _ = dirs.len();
}

#[test]
fn discover_plugins_in_directory_loads_valid_plugin_toml() {
    let dir = tempfile::tempdir().expect("tempdir");
    let plugin_home = dir.path().join("demo_plugin");
    fs::create_dir_all(&plugin_home).expect("mkdir");
    fs::write(
        plugin_home.join("plugin.toml"),
        r#"name = "demo_plugin"
version = "0.2.0"
description = "Demo"
"#,
    )
    .expect("write meta");

    let mut mgr = PluginManager::new();
    let n = super::discover_plugins_in_directory(dir.path(), &mut mgr).expect("discover");
    assert_eq!(n, 1);
    assert!(mgr.get_plugin("demo_plugin").is_ok());
}

#[test]
fn discover_plugins_skips_missing_dir() {
    let mut mgr = PluginManager::new();
    let n = super::discover_plugins_in_directory(Path::new("/nonexistent/path/xyz"), &mut mgr)
        .expect("ok");
    assert_eq!(n, 0);
}

#[test]
fn discover_plugins_skips_files_without_plugin_toml() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sub = dir.path().join("empty_dir");
    fs::create_dir_all(&sub).expect("mkdir");
    let mut mgr = PluginManager::new();
    let n = super::discover_plugins_in_directory(dir.path(), &mut mgr).expect("discover");
    assert_eq!(n, 0);
}

#[test]
fn register_builtin_plugins_registers_example_factory() {
    let mut mgr = PluginManager::new();
    let n = super::register_builtin_plugins(&mut mgr).expect("builtin");
    assert!(n >= 1);
    assert!(!mgr.list_plugins().is_empty());
}
