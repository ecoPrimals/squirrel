// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

use super::*;

#[test]
fn test_zero_copy_metadata() {
    let metadata = ZeroCopyPluginMetadata::new(
        Uuid::new_v4(),
        "test-plugin".to_string(),
        "1.0.0".to_string(),
        "A test plugin".to_string(),
        "Test Author".to_string(),
    );

    assert_eq!(metadata.name(), "test-plugin");
    assert_eq!(metadata.version(), "1.0.0");
    assert_eq!(metadata.description(), "A test plugin");
    assert_eq!(metadata.author(), "Test Author");
}

#[test]
fn test_metadata_builder() {
    let metadata = PluginMetadataBuilder::new()
        .name("test-plugin".to_string())
        .version("1.0.0".to_string())
        .description("A test plugin".to_string())
        .author("Test Author".to_string())
        .capability("text-processing".to_string())
        .dependency("core".to_string())
        .tag("utility".to_string())
        .custom_metadata("priority".to_string(), "high".to_string())
        .build();

    assert_eq!(metadata.name(), "test-plugin");
    assert!(metadata.has_capability("text-processing"));
    assert!(metadata.has_dependency("core"));
    assert_eq!(metadata.get_custom_metadata("priority"), Some("high"));
}

#[tokio::test]
async fn test_plugin_registration_and_retrieval() {
    let registry = Arc::new(ZeroCopyPluginRegistry::new());
    let metadata = PluginMetadataBuilder::new()
        .name("test-plugin".to_string())
        .version("1.0.0".to_string())
        .description("Test plugin".to_string())
        .author("Test".to_string())
        .capability("text-processing".to_string())
        .build();
    let entry = ZeroCopyPluginEntry::new(
        metadata.clone(),
        ZeroCopyPluginConfig::new(metadata.id),
        None,
    );

    registry
        .register_plugin(entry)
        .await
        .expect("Plugin registration should succeed in test environment");

    let retrieved = registry
        .get_plugin(metadata.id)
        .await
        .expect("Plugin retrieval by ID should succeed after successful registration");

    assert_eq!(retrieved.metadata.id, metadata.id);

    let retrieved_by_name = registry
        .get_plugin_by_name("test-plugin")
        .await
        .expect("Plugin retrieval by name should succeed after successful registration");

    assert_eq!(retrieved_by_name.metadata.name.as_ref(), "test-plugin");
}

#[test]
fn test_plugin_event() {
    let event = PluginEvent::new_borrowed("test-event", "test-data");
    assert_eq!(event.event_type(), "test-event");
    assert_eq!(event.data(), "test-data");

    let event_owned = PluginEvent::new_owned("owned-event".to_string(), "owned-data".to_string());
    assert_eq!(event_owned.event_type(), "owned-event");
    assert_eq!(event_owned.data(), "owned-data");
}

#[test]
fn from_arc_params_and_metadata_accessors() {
    let id = Uuid::new_v4();
    let params = FromArcParams {
        id,
        name: Arc::from("n"),
        version: Arc::from("v"),
        description: Arc::from("d"),
        author: Arc::from("a"),
        dependencies: Arc::new(vec!["d1".to_string()]),
        capabilities: Arc::new(vec!["c1".to_string()]),
        tags: Arc::new(vec!["t1".to_string()]),
        custom_metadata: Arc::new(std::iter::once(("k".to_string(), "v".to_string())).collect()),
    };
    let m = ZeroCopyPluginMetadata::from_arc(params);
    assert!(m.has_capability("c1"));
    assert!(m.has_dependency("d1"));
    assert_eq!(m.get_custom_metadata("k"), Some("v"));
}

#[tokio::test]
async fn registry_capability_index_and_miss_stats() {
    let reg = ZeroCopyPluginRegistry::new();
    let meta = PluginMetadataBuilder::new()
        .name("indexed".to_string())
        .capability("unique-cap".to_string())
        .build();
    let entry = ZeroCopyPluginEntry::new(
        meta.clone(),
        ZeroCopyPluginConfig::new(meta.id),
        Some(std::path::PathBuf::from("/tmp/plugin")),
    );
    reg.register_plugin(entry).await.expect("register");
    let by_cap = reg.find_plugins_by_capability("unique-cap").await;
    assert_eq!(by_cap.len(), 1);
    assert_eq!(by_cap[0].path(), Some(std::path::Path::new("/tmp/plugin")));
    let _ = reg.get_plugin(Uuid::new_v4()).await;
    let s = reg.stats().await;
    assert!(s.registry_misses >= 1);
}

#[test]
fn zero_copy_plugin_state_apply_and_builder() {
    let mut st = ZeroCopyPluginState::new(Uuid::new_v4(), crate::types::PluginStatus::Registered);
    st.apply_status(
        crate::types::PluginStatus::Running,
        Some("start".to_string()),
    );
    assert_eq!(st.status, crate::types::PluginStatus::Running);
    let st2 = ZeroCopyPluginState::new(Uuid::new_v4(), crate::types::PluginStatus::Registered)
        .with_status(crate::types::PluginStatus::Stopped, None);
    assert_eq!(st2.status, crate::types::PluginStatus::Stopped);
}

#[test]
fn zero_copy_plugin_config_getters() {
    use std::collections::HashMap;
    let mut cfg = ZeroCopyPluginConfig::new(Uuid::new_v4());
    let mut data = HashMap::new();
    data.insert("key".to_string(), serde_json::json!({"x": 1}));
    cfg.config_data = Arc::new(data);
    let mut env = HashMap::new();
    env.insert("VAR".to_string(), "val".to_string());
    cfg.environment = Arc::new(env);
    assert!(cfg.get_config("key").is_some());
    assert_eq!(cfg.get_env("VAR"), Some("val"));
}

#[test]
fn zero_copy_plugin_metadata_macro_expands() {
    let m = crate::zero_copy_plugin_metadata!("plug", "0.1.0", "desc", "auth");
    assert_eq!(m.name(), "plug");
    assert_eq!(m.version(), "0.1.0");
}
