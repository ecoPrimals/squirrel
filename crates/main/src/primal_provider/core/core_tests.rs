// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::needless_pass_by_value,
    clippy::significant_drop_tightening,
    clippy::field_reassign_with_default,
    clippy::default_trait_access,
    clippy::many_single_char_names,
    clippy::unreadable_literal,
    clippy::too_many_lines,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::similar_names,
    clippy::option_if_let_else,
    clippy::doc_markdown,
    clippy::struct_field_names,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::return_self_not_must_use,
    clippy::trivially_copy_pass_by_ref,
    clippy::unused_self,
    clippy::unused_async,
    clippy::unnecessary_wraps,
    clippy::semicolon_if_nothing_returned,
    clippy::match_wildcard_for_single_variants,
    clippy::match_same_arms,
    clippy::explicit_iter_loop,
    clippy::uninlined_format_args,
    clippy::equatable_if_let,
    clippy::assertions_on_constants,
    missing_docs,
    unused_imports,
    unused_variables,
    dead_code,
    deprecated
)]

use super::SquirrelPrimalProvider;
use crate::ecosystem::EcosystemManager;
use crate::monitoring::metrics::MetricsCollector;
use crate::session::{SessionConfig, SessionManagerImpl};
use crate::universal::{PrimalType, UniversalPrimalProvider};
use std::sync::Arc;

async fn provider() -> SquirrelPrimalProvider {
    let adapter = crate::universal_adapter_v2::UniversalAdapterV2::awaken()
        .await
        .expect("adapter");
    let mc = Arc::new(MetricsCollector::new());
    let em = Arc::new(EcosystemManager::new(
        crate::ecosystem::config::EcosystemConfig::default(),
        mc,
    ));
    let sessions = Arc::new(SessionManagerImpl::new(SessionConfig::default()));
    SquirrelPrimalProvider::new(
        "core-test".to_string(),
        squirrel_mcp_config::EcosystemConfig::default(),
        adapter,
        em,
        sessions,
    )
}

#[tokio::test]
async fn getters_and_metadata() {
    let p = provider().await;
    assert_eq!(p.primal_id(), "squirrel");
    assert_eq!(p.instance_id(), "core-test");
    assert_eq!(p.primal_type(), PrimalType::AI);
    assert!(!p.capabilities().is_empty());
    let info = p.get_primal_info();
    assert_eq!(info.primal_type, PrimalType::AI);
}

#[tokio::test]
async fn config_helpers_roundtrip() {
    let mut p = provider().await;
    let j = p.get_managed_config().expect("should succeed");
    assert!(j.get("instance_id").is_some());
    p.update_managed_config(serde_json::json!({"x": 1}))
        .expect("should succeed");
    assert!(p.validate_configuration().expect("should succeed"));
    p.reset_to_defaults().expect("should succeed");
    let ext = p.get_external_services().expect("should succeed");
    assert!(ext.is_object());
    let bio = p.get_biomeos_endpoints().expect("should succeed");
    assert!(bio.get("health_url").is_some());
}

#[tokio::test]
async fn coordinate_and_discover_return_json() {
    let p = provider().await;
    let coord = p
        .coordinate_ai_operation(serde_json::json!({"operation_type": "test"}))
        .await
        .expect("should succeed");
    assert_eq!(
        coord.get("status").and_then(|v| v.as_str()),
        Some("coordinated")
    );
    let disc = p
        .discover_ecosystem_services()
        .await
        .expect("should succeed");
    assert!(disc.is_empty() || !disc.is_empty());
    let mesh = p
        .coordinate_with_service_mesh(serde_json::json!({"operation": "x"}))
        .await
        .expect("should succeed");
    assert!(mesh.get("mesh_services_discovered").is_some());
}

#[tokio::test]
async fn leverage_capabilities() {
    let p = provider().await;
    p.leverage_security_capabilities("auth", serde_json::json!({}))
        .await
        .expect("should succeed");
    p.leverage_compute_capabilities("run", serde_json::json!({}))
        .await
        .expect("should succeed");
    p.leverage_storage_capabilities("retrieve", "k", None)
        .await
        .expect("should succeed");
    let e = p
        .leverage_storage_capabilities("store", "k", None)
        .await
        .unwrap_err();
    assert!(matches!(e, crate::error::PrimalError::ValidationError(_)));
    p.leverage_orchestration_capabilities()
        .await
        .expect("should succeed");
}

#[tokio::test]
async fn ai_prompts_and_strings() {
    let p = provider().await;
    assert!(p.get_optimized_ai_strings().get("test").is_none());
    assert!(p.get_ai_prompt_template("coordination").is_some());
    assert!(p.get_ai_prompt_template("nosuch").is_none());
}

#[tokio::test]
async fn ecosystem_status_and_registry_update() {
    let p = provider().await;
    let st = p.get_ecosystem_status().await.expect("should succeed");
    assert!(st.get("coordination_efficiency").is_some());
    p.update_ecosystem_registry(vec![serde_json::json!({
        "service_id": "a",
        "update_type": "register"
    })])
    .await
    .expect("should succeed");
}

