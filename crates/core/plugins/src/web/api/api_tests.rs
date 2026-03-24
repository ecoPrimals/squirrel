// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use std::collections::HashMap;
use std::sync::Arc;

use super::*;
use crate::DefaultPluginManager;
use crate::discovery::create_noop_plugin;
use crate::plugin::PluginMetadata;
use crate::registry::PluginRegistry;
use crate::types::PluginStatus as RegistryPluginStatus;
use crate::web::ExampleWebPlugin;
use crate::web::{HttpMethod, HttpStatus, WebRequest};
use serde_json::json;
use uuid::Uuid;

fn web_req(method: HttpMethod, path: &str, body: Option<serde_json::Value>) -> WebRequest {
    WebRequest {
        method,
        path: path.to_string(),
        query_params: HashMap::new(),
        headers: HashMap::new(),
        body,
        user_id: None,
        permissions: vec![],
        route_params: HashMap::new(),
    }
}

#[test]
fn api_dtos_serde_roundtrip() {
    let info = PluginInfo {
        id: Uuid::new_v4(),
        name: "n".to_string(),
        version: "1".to_string(),
        description: "d".to_string(),
        author: "a".to_string(),
        status: RegistryPluginStatus::Registered,
        capabilities: vec!["c".to_string()],
        dependencies: vec!["dep".to_string()],
        endpoints: vec![EndpointInfo {
            path: "/p".to_string(),
            method: "GET".to_string(),
            description: "x".to_string(),
            permissions: vec![],
        }],
    };
    let j = serde_json::to_string(&info).expect("should succeed");
    let back: PluginInfo = serde_json::from_str(&j).expect("should succeed");
    assert_eq!(back.name, info.name);

    let install = PluginInstallRequest {
        source: "s".to_string(),
        version: Some("1.0".to_string()),
        configuration: None,
    };
    let j = serde_json::to_string(&install).expect("should succeed");
    let _: PluginInstallRequest = serde_json::from_str(&j).expect("should succeed");

    let cfg = PluginConfigurationRequest {
        configuration: std::iter::once(("k".to_string(), json!(1))).collect(),
    };
    let j = serde_json::to_string(&cfg).expect("should succeed");
    let _: PluginConfigurationRequest = serde_json::from_str(&j).expect("should succeed");

    let exec = PluginExecutionRequest {
        command: "cmd".to_string(),
        parameters: std::iter::once(("p".to_string(), json!("v"))).collect(),
    };
    let j = serde_json::to_string(&exec).expect("should succeed");
    let _: PluginExecutionRequest = serde_json::from_str(&j).expect("should succeed");

    let search = PluginSearchRequest {
        query: Some("q".to_string()),
        category: None,
        author: None,
        capabilities: None,
        limit: Some(5),
        offset: Some(0),
    };
    let j = serde_json::to_string(&search).expect("should succeed");
    let _: PluginSearchRequest = serde_json::from_str(&j).expect("should succeed");

    let mpe = PluginMarketplaceEntry {
        id: Uuid::new_v4(),
        name: "n".to_string(),
        version: "1".to_string(),
        description: "d".to_string(),
        author: "a".to_string(),
        category: "c".to_string(),
        capabilities: vec![],
        download_url: "u".to_string(),
        documentation_url: None,
        rating: Some(4.0),
        downloads: 1,
        verified: true,
    };
    let j = serde_json::to_string(&mpe).expect("should succeed");
    let _: PluginMarketplaceEntry = serde_json::from_str(&j).expect("should succeed");

    let ws = WebSocketMessage {
        event_type: "e".to_string(),
        plugin_id: Some(Uuid::new_v4()),
        data: json!({}),
        timestamp: chrono::Utc::now(),
    };
    let j = serde_json::to_string(&ws).expect("should succeed");
    let _: WebSocketMessage = serde_json::from_str(&j).expect("should succeed");
}

#[test]
fn websocket_connection_debug_clone() {
    let c = WebSocketConnection {
        id: Uuid::new_v4(),
        metadata: std::iter::once(("a".to_string(), "b".to_string())).collect(),
        subscriptions: vec!["s".to_string()],
    };
    let _ = format!("{c:?}");
    let d = c.clone();
    assert_eq!(d.id, c.id);
}

#[tokio::test]
async fn test_plugin_management_api_creation() {
    let manager = Arc::new(DefaultPluginManager::new());
    let api = PluginManagementAPI::new(manager);

    let endpoints = api.get_endpoints();
    assert!(!endpoints.is_empty());
    assert!(endpoints.iter().any(|ep| ep.path == "/api/plugins"));
    assert!(
        endpoints
            .iter()
            .any(|ep| ep.is_public && ep.path.contains("marketplace"))
    );
}

#[tokio::test]
async fn test_plugin_id_extraction_ok_and_errors() {
    let manager = Arc::new(DefaultPluginManager::new());
    let api = PluginManagementAPI::new(manager);

    let plugin_id = Uuid::new_v4();
    let path = format!("/api/plugins/{plugin_id}");
    assert_eq!(
        api.extract_plugin_id(&path).expect("should succeed"),
        plugin_id
    );

    assert!(api.extract_plugin_id("/api/plugins").is_err());
    assert!(api.extract_plugin_id("/api/plugins/not-a-uuid").is_err());
}

