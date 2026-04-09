// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use super::ApiServer;
use crate::ecosystem::EcosystemService;
use crate::federation::FederationService;
use crate::monitoring::MonitoringConfig;
use crate::routing::McpRoutingService;
use crate::{
    AgentSpec, DiscoveryConfig, EcosystemConfig, EcosystemMode, FederationConfig, McpTask,
    MonitoringService, RoutingConfig, Task, TaskPriority, TaskRequirements, TaskType,
};
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use chrono::Duration as ChronoDuration;
use ecosystem_api::PrimalType;
use std::collections::HashMap;
use std::sync::Arc;
use tower05::util::ServiceExt;

fn test_routing() -> Arc<McpRoutingService> {
    Arc::new(McpRoutingService::new(RoutingConfig::default()).expect("routing config"))
}

fn test_federation() -> Arc<FederationService> {
    Arc::new(FederationService::new(FederationConfig::default()).expect("fed"))
}

fn test_ecosystem_disabled() -> Arc<EcosystemService> {
    Arc::new(
        EcosystemService::new(
            EcosystemConfig {
                enabled: false,
                mode: EcosystemMode::Standalone,
                discovery: DiscoveryConfig::default(),
            },
            Arc::new(MonitoringService::new(MonitoringConfig::default())),
        )
        .expect("eco"),
    )
}

fn test_ecosystem_coordinated() -> Arc<EcosystemService> {
    Arc::new(
        EcosystemService::new(
            EcosystemConfig {
                enabled: true,
                mode: EcosystemMode::Coordinated,
                discovery: DiscoveryConfig {
                    auto_discovery: false,
                    discovery_endpoint: None,
                    direct_endpoints: HashMap::new(),
                    probe_interval: ChronoDuration::seconds(60),
                    health_check_timeout: ChronoDuration::seconds(5),
                },
            },
            Arc::new(MonitoringService::new(MonitoringConfig::default())),
        )
        .expect("eco"),
    )
}

fn app_disabled_eco() -> axum::Router {
    ApiServer::new(test_ecosystem_disabled(), test_routing(), test_federation()).create_router()
}

fn app_coord_eco() -> axum::Router {
    ApiServer::new(
        test_ecosystem_coordinated(),
        test_routing(),
        test_federation(),
    )
    .create_router()
}

async fn read_body_json(resp: axum::response::Response) -> serde_json::Value {
    let bytes = to_bytes(resp.into_body(), usize::MAX).await.expect("body");
    serde_json::from_slice(&bytes).expect("json")
}

#[tokio::test]
async fn health_returns_healthy_payload() {
    let app = app_disabled_eco();
    let res = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .expect("req"),
        )
        .await
        .expect("call");
    assert_eq!(res.status(), StatusCode::OK);
    let v = read_body_json(res).await;
    assert_eq!(v["status"], "healthy");
}

#[tokio::test]
async fn route_mcp_without_agents_returns_500() {
    let app = app_disabled_eco();
    let task = McpTask {
        id: "t1".to_string(),
        agent_id: None,
        payload: serde_json::json!({}),
        context: None,
        routing_hints: vec![],
        context_requirements: None,
        mcp_request: serde_json::json!({}),
    };
    let body = serde_json::to_vec(&task).expect("ser");
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/route")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .expect("req"),
        )
        .await
        .expect("call");
    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn route_mcp_after_register_agent_succeeds() {
    let app = app_disabled_eco();
    let agent = AgentSpec {
        id: "a1".to_string(),
        endpoint: "http://127.0.0.1:1".to_string(),
        capabilities: vec!["mcp".to_string()],
        weight: None,
        max_concurrent_tasks: 4,
        metadata: HashMap::new(),
    };
    let reg = serde_json::to_vec(&agent).expect("ser");
    let res_reg = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/agents")
                .header("content-type", "application/json")
                .body(Body::from(reg))
                .expect("req"),
        )
        .await
        .expect("call");
    assert_eq!(res_reg.status(), StatusCode::OK);

    let task = McpTask {
        id: "t1".to_string(),
        agent_id: None,
        payload: serde_json::json!({}),
        context: None,
        routing_hints: vec![],
        context_requirements: None,
        mcp_request: serde_json::json!({}),
    };
    let body = serde_json::to_vec(&task).expect("ser");
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/route")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .expect("req"),
        )
        .await
        .expect("call");
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn info_returns_node_and_version_fields() {
    let app = app_disabled_eco();
    let res = app
        .oneshot(
            Request::builder()
                .uri("/info")
                .body(Body::empty())
                .expect("req"),
        )
        .await
        .expect("call");
    assert_eq!(res.status(), StatusCode::OK);
    let v = read_body_json(res).await;
    assert!(v.get("node_id").is_some());
    assert_eq!(
        v["version"].as_str().unwrap_or(""),
        crate::SQUIRREL_MCP_VERSION
    );
}

#[tokio::test]
async fn coordinate_task_returns_503_when_routing_fails() {
    let app = app_coord_eco();
    let task = Task {
        id: "x1".to_string(),
        task_type: TaskType::McpCoordination,
        priority: TaskPriority::Normal,
        requirements: TaskRequirements {
            cpu: None,
            memory: None,
            storage: None,
            network: None,
            required_capabilities: vec!["missing".to_string()],
            preferred_primals: vec![PrimalType::Squirrel],
            constraints: HashMap::new(),
        },
        context: serde_json::json!({}),
        deadline: None,
    };
    let body = serde_json::to_vec(&task).expect("ser");
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/coordinate")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .expect("req"),
        )
        .await
        .expect("call");
    assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn list_discovered_primals_returns_zero_by_default() {
    let app = app_disabled_eco();
    let res = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/primals")
                .body(Body::empty())
                .expect("req"),
        )
        .await
        .expect("call");
    assert_eq!(res.status(), StatusCode::OK);
    let v = read_body_json(res).await;
    assert_eq!(v["total_primals"], 0);
}

#[tokio::test]
async fn get_task_status_returns_not_found_until_persistence() {
    let app = app_disabled_eco();
    let res = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/tasks/task-abc")
                .body(Body::empty())
                .expect("req"),
        )
        .await
        .expect("call");
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
    let v = read_body_json(res).await;
    assert_eq!(v["task_id"], "task-abc");
    assert_eq!(v["status"], "unknown");
}

#[tokio::test]
async fn shutdown_endpoint_returns_message() {
    let app = app_disabled_eco();
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/shutdown")
                .body(Body::empty())
                .expect("req"),
        )
        .await
        .expect("call");
    assert_eq!(res.status(), StatusCode::OK);
    let v = read_body_json(res).await;
    assert_eq!(v["message"], "Shutdown initiated");
}