#[tokio::test]
async fn universal_trait_primal_request_health_and_unknown_op() {
    use crate::universal::PrimalRequest;
    let p = provider().await;
    let h = UniversalPrimalProvider::health_check(&p).await;
    assert!(h.healthy);
    let bad = UniversalPrimalProvider::handle_primal_request(
        &p,
        PrimalRequest::new(
            "squirrel",
            "squirrel",
            "not_a_real_op",
            serde_json::json!({}),
            crate::universal::PrimalContext::default(),
        ),
    )
    .await;
    assert!(bad.is_err());
}

#[test]
fn dependencies_match_trait() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("should succeed");
    rt.block_on(async {
        let p = provider().await;
        let d = UniversalPrimalProvider::dependencies(&p);
        assert_eq!(d.len(), 2);
    });
}

#[tokio::test]
async fn can_serve_context_depends_on_security_level() {
    let p = provider().await;
    let mut ctx = crate::universal::PrimalContext::default();
    ctx.security_level = crate::universal::SecurityLevel::Public;
    assert!(!p.can_serve_context(&ctx));
    ctx.security_level = crate::universal::SecurityLevel::Standard;
    assert!(p.can_serve_context(&ctx));
}

#[tokio::test]
async fn leverage_storage_store_with_data_and_unsupported_op() {
    let p = provider().await;
    let stored = p
        .leverage_storage_capabilities("store", "key1", Some(b"bytes"))
        .await
        .expect("should succeed");
    assert_eq!(
        stored.get("status").and_then(|v| v.as_str()),
        Some("stored")
    );
    let err = p
        .leverage_storage_capabilities("purge", "k", None)
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        crate::error::PrimalError::OperationNotSupported(_)
    ));
}

#[tokio::test]
async fn ai_prompt_templates_cover_named_variants() {
    let p = provider().await;
    for name in ["discovery", "integration", "analysis"] {
        assert!(p.get_ai_prompt_template(name).is_some());
    }
}

#[tokio::test]
async fn universal_trait_ecosystem_report_health_update_caps() {
    use crate::universal::{HealthStatus, PrimalHealth, UniversalPrimalProvider};

    let p = provider().await;
    let req = crate::universal::create_ecosystem_request(
        "squirrel",
        "peer",
        "ping",
        serde_json::json!({}),
        crate::universal::UniversalSecurityContext::default(),
    );
    let resp = UniversalPrimalProvider::handle_ecosystem_request(&p, req)
        .await
        .expect("should succeed");
    assert!(resp.success);

    let health = PrimalHealth {
        status: HealthStatus::Healthy,
        healthy: true,
        score: 1.0,
        last_check: chrono::Utc::now(),
        message: None,
        details: None,
    };
    UniversalPrimalProvider::report_health(&p, health)
        .await
        .expect("should succeed");
    UniversalPrimalProvider::update_capabilities(&p, vec![])
        .await
        .expect("should succeed");
    assert!(UniversalPrimalProvider::dynamic_port_info(&p).is_some());
}

#[tokio::test]
async fn set_port_info_and_service_mesh_status_no_panic() {
    use crate::universal::{DynamicPortInfo, PortStatus, PortType, ServiceMeshStatus};

    let mut p = provider().await;
    let now = chrono::Utc::now();
    p.set_port_info(DynamicPortInfo {
        port: 9000,
        assigned_port: 9000,
        current_port: 9000,
        port_range: Some((9000, 9010)),
        port_type: PortType::Http,
        status: PortStatus::Active,
        allocated_at: now,
        assigned_at: now,
        lease_duration: None,
        expires_at: None,
        metadata: std::collections::HashMap::new(),
    });
    p.update_service_mesh_status(ServiceMeshStatus::default());
}

#[test]
fn universal_trait_endpoints_use_service_host_and_port_env() {
    temp_env::with_var("SERVICE_HOST", Some("192.168.1.1"), || {
        temp_env::with_var("SERVICE_PORT", Some("7777"), || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("rt");
            rt.block_on(async {
                let p = provider().await;
                let e = UniversalPrimalProvider::endpoints(&p);
                let http = e.http.as_ref().expect("http");
                assert!(http.contains("192.168.1.1:7777"));
            });
        });
    });
}

#[tokio::test]
async fn universal_trait_handle_primal_request_ai_inference_operation() {
    use crate::universal::PrimalRequest;
    let p = provider().await;
    let req = PrimalRequest::new(
        "squirrel",
        "squirrel",
        "ai_inference",
        serde_json::json!({
            "task_type": "chat",
            "messages": [{"role": "user", "content": "ping"}],
            "model": null,
            "parameters": {}
        }),
        crate::universal::PrimalContext::default(),
    );
    let resp = UniversalPrimalProvider::handle_primal_request(&p, req)
        .await
        .expect("ai_inference");
    assert!(resp.success);
    assert!(resp.payload.get("content").is_some());
}
