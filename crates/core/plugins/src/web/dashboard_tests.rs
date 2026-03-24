// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use super::*;
use crate::DefaultPluginManager;
use crate::types::PluginStatus;
use std::sync::Arc;

#[tokio::test]
async fn test_plugin_dashboard_creation() {
    let manager = Arc::new(DefaultPluginManager::new());
    let dashboard = PluginDashboard::new(manager);

    let endpoints = dashboard.get_endpoints();
    assert!(!endpoints.is_empty());
    assert!(
        endpoints
            .iter()
            .any(|ep| ep.path == "/api/dashboard/overview")
    );
}

#[tokio::test]
async fn test_dashboard_overview() {
    let manager = Arc::new(DefaultPluginManager::new());
    let dashboard = PluginDashboard::new(manager);

    let response = dashboard
        .get_dashboard_overview()
        .await
        .expect("should succeed");
    assert_eq!(response.status, HttpStatus::Ok);

    let body = response.body.expect("should succeed");
    assert!(body.get("plugin_stats").is_some());
    assert!(body.get("system_health").is_some());
    assert!(body.get("recent_activities").is_some());
}

#[tokio::test]
async fn test_plugin_statistics() {
    let manager = Arc::new(DefaultPluginManager::new());
    let dashboard = PluginDashboard::new(manager);

    let stats = dashboard.collect_plugin_statistics().await;
    assert_eq!(stats.total_plugins, 0); // No plugins registered in test
}

#[test]
fn dashboard_config_default_and_serde_roundtrip() {
    let d = DashboardConfig::default();
    assert_eq!(d.refresh_interval, 30);
    assert!(!d.theme.dark_mode);
    let j = serde_json::to_string(&d).expect("should succeed");
    let back: DashboardConfig = serde_json::from_str(&j).expect("should succeed");
    assert_eq!(back.refresh_interval, d.refresh_interval);
}

#[test]
fn dashboard_types_serde_roundtrip() {
    let overview = DashboardOverview {
        plugin_stats: PluginStatistics {
            total_plugins: 1,
            active_plugins: 1,
            inactive_plugins: 0,
            failed_plugins: 0,
            pending_updates: 0,
            memory_usage_mb: Some(1.5),
            cpu_usage_percent: None,
        },
        system_health: SystemHealth {
            status: HealthStatus::Healthy,
            uptime_seconds: 1,
            memory_usage: Some(MemoryUsage {
                used_mb: 1.0,
                total_mb: 8.0,
                usage_percent: 12.5,
            }),
            cpu_usage: Some(CpuUsage {
                current_percent: 1.0,
                avg_1min_percent: 2.0,
                avg_5min_percent: 3.0,
            }),
            disk_usage: Some(DiskUsage {
                used_mb: 10.0,
                total_mb: 100.0,
                usage_percent: 10.0,
            }),
            network_stats: Some(NetworkStats {
                bytes_sent: 1,
                bytes_received: 2,
                packets_sent: 3,
                packets_received: 4,
            }),
            observability_hints: Some(vec!["hint".to_string()]),
        },
        recent_activities: vec![],
        quick_actions: vec![],
        alerts: vec![],
    };
    let j = serde_json::to_string(&overview).expect("should succeed");
    let _: DashboardOverview = serde_json::from_str(&j).expect("should succeed");

    let alert = Alert {
        id: Uuid::new_v4(),
        level: AlertLevel::Warning,
        title: "t".to_string(),
        message: "m".to_string(),
        timestamp: chrono::Utc::now(),
        dismissed: false,
        actions: vec![AlertAction {
            id: "a".to_string(),
            title: "at".to_string(),
            url: "/u".to_string(),
            method: "GET".to_string(),
        }],
    };
    let j = serde_json::to_string(&alert).expect("should succeed");
    let _: Alert = serde_json::from_str(&j).expect("should succeed");
}

#[tokio::test]
async fn with_config_and_get_components() {
    let manager = Arc::new(DefaultPluginManager::new());
    let cfg = DashboardConfig {
        show_marketplace: false,
        ..Default::default()
    };
    let d = PluginDashboard::with_config(manager, cfg);
    let comps = d.get_components();
    assert_eq!(comps.len(), 5);
    assert!(comps.iter().any(|c| c.name == "plugin-dashboard"));
}