#[tokio::test]
async fn extract_search_params_default() {
    let manager = Arc::new(DefaultPluginManager::new());
    let api = PluginManagementAPI::new(manager);
    let r = web_req(HttpMethod::Get, "/api/marketplace/plugins", None);
    let s = api.extract_search_params(&r).expect("should succeed");
    assert_eq!(s.limit, Some(10));
    assert_eq!(s.offset, Some(0));
}

#[tokio::test]
async fn handle_list_get_plugins_empty_and_with_example_web() {
    let manager = Arc::new(DefaultPluginManager::new());
    let api = PluginManagementAPI::new(manager.clone());

    let res = api
        .handle_request(web_req(HttpMethod::Get, "/api/plugins", None))
        .await
        .expect("should succeed");
    assert_eq!(res.status, HttpStatus::Ok);
    assert_eq!(res.body.as_ref().expect("should succeed")["total"], 0);

    let ex = Arc::new(ExampleWebPlugin::new()) as Arc<dyn crate::Plugin>;
    let ex_id = ex.id();
    manager.register_plugin(ex).await.expect("should succeed");

    let res = api
        .handle_request(web_req(HttpMethod::Get, "/api/plugins", None))
        .await
        .expect("should succeed");
    assert_eq!(res.body.as_ref().expect("should succeed")["total"], 1);
    let plugins = res.body.as_ref().expect("should succeed")["plugins"]
        .as_array()
        .expect("should succeed");
    let first = &plugins[0];
    assert_eq!(
        first["id"].as_str().expect("should succeed"),
        ex_id.to_string()
    );
    let ep_list = first["endpoints"].as_array().expect("should succeed");
    assert!(!ep_list.is_empty());
}

#[tokio::test]
async fn handle_get_plugin_details_and_config() {
    let manager = Arc::new(DefaultPluginManager::new());
    let meta = PluginMetadata::new("t", "1.0.0", "d", "a");
    let id = meta.id;
    let plugin = create_noop_plugin(meta);
    manager
        .register_plugin(plugin)
        .await
        .expect("should succeed");

    let api = PluginManagementAPI::new(manager);

    let res = api
        .handle_request(web_req(
            HttpMethod::Get,
            &format!("/api/plugins/{id}"),
            None,
        ))
        .await
        .expect("should succeed");
    assert_eq!(res.status, HttpStatus::Ok);
    assert_eq!(res.body.as_ref().expect("should succeed")["name"], "t");

    let cfg = api
        .handle_request(web_req(
            HttpMethod::Get,
            &format!("/api/plugins/{id}/config"),
            None,
        ))
        .await
        .expect("should succeed");
    assert_eq!(cfg.status, HttpStatus::Ok);
    assert!(
        cfg.body
            .as_ref()
            .expect("should succeed")
            .get("configuration")
            .is_some()
    );
}

#[tokio::test]
async fn handle_install_post_accepted() {
    let manager = Arc::new(DefaultPluginManager::new());
    let api = PluginManagementAPI::new(manager);
    let res = api
        .handle_request(web_req(
            HttpMethod::Post,
            "/api/plugins",
            Some(json!({
                "source": "https://example.com/p.zip",
                "version": "1.2.3"
            })),
        ))
        .await
        .expect("should succeed");
    assert_eq!(res.status, HttpStatus::Accepted);
    assert_eq!(
        res.body.as_ref().expect("should succeed")["status"],
        "installing"
    );
}

#[tokio::test]
async fn handle_marketplace_search_and_categories() {
    let manager = Arc::new(DefaultPluginManager::new());
    let api = PluginManagementAPI::new(manager);

    let res = api
        .handle_request(web_req(HttpMethod::Get, "/api/marketplace/plugins", None))
        .await
        .expect("should succeed");
    assert_eq!(res.status, HttpStatus::Ok);
    assert!(res.body.as_ref().expect("should succeed")["plugins"].is_array());

    let cat = api
        .handle_request(web_req(
            HttpMethod::Get,
            "/api/marketplace/categories",
            None,
        ))
        .await
        .expect("should succeed");
    assert_eq!(cat.status, HttpStatus::Ok);
    assert!(cat.body.as_ref().expect("should succeed")["categories"].is_array());
}

#[tokio::test]
async fn handle_health_and_metrics() {
    let manager = Arc::new(DefaultPluginManager::new());
    let api = PluginManagementAPI::new(manager);

    let h = api
        .handle_request(web_req(HttpMethod::Get, "/api/plugins/health", None))
        .await
        .expect("should succeed");
    assert_eq!(h.status, HttpStatus::Ok);
    assert!(
        h.body
            .as_ref()
            .expect("should succeed")
            .get("healthy_plugins")
            .is_some()
    );

    let m = api
        .handle_request(web_req(HttpMethod::Get, "/api/plugins/metrics", None))
        .await
        .expect("should succeed");
    assert_eq!(m.status, HttpStatus::Ok);
    assert!(
        m.body
            .as_ref()
            .expect("should succeed")
            .get("api_uptime_seconds")
            .is_some()
    );
}

