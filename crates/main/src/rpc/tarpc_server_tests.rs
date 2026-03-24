// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use super::*;
use crate::rpc::tarpc_service::{
    AnnounceCapabilitiesParams, ContextCreateParams, ContextSummarizeParams, ContextUpdateParams,
    QueryAiParams, ToolSource,
};
use serde_json::json;
use std::sync::Arc;
use tarpc::context;

#[tokio::test]
async fn test_tarpc_server_from_jsonrpc() {
    let jsonrpc = Arc::new(crate::rpc::JsonRpcServer::new("/tmp/test.sock".to_string()));
    let server = TarpcRpcServer::from_jsonrpc(jsonrpc);
    let ctx = context::current();
    let ping = server.system_ping(ctx).await;
    assert!(ping.pong);
    assert!(!ping.version.is_empty());
}

#[tokio::test]
async fn test_tarpc_system_health() {
    let jsonrpc = Arc::new(crate::rpc::JsonRpcServer::new("/tmp/test.sock".to_string()));
    let server = TarpcRpcServer::from_jsonrpc(jsonrpc);
    let ctx = context::current();
    let health = server.system_health(ctx).await;
    assert_eq!(health.status, "ready");
    assert_eq!(health.tier, crate::rpc::types::HealthTier::Ready);
    assert_eq!(health.version, env!("CARGO_PKG_VERSION"));
}

#[tokio::test]
async fn test_tarpc_ai_list_providers_no_router() {
    let jsonrpc = Arc::new(crate::rpc::JsonRpcServer::new("/tmp/test.sock".to_string()));
    let server = TarpcRpcServer::from_jsonrpc(jsonrpc);
    let ctx = context::current();
    let result = server.ai_list_providers(ctx).await;
    assert_eq!(result.total, 0);
    assert!(result.providers.is_empty());
}

#[test]
fn json_to_query_result_defaults_and_fields() {
    let empty = json!({});
    let r = TarpcRpcServer::json_to_query_result(&empty);
    assert_eq!(r.response, "");
    assert_eq!(r.provider.as_ref(), "none");
    assert_eq!(r.model.as_ref(), "none");
    assert!(r.tokens_used.is_none());
    assert_eq!(r.latency_ms, 0);
    assert!(!r.success);

    let full = json!({
        "response": "hi",
        "provider": "p",
        "model": "m",
        "tokens_used": 42,
        "latency_ms": 7,
        "success": true
    });
    let r2 = TarpcRpcServer::json_to_query_result(&full);
    assert_eq!(r2.response, "hi");
    assert_eq!(r2.provider.as_ref(), "p");
    assert_eq!(r2.model.as_ref(), "m");
    assert_eq!(r2.tokens_used, Some(42));
    assert_eq!(r2.latency_ms, 7);
    assert!(r2.success);
}

#[test]
fn json_to_list_providers_result_parses_and_skips_invalid() {
    let v = json!({
        "total": 2,
        "providers": [
            {
                "id": "a",
                "name": "A",
                "models": ["m1"],
                "capabilities": ["c1"],
                "online": true,
                "avg_latency_ms": 12.5,
                "cost_tier": "low"
            },
            { "id": "broken" }
        ]
    });
    let r = TarpcRpcServer::json_to_list_providers_result(&v);
    assert_eq!(r.total, 2);
    assert_eq!(r.providers.len(), 1);
    assert_eq!(r.providers[0].id.as_ref(), "a");
    assert_eq!(r.providers[0].models[0].as_ref(), "m1");
}

#[test]
fn json_to_announce_and_health_and_ping() {
    let ann = json!({
        "success": true,
        "message": "ok",
        "announced_at": "t0",
        "tools_registered": 3
    });
    let a = TarpcRpcServer::json_to_announce_result(&ann);
    assert!(a.success);
    assert_eq!(a.message, "ok");
    assert_eq!(a.announced_at, "t0");
    assert_eq!(a.tools_registered, 3);

    let health = json!({
        "tier": "ready",
        "alive": true,
        "ready": true,
        "healthy": false,
        "status": "degraded",
        "version": "v9",
        "uptime_seconds": 100,
        "active_providers": 2,
        "requests_processed": 50,
        "avg_response_time_ms": 1.25
    });
    let h = TarpcRpcServer::json_to_health_result(&health);
    assert_eq!(h.status, "degraded");
    assert_eq!(h.version, "v9");
    assert_eq!(h.uptime_seconds, 100);
    assert_eq!(h.active_providers, 2);
    assert_eq!(h.requests_processed, 50);
    assert_eq!(h.avg_response_time_ms, Some(1.25));

    let ping = json!({
        "pong": false,
        "timestamp": "ts",
        "version": "pv"
    });
    let p = TarpcRpcServer::json_to_ping_result(&ping);
    assert!(!p.pong);
    assert_eq!(p.timestamp, "ts");
    assert_eq!(p.version, "pv");
}

