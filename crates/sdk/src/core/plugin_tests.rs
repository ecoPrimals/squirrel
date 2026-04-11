// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use super::super::manager::{PluginManager, utils};
use super::*;
use crate::config::PluginConfig;
use crate::mcp::McpCapabilities;

#[test]
fn test_plugin_creation() {
    let plugin = BasePlugin::new("test".to_string(), "1.0.0".to_string());
    let info = WasmPlugin::get_info(&plugin);
    assert_eq!(info.name, "test");
    assert_eq!(info.version, "1.0.0");
}

#[tokio::test]
#[cfg(target_arch = "wasm32")]
async fn test_plugin_lifecycle() {
    let mut plugin = BasePlugin::new("test".to_string(), "1.0.0".to_string());
    let config = PluginConfig::default();
    let js_config = serde_wasm_bindgen::to_value(&config).expect("config to wasm");

    // Test initialization
    assert!(plugin.initialize(js_config).await.is_ok());
    let info = WasmPlugin::get_info(&plugin);
    assert_eq!(info.name, "test");

    // Test state
    let info = WasmPlugin::get_info(&plugin);
    assert_eq!(info.name, "test");
}

#[test]
fn test_plugin_validation() {
    let plugin = BasePlugin::new("test".to_string(), "1.0.0".to_string());
    let info = WasmPlugin::get_info(&plugin);
    assert_eq!(info.name, "test");
}

#[test]
fn test_plugin_stats_default() {
    let stats = PluginStats::default();
    assert_eq!(stats.commands_executed, 0);
    assert_eq!(stats.error_count, 0);
    assert!(!stats.start_time.is_empty());
    assert!(!stats.last_activity.is_empty());
}

#[test]
fn test_plugin_capabilities_all() {
    let caps = PluginCapabilities::all();
    assert_eq!(caps.permissions.len(), 2);
    // Check we have LocalStorage and SessionStorage (Permission doesn't impl PartialEq)
    let has_local = caps
        .permissions
        .iter()
        .any(|p| matches!(p, Permission::LocalStorage));
    let has_session = caps
        .permissions
        .iter()
        .any(|p| matches!(p, Permission::SessionStorage));
    assert!(has_local);
    assert!(has_session);
}

#[test]
fn test_plugin_capabilities_none() {
    let caps = PluginCapabilities::none();
    assert!(caps.commands.is_empty());
    assert!(caps.events.is_empty());
    assert!(caps.permissions.is_empty());
}

#[test]
fn test_plugin_context_new() {
    let ctx = PluginContext::new("plugin-123".to_string());
    assert_eq!(ctx.plugin_id, "plugin-123");
    assert_eq!(ctx.working_directory, "/");
    assert!(ctx.environment.is_empty());
}

#[test]
fn test_plugin_context_get_set_env() {
    let mut ctx = PluginContext::new("test".to_string());
    ctx.set_env("KEY".to_string(), "value".to_string());
    assert_eq!(ctx.get_env("KEY"), Some(&"value".to_string()));
    assert_eq!(ctx.get_env("MISSING"), None);
}

#[test]
fn test_plugin_command_result_success() {
    let result = PluginCommandResult::success("data".to_string());
    assert!(result.success);
    assert_eq!(result.data, "data");
    assert!(result.error.is_none());
}

#[test]
fn test_plugin_command_result_error() {
    let result = PluginCommandResult::error("failed".to_string());
    assert!(!result.success);
    assert_eq!(result.error, Some("failed".to_string()));
}