#[tokio::test]
async fn handle_request_routes_and_not_found() {
    let manager = Arc::new(DefaultPluginManager::new());
    let dashboard = PluginDashboard::new(manager);

    for (path, method) in [
        ("/api/dashboard/overview", HttpMethod::Get),
        ("/api/dashboard/stats", HttpMethod::Get),
        ("/api/dashboard/health", HttpMethod::Get),
        ("/api/dashboard/activities", HttpMethod::Get),
        ("/api/dashboard/alerts", HttpMethod::Get),
        ("/api/dashboard/config", HttpMethod::Get),
    ] {
        let r = WebRequest {
            method,
            path: path.to_string(),
            query_params: HashMap::new(),
            headers: HashMap::new(),
            body: None,
            user_id: None,
            permissions: vec![],
            route_params: HashMap::new(),
        };
        let res = dashboard.handle_request(r).await.expect("should succeed");
        assert_eq!(res.status, HttpStatus::Ok, "path={path}");
    }

    let nf = dashboard
        .handle_request(WebRequest {
            method: HttpMethod::Get,
            path: "/api/dashboard/nope".to_string(),
            query_params: HashMap::new(),
            headers: HashMap::new(),
            body: None,
            user_id: None,
            permissions: vec![],
            route_params: HashMap::new(),
        })
        .await
        .expect("should succeed");
    assert_eq!(nf.status, HttpStatus::NotFound);
}

#[tokio::test]
async fn put_config_and_dismiss_alert() {
    let manager = Arc::new(DefaultPluginManager::new());
    let dashboard = PluginDashboard::new(manager);
    let new_cfg = DashboardConfig {
        refresh_interval: 60,
        recent_activities_limit: 5,
        show_system_metrics: false,
        show_marketplace: true,
        theme: DashboardTheme {
            primary_color: "#000".to_string(),
            secondary_color: "#111".to_string(),
            background_color: "#fff".to_string(),
            text_color: "#222".to_string(),
            dark_mode: true,
        },
    };
    let put = dashboard
        .handle_request(WebRequest {
            method: HttpMethod::Put,
            path: "/api/dashboard/config".to_string(),
            query_params: HashMap::new(),
            headers: HashMap::new(),
            body: Some(serde_json::to_value(&new_cfg).expect("should succeed")),
            user_id: None,
            permissions: vec![],
            route_params: HashMap::new(),
        })
        .await
        .expect("should succeed");
    assert_eq!(put.status, HttpStatus::Ok);

    let aid = Uuid::new_v4();
    let dismiss = dashboard
        .handle_request(WebRequest {
            method: HttpMethod::Post,
            path: format!("/api/dashboard/alerts/{aid}/dismiss"),
            query_params: HashMap::new(),
            headers: HashMap::new(),
            body: None,
            user_id: None,
            permissions: vec![],
            route_params: HashMap::new(),
        })
        .await
        .expect("should succeed");
    assert_eq!(dismiss.status, HttpStatus::Ok);
}

#[tokio::test]
async fn extract_alert_id_errors() {
    let manager = Arc::new(DefaultPluginManager::new());
    let d = PluginDashboard::new(manager);
    assert!(d.extract_alert_id("/api/dashboard/alerts").is_err());
    assert!(
        d.extract_alert_id("/api/dashboard/alerts/not-uuid/dismiss")
            .is_err()
    );
}

#[tokio::test]
async fn extract_alert_id_ok() {
    let manager = Arc::new(DefaultPluginManager::new());
    let d = PluginDashboard::new(manager);
    let id = Uuid::new_v4();
    assert_eq!(
        d.extract_alert_id(&format!("/api/dashboard/alerts/{id}/dismiss"))
            .expect("should succeed"),
        id
    );
}

#[tokio::test]
async fn failed_plugin_produces_alert_and_dismiss_hides() {
    use crate::discovery::create_noop_plugin;
    use crate::plugin::PluginMetadata;
    use crate::registry::PluginRegistry;

    let manager = Arc::new(DefaultPluginManager::new());
    let meta = PluginMetadata::new("bad", "1.0.0", "d", "a");
    let id = meta.id;
    let p = create_noop_plugin(meta);
    manager.register_plugin(p).await.expect("should succeed");
    PluginRegistry::set_plugin_status(manager.as_ref(), id, PluginStatus::Failed)
        .await
        .expect("should succeed");

    let dashboard = PluginDashboard::new(manager);
    let alerts = dashboard.collect_registry_alerts().await;
    assert_eq!(alerts.len(), 1);
    let alert_id = alerts[0].id;

    dashboard
        .dismiss_alert(alert_id)
        .await
        .expect("should succeed");
    let after = dashboard.collect_registry_alerts().await;
    assert!(after.is_empty());
}