#[test]
fn json_to_discovery_peers_result() {
    let v = json!({
        "total": 1,
        "peers": [{
            "id": "peer1",
            "socket": "/tmp/s",
            "capabilities": ["a"],
            "discovered_via": "mdns"
        }],
        "discovery_method": "registry"
    });
    let r = TarpcRpcServer::json_to_discovery_peers_result(&v);
    assert_eq!(r.total, 1);
    assert_eq!(r.peers.len(), 1);
    assert_eq!(r.peers[0].id, "peer1");
    assert_eq!(r.discovery_method, "registry");

    let missing_caps = json!({
        "peers": [{ "id": "x", "socket": "y" }]
    });
    let r2 = TarpcRpcServer::json_to_discovery_peers_result(&missing_caps);
    assert!(r2.peers.is_empty());
}

#[test]
fn json_to_tool_execute_and_list() {
    let ex = json!({
        "tool": "t1",
        "success": true,
        "output": "out",
        "error": null,
        "timestamp": "now"
    });
    let e = TarpcRpcServer::json_to_tool_execute_result(&ex);
    assert_eq!(e.tool, "t1");
    assert!(e.success);
    assert_eq!(e.output, "out");
    assert!(e.error.is_none());

    let list = json!({
        "total": 2,
        "tools": [
            {
                "name": "n",
                "description": "d",
                "domain": "dom",
                "source": { "Remote": { "primal": "bird" } },
                "input_schema": { "type": "object" }
            },
            {
                "name": "b",
                "description": "bd",
                "domain": "bd",
                "source": { "primal": "inline" }
            }
        ]
    });
    let t = TarpcRpcServer::json_to_tool_list_result(&list);
    assert_eq!(t.total, 2);
    assert_eq!(t.tools.len(), 2);
    assert!(
        matches!(&t.tools[0].source, ToolSource::Remote { .. }),
        "expected Remote, got {:?}",
        t.tools[0].source
    );
    if let ToolSource::Remote { primal } = &t.tools[0].source {
        assert_eq!(primal, "bird");
    }
    assert!(
        matches!(&t.tools[1].source, ToolSource::Remote { .. }),
        "expected Remote from primal key, got {:?}",
        t.tools[1].source
    );
    if let ToolSource::Remote { primal } = &t.tools[1].source {
        assert_eq!(primal, "inline");
    }
}

#[test]
fn json_to_capability_discover_and_system_metrics() {
    let cap = json!({
        "primal": "sq",
        "capabilities": ["x"],
        "version": "1",
        "metadata": { "k": "v" }
    });
    let c = TarpcRpcServer::json_to_capability_discover_result(&cap);
    assert_eq!(c.primal, "sq");
    assert_eq!(c.capabilities, vec!["x"]);
    assert_eq!(c.version, "1");
    assert!(c.metadata.contains_key("k"));

    let m = json!({
        "requests_handled": 9,
        "errors": 1,
        "uptime_seconds": 3,
        "avg_response_time_ms": 4.0,
        "success_rate": 0.9
    });
    let sm = TarpcRpcServer::json_to_system_metrics_result(&m);
    assert_eq!(sm.requests_handled, 9);
    assert_eq!(sm.errors, 1);
    assert_eq!(sm.uptime_seconds, 3);
    assert_eq!(sm.avg_response_time_ms, Some(4.0));
    assert!((sm.success_rate - 0.9).abs() < f64::EPSILON);
}