#[test]
fn test_plugin_command_result_with_metadata() {
    let result = PluginCommandResult::success("data".to_string())
        .with_metadata(r#"{"key":"value"}"#.to_string());
    assert_eq!(result.metadata, r#"{"key":"value"}"#);
}

#[test]
fn test_plugin_status_equality() {
    assert_eq!(PluginStatus::Uninitialized, PluginStatus::Uninitialized);
    assert_eq!(PluginStatus::Active, PluginStatus::Active);
    assert_ne!(PluginStatus::Uninitialized, PluginStatus::Active);
    assert_eq!(
        PluginStatus::Error("msg".to_string()),
        PluginStatus::Error("msg".to_string())
    );
}

#[test]
fn test_command_info_creation() {
    let cmd = CommandInfo {
        name: "test_cmd".to_string(),
        description: "A test".to_string(),
        category: Some("utils".to_string()),
        parameters: Some(serde_json::json!({"type": "string"})),
    };
    assert_eq!(cmd.name, "test_cmd");
    assert_eq!(cmd.description, "A test");
    assert_eq!(cmd.category, Some("utils".to_string()));
}

#[test]
fn test_plugin_context_get_config() {
    let mut ctx = PluginContext::new("test".to_string());
    ctx.config = serde_json::json!({"key": "value"});
    assert_eq!(ctx.get_config("key"), Some(&serde_json::json!("value")));
    assert_eq!(ctx.get_config("missing"), None);
}

#[test]
fn test_plugin_manager_lifecycle() {
    let manager = PluginManager::new();
    assert!(manager.list_plugins().expect("list_plugins").is_empty());
    assert!(!manager.has_plugin("nonexistent").expect("has_plugin"));
}

#[test]
fn test_plugin_manager_has_plugin_empty() {
    let manager = PluginManager::new();
    assert!(!manager.has_plugin("x").expect("has_plugin"));
}

#[test]
fn test_validate_plugin_info_valid() {
    let info = PluginInfo {
        id: "test-id".to_string(),
        name: "Test".to_string(),
        version: "1.0.0".to_string(),
        state: PluginStatus::Uninitialized,
        config: PluginConfig::default(),
        stats: PluginStats::default(),
        capabilities: vec![],
        description: String::new(),
        author: String::new(),
        license: "MIT".to_string(),
        repository: None,
        keywords: vec![],
        metadata: serde_json::json!({}),
    };
    assert!(utils::validate_plugin_info(&info).is_ok());
}

#[test]
fn test_validate_plugin_info_empty_id() {
    let mut info = PluginInfo {
        id: String::new(),
        name: "Test".to_string(),
        version: "1.0.0".to_string(),
        state: PluginStatus::Uninitialized,
        config: PluginConfig::default(),
        stats: PluginStats::default(),
        capabilities: vec![],
        description: String::new(),
        author: String::new(),
        license: "MIT".to_string(),
        repository: None,
        keywords: vec![],
        metadata: serde_json::json!({}),
    };
    assert!(utils::validate_plugin_info(&info).is_err());
    info.id = "x".to_string();
    info.name = String::new();
    assert!(utils::validate_plugin_info(&info).is_err());
    info.name = "Test".to_string();
    info.version = String::new();
    assert!(utils::validate_plugin_info(&info).is_err());
    info.version = "invalid".to_string();
    assert!(utils::validate_plugin_info(&info).is_err());
}

#[test]
fn test_plugin_utils_create_default_context() {
    let ctx = utils::create_default_context("my-plugin".to_string());
    assert_eq!(ctx.plugin_id, "my-plugin");
}

#[test]
fn runtime_context_stores_mcp_capabilities() {
    let ctx = RuntimeContext {
        plugin_id: "pid".to_string(),
        config: PluginConfig::default(),
        capabilities: PluginCapabilities::none(),
        mcp_capabilities: McpCapabilities::default(),
    };
    assert_eq!(ctx.plugin_id, "pid");
}

#[test]
fn wasm_plugin_trait_stats_capabilities_and_events_no_js() {
    let p = BasePlugin::new("stats".to_string(), "1.0.0".to_string());
    let s = WasmPlugin::get_stats(&p);
    assert_eq!(s.commands_executed, 0);
    let caps = WasmPlugin::get_capabilities(&p);
    assert!(caps.commands.is_empty());
    assert!(WasmPlugin::handle_event(&p, wasm_bindgen::JsValue::NULL).is_ok());
}