#[tokio::test]
async fn handle_not_found() {
    let manager = Arc::new(DefaultPluginManager::new());
    let api = PluginManagementAPI::new(manager);
    let res = api
        .handle_request(web_req(HttpMethod::Get, "/api/unknown", None))
        .await
        .expect("should succeed");
    assert_eq!(res.status, HttpStatus::NotFound);
}

#[tokio::test]
async fn handle_uninstall_start_stop_restart_execute_config() {
    let manager = Arc::new(DefaultPluginManager::new());
    let meta = PluginMetadata::new("life", "1.0.0", "d", "a");
    let id = meta.id;
    let plugin = create_noop_plugin(meta);
    manager
        .register_plugin(plugin)
        .await
        .expect("should succeed");

    let api = PluginManagementAPI::new(manager.clone());

    let un = api
        .handle_request(web_req(
            HttpMethod::Delete,
            &format!("/api/plugins/{id}"),
            None,
        ))
        .await
        .expect("should succeed");
    assert_eq!(un.status, HttpStatus::Ok);

    let meta2 = PluginMetadata::new("life2", "1.0.0", "d", "a");
    let id2 = meta2.id;
    manager
        .register_plugin(create_noop_plugin(meta2))
        .await
        .expect("should succeed");

    let st = api
        .handle_request(web_req(
            HttpMethod::Post,
            &format!("/api/plugins/{id2}/start"),
            None,
        ))
        .await
        .expect("should succeed");
    assert_eq!(st.status, HttpStatus::Ok);

    let sp = api
        .handle_request(web_req(
            HttpMethod::Post,
            &format!("/api/plugins/{id2}/stop"),
            None,
        ))
        .await
        .expect("should succeed");
    assert_eq!(sp.status, HttpStatus::Ok);

    let rs = api
        .handle_request(web_req(
            HttpMethod::Post,
            &format!("/api/plugins/{id2}/restart"),
            None,
        ))
        .await
        .expect("should succeed");
    assert_eq!(rs.status, HttpStatus::Ok);

    let put = api
        .handle_request(web_req(
            HttpMethod::Put,
            &format!("/api/plugins/{id2}/config"),
            Some(json!({"configuration": {"k": "v"}})),
        ))
        .await
        .expect("should succeed");
    assert_eq!(put.status, HttpStatus::Ok);

    let ex = api
        .handle_request(web_req(
            HttpMethod::Post,
            &format!("/api/plugins/{id2}/execute"),
            Some(json!({
                "command": "ping",
                "parameters": {}
            })),
        ))
        .await
        .expect("should succeed");
    assert_eq!(ex.status, HttpStatus::Ok);
    assert_eq!(ex.body.as_ref().expect("should succeed")["command"], "ping");
}

#[tokio::test]
async fn marketplace_paths_with_wrong_segment_return_err() {
    let manager = Arc::new(DefaultPluginManager::new());
    let api = PluginManagementAPI::new(manager);
    let pid = Uuid::new_v4();
    let err = api
        .handle_request(web_req(
            HttpMethod::Get,
            &format!("/api/marketplace/plugins/{pid}"),
            None,
        ))
        .await;
    assert!(err.is_err());

    let err2 = api
        .handle_request(web_req(
            HttpMethod::Post,
            &format!("/api/marketplace/plugins/{pid}/install"),
            None,
        ))
        .await;
    assert!(err2.is_err());
}

#[tokio::test]
async fn websocket_handler_register_and_remove() {
    let manager = Arc::new(DefaultPluginManager::new());
    let api = Arc::new(PluginManagementAPI::new(manager));
    let h = PluginWebSocketHandler::new(api.clone());
    let cid = Uuid::new_v4();
    h.handle_connection(cid).await.expect("should succeed");
    assert_eq!(api.websocket_connections.read().await.len(), 1);
    h.handle_disconnection(cid).await.expect("should succeed");
    assert_eq!(api.websocket_connections.read().await.len(), 0);
}

#[tokio::test]
async fn get_plugin_unknown_id_errors() {
    let manager = Arc::new(DefaultPluginManager::new());
    let api = PluginManagementAPI::new(manager);
    let err = api
        .handle_request(web_req(
            HttpMethod::Get,
            &format!("/api/plugins/{}", Uuid::new_v4()),
            None,
        ))
        .await;
    assert!(err.is_err());
}

#[tokio::test]
async fn post_plugins_invalid_json_errors() {
    let manager = Arc::new(DefaultPluginManager::new());
    let api = PluginManagementAPI::new(manager);
    let err = api
        .handle_request(web_req(
            HttpMethod::Post,
            "/api/plugins",
            Some(json!("not-object")),
        ))
        .await;
    assert!(err.is_err());
}