#[test]
fn json_to_edge_defaults_and_tool_list_builtin() {
    let ex = json!({
        "tool": "t",
        "success": false,
        "output": "",
        "error": "boom",
        "timestamp": "t1"
    });
    let e = TarpcRpcServer::json_to_tool_execute_result(&ex);
    assert_eq!(e.error.as_deref(), Some("boom"));

    let list = json!({
        "total": 1,
        "tools": [{
            "name": "builtin.tool",
            "description": "d",
            "domain": "d0",
            "input_schema": null
        }]
    });
    let t = TarpcRpcServer::json_to_tool_list_result(&list);
    assert_eq!(t.tools.len(), 1);
    assert!(matches!(t.tools[0].source, ToolSource::Builtin));

    let peers = json!({
        "total": 1,
        "peers": [{
            "id": "p",
            "socket": "/s",
            "capabilities": ["c"],
            "discovered_via": ""
        }]
    });
    let d = TarpcRpcServer::json_to_discovery_peers_result(&peers);
    assert_eq!(d.peers[0].discovered_via, "");

    let cap = json!({
        "primal": "x",
        "capabilities": [],
        "version": ""
    });
    let c = TarpcRpcServer::json_to_capability_discover_result(&cap);
    assert!(c.metadata.is_empty());

    let m = json!({});
    let sm = TarpcRpcServer::json_to_system_metrics_result(&m);
    assert_eq!(sm.requests_handled, 0);
    assert!((sm.success_rate - 1.0).abs() < f64::EPSILON);
}

#[tokio::test]
async fn tarpc_delegates_ai_aliases_system_metrics_status_and_tools() {
    let jsonrpc = Arc::new(crate::rpc::JsonRpcServer::new(
        "/tmp/tarpc-delegate.sock".to_string(),
    ));
    let server = TarpcRpcServer::from_jsonrpc(jsonrpc);

    let q = QueryAiParams {
        prompt: "p".to_string(),
        model: None,
        max_tokens: None,
        temperature: None,
    };
    let r = server
        .clone()
        .ai_complete(context::current(), q.clone())
        .await;
    assert!(!r.success);
    let r2 = server.clone().ai_chat(context::current(), q).await;
    assert!(!r2.success);

    let m = server.clone().system_metrics(context::current()).await;
    assert!(m.success_rate <= 1.0);

    let h = server.clone().system_status(context::current()).await;
    assert_eq!(h.status, "ready");

    let disc = server.clone().capability_discover(context::current()).await;
    assert_eq!(disc.primal, "squirrel");
    assert!(!disc.capabilities.is_empty());

    let peers = server.clone().discovery_peers(context::current()).await;
    assert_eq!(peers.peers.len(), peers.total);

    let tools = server.clone().tool_list(context::current()).await;
    assert!(tools.total > 0);

    let exec = server
        .clone()
        .tool_execute(
            context::current(),
            "system.health".to_string(),
            std::collections::HashMap::new(),
        )
        .await;
    assert!(exec.success);

    let ann = server
        .clone()
        .capability_announce(
            context::current(),
            AnnounceCapabilitiesParams {
                capabilities: vec!["ai.query".to_string()],
                primal: Some("p".to_string()),
                socket_path: None,
                tools: None,
                sub_federations: None,
                genetic_families: None,
            },
        )
        .await;
    assert!(ann.success);

    let life = server.clone().lifecycle_register(context::current()).await;
    assert!(life.success);

    let st = server.lifecycle_status(context::current()).await;
    assert_eq!(st.status, "healthy");
}

#[tokio::test]
async fn tarpc_context_roundtrip() {
    let jsonrpc = Arc::new(crate::rpc::JsonRpcServer::new(
        "/tmp/tarpc-ctx.sock".to_string(),
    ));
    let server = TarpcRpcServer::from_jsonrpc(jsonrpc);
    let ctx = context::current();

    let created = server
        .clone()
        .context_create(
            ctx,
            ContextCreateParams {
                session_id: Some("sess-tarpc-1".to_string()),
                metadata: Some(json!({ "k": "v" })),
            },
        )
        .await;
    assert!(!created.id.is_empty());

    let updated = server
        .clone()
        .context_update(
            context::current(),
            ContextUpdateParams {
                id: created.id.clone(),
                data: json!({ "x": 1 }),
            },
        )
        .await;
    assert_eq!(updated.id, created.id);
    assert!(updated.version >= 1);

    let sum = server
        .context_summarize(
            context::current(),
            ContextSummarizeParams { id: created.id },
        )
        .await;
    assert!(!sum.summary.is_empty());
}
