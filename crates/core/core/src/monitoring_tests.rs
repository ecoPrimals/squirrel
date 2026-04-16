// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use super::monitoring_provider::{MonitoringProviderImpl, TestFailingProvider, TestOkProvider};
use super::*;
use crate::HealthStatus;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

fn base_metric() -> Metric {
    Metric {
        name: "m".into(),
        value: MetricValue::Counter(1),
        labels: HashMap::new(),
        timestamp: Utc::now(),
    }
}

#[test]
fn monitoring_config_and_fallback_defaults() {
    let mc = MonitoringConfig::default();
    assert!(mc.enabled);
    assert!(!mc.require_provider);
    assert!(mc.monitoring_service_config.is_none());
    assert!(mc.provider_configs.is_empty());
    let fc = FallbackConfig::default();
    assert_eq!(fc.log_level, "info");
    assert!(fc.include_metrics && fc.include_health && fc.include_performance);
}

#[test]
fn monitoring_config_roundtrip_serde_without_monitoring_service() {
    let cfg = MonitoringConfig::default();
    let json = serde_json::to_string(&cfg).expect("serialize");
    let back: MonitoringConfig = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back.enabled, cfg.enabled);
    assert_eq!(back.require_provider, cfg.require_provider);
}

#[test]
fn metric_event_types_serde_roundtrip() {
    let ts = Utc::now();
    let ev = MonitoringEvent::ServiceStarted {
        service: "s".into(),
        version: "1".into(),
        timestamp: ts,
    };
    let v = serde_json::to_value(&ev).expect("should succeed");
    let back: MonitoringEvent = serde_json::from_value(v).expect("should succeed");
    assert!(matches!(back, MonitoringEvent::ServiceStarted { .. }));

    let m = Metric {
        name: "n".into(),
        value: MetricValue::Histogram {
            buckets: vec![0.0, 1.0],
            counts: vec![1, 2],
        },
        labels: std::iter::once(("k".into(), "v".into())).collect(),
        timestamp: ts,
    };
    let mv = serde_json::to_value(&m).expect("should succeed");
    let mb: Metric = serde_json::from_value(mv).expect("should succeed");
    assert_eq!(mb.name, "n");
}

#[tokio::test]
async fn initialize_disabled_short_circuits() {
    let cfg = MonitoringConfig {
        enabled: false,
        ..MonitoringConfig::default()
    };
    let svc = MonitoringService::new(cfg);
    svc.initialize().expect("init");
}

#[tokio::test]
async fn initialize_require_provider_errors_without_providers() {
    let cfg = MonitoringConfig {
        require_provider: true,
        monitoring_service_config: None,
        ..MonitoringConfig::default()
    };
    let svc = MonitoringService::new(cfg);
    let err = svc.initialize().unwrap_err();
    assert!(
        matches!(err, crate::Error::Monitoring(ref s) if s.contains("No monitoring")),
        "{err:?}"
    );
}

#[tokio::test]
async fn initialize_with_monitoring_service_config_adds_provider() {
    let cfg = MonitoringConfig {
        require_provider: false,
        monitoring_service_config: Some(MonitoringServiceConfig {
            endpoint: "unix:///tmp/sb".into(),
            service_name: "svc".into(),
            auth_token: None,
            batch_size: 10,
            flush_interval: std::time::Duration::from_secs(1),
        }),
        ..MonitoringConfig::default()
    };
    let svc = MonitoringService::new(cfg);
    svc.initialize().expect("init");
    let names = svc.get_providers();
    assert!(
        names
            .iter()
            .any(|n| n == universal_constants::capabilities::SERVICE_MESH_CAPABILITY)
    );
}

#[tokio::test]
async fn initialize_unknown_provider_is_skipped_with_warning_path() {
    let cfg = MonitoringConfig {
        require_provider: false,
        provider_configs: std::iter::once(("not-monitoring".into(), serde_json::json!({})))
            .collect(),
        ..MonitoringConfig::default()
    };
    let svc = MonitoringService::new(cfg);
    svc.initialize().expect("init");
    assert!(svc.get_providers().is_empty());
}

#[tokio::test]
async fn initialize_provider_configs_monitoring_service_branch() {
    let cfg = MonitoringConfig {
        monitoring_service_config: None,
        provider_configs: std::iter::once((
            "monitoring".into(),
            serde_json::json!({
                "endpoint": "unix:///x",
                "service_name": "n",
                "auth_token": null,
                "batch_size": 1,
                "flush_interval": {"secs": 1, "nanos": 0}
            }),
        ))
        .collect(),
        ..MonitoringConfig::default()
    };
    let svc = MonitoringService::new(cfg);
    svc.initialize().expect("init");
    assert!(
        svc.get_providers()
            .iter()
            .any(|n| n == universal_constants::capabilities::SERVICE_MESH_CAPABILITY)
    );
}

#[tokio::test]
async fn monitoring_service_provider_new_succeeds_and_unknown_provider_config_skipped() {
    let res = MonitoringServiceProvider::new(MonitoringServiceConfig {
        endpoint: "e".into(),
        service_name: "s".into(),
        auth_token: None,
        batch_size: 1,
        flush_interval: std::time::Duration::from_millis(0),
    });
    assert!(res.is_ok());
    let cfg = MonitoringConfig {
        provider_configs: std::iter::once(("unknown".into(), serde_json::json!({}))).collect(),
        ..MonitoringConfig::default()
    };
    let svc = MonitoringService::new(cfg);
    svc.initialize().expect("no required provider");
}

#[tokio::test]
async fn record_paths_with_fallback_logger() {
    let cfg = MonitoringConfig {
        enabled: true,
        require_provider: false,
        monitoring_service_config: None,
        provider_configs: HashMap::new(),
        fallback_config: FallbackConfig {
            log_level: "debug".into(),
            ..FallbackConfig::default()
        },
    };
    let svc = MonitoringService::new(cfg);
    let ts = Utc::now();
    svc.record_event(MonitoringEvent::ServiceStopped {
        service: "x".into(),
        timestamp: ts,
    })
    .await
    .expect("should succeed");
    svc.record_metric(base_metric())
        .await
        .expect("should succeed");
    svc.record_health("c", HealthStatus::Healthy)
        .await
        .expect("should succeed");
    svc.record_performance(
        "p",
        PerformanceMetrics {
            cpu_usage: Some(0.5),
            memory_usage: None,
            network_usage: None,
            response_time: Some(std::time::Duration::from_millis(12)),
            throughput: None,
            error_rate: None,
            queue_length: None,
            active_connections: None,
            custom_metrics: HashMap::new(),
        },
    )
    .await
    .expect("should succeed");
    let st = svc.get_status().await;
    assert!(st.fallback_active);
    assert_eq!(st.provider_count, 0);
}

#[tokio::test]
async fn record_with_failing_provider_still_ok() {
    let svc = MonitoringService::new(MonitoringConfig::default());
    svc.add_provider(Arc::new(MonitoringProviderImpl::TestFailing(
        TestFailingProvider,
    )));
    svc.record_event(MonitoringEvent::Custom {
        event_type: "t".into(),
        data: serde_json::json!({}),
        timestamp: Utc::now(),
    })
    .await
    .expect("should succeed");
}

#[tokio::test]
async fn add_remove_get_providers_and_status() {
    let svc = MonitoringService::new(MonitoringConfig::default());
    svc.add_provider(Arc::new(MonitoringProviderImpl::TestOk(TestOkProvider(
        "p1",
    ))));
    svc.add_provider(Arc::new(MonitoringProviderImpl::TestOk(TestOkProvider(
        "p2",
    ))));
    assert_eq!(svc.get_providers().len(), 2);
    svc.remove_provider("p1");
    assert_eq!(svc.get_providers(), vec!["p2"]);
    let st = svc.get_status().await;
    assert_eq!(st.provider_count, 1);
    assert!(!st.fallback_active);
    assert_eq!(st.providers[0].name, "p2");
}

#[test]
fn monitoring_service_provider_ipc_healthy_then_degraded_when_socket_removed() {
    let dir = tempfile::tempdir().expect("tempdir");
    let runtime_dir = dir.path();
    let biomeos = runtime_dir.join("biomeos");
    std::fs::create_dir_all(&biomeos).expect("mkdir");
    let sock_path = biomeos.join("monitoring.sock");

    let rt = tokio::runtime::Runtime::new().expect("rt");
    let _enter = rt.enter();
    let listener = tokio::net::UnixListener::bind(&sock_path).expect("bind");

    let server = rt.spawn(async move {
        for _ in 0..16 {
            let Ok(Ok((mut stream, _))) =
                tokio::time::timeout(std::time::Duration::from_millis(500), listener.accept())
                    .await
            else {
                break;
            };
            let mut buf = Vec::new();
            if stream.read_to_end(&mut buf).await.is_err() {
                continue;
            }
            let Ok(req) = serde_json::from_slice::<serde_json::Value>(&buf) else {
                continue;
            };
            let id = req
                .get("id")
                .cloned()
                .unwrap_or_else(|| serde_json::json!(0));
            let resp = serde_json::json!({ "jsonrpc": "2.0", "id": id, "result": {} });
            let mut out = serde_json::to_vec(&resp).expect("serialize response");
            out.push(b'\n');
            let _ = stream.write_all(&out).await;
            let _ = stream.shutdown().await;
        }
    });

    temp_env::with_var(
        "XDG_RUNTIME_DIR",
        Some(runtime_dir.to_str().expect("utf8 path")),
        || {
            rt.block_on(async {
                let p = MonitoringServiceProvider::new(MonitoringServiceConfig {
                    endpoint: "e".into(),
                    service_name: "n".into(),
                    auth_token: None,
                    batch_size: 1,
                    flush_interval: std::time::Duration::from_millis(1),
                })
                .expect("new with local monitoring.sock");

                assert_eq!(
                    p.provider_health().await.expect("health"),
                    HealthStatus::Healthy
                );

                p.record_event(MonitoringEvent::Custom {
                    event_type: "t".into(),
                    data: serde_json::json!({}),
                    timestamp: Utc::now(),
                })
                .await
                .expect("record while socket up");

                std::fs::remove_file(&sock_path).ok();
                server.abort();

                p.record_event(MonitoringEvent::Custom {
                    event_type: "t2".into(),
                    data: serde_json::json!({}),
                    timestamp: Utc::now(),
                })
                .await
                .expect("record after socket removed");

                assert_eq!(
                    p.provider_health().await.expect("health after fail"),
                    HealthStatus::Degraded
                );
            });
        },
    );
}

#[tokio::test]
async fn monitoring_service_provider_trait_methods() {
    let p = MonitoringServiceProvider::new(MonitoringServiceConfig {
        endpoint: "e".into(),
        service_name: "n".into(),
        auth_token: Some("t".into()),
        batch_size: 2,
        flush_interval: std::time::Duration::from_secs(2),
    })
    .expect("new");
    assert_eq!(
        p.provider_name(),
        universal_constants::capabilities::SERVICE_MESH_CAPABILITY
    );
    assert_eq!(p.provider_version(), "1.0.0");
    p.record_event(MonitoringEvent::PrimalDiscovered {
        primal_id: "i".into(),
        primal_type: "t".into(),
        endpoint: "ep".into(),
        timestamp: Utc::now(),
    })
    .await
    .expect("should succeed");
    p.record_metric(base_metric())
        .await
        .expect("should succeed");
    p.record_health("c", HealthStatus::Degraded)
        .await
        .expect("should succeed");
    p.record_performance(
        "c",
        PerformanceMetrics {
            cpu_usage: None,
            memory_usage: None,
            network_usage: None,
            response_time: None,
            throughput: None,
            error_rate: None,
            queue_length: None,
            active_connections: None,
            custom_metrics: HashMap::new(),
        },
    )
    .await
    .expect("should succeed");
    assert!(matches!(
        p.provider_health().await.expect("should succeed"),
        HealthStatus::Unknown
    ));
    let caps = p.provider_capabilities().await.expect("should succeed");
    assert!(
        caps.iter()
            .any(|c| matches!(c, MonitoringCapability::Events))
    );
}

#[test]
fn fallback_logger_debug_branches_cover_metrics_health_performance() {
    let fb = FallbackLogger::new(FallbackConfig {
        log_level: "debug".into(),
        include_metrics: true,
        include_health: true,
        include_performance: true,
    });
    let ts = Utc::now();
    fb.log_metric(&base_metric());
    fb.log_health("c", &HealthStatus::Healthy);
    fb.log_performance(
        "p",
        &PerformanceMetrics {
            cpu_usage: None,
            memory_usage: None,
            network_usage: None,
            response_time: None,
            throughput: None,
            error_rate: None,
            queue_length: None,
            active_connections: None,
            custom_metrics: HashMap::new(),
        },
    );
    fb.log_event(&MonitoringEvent::Custom {
        event_type: "t".into(),
        data: serde_json::json!({}),
        timestamp: ts,
    });
}

#[test]
fn fallback_logger_task_completed_success_and_service_stopped_info() {
    let fb = FallbackLogger::new(FallbackConfig {
        log_level: "info".into(),
        ..FallbackConfig::default()
    });
    let ts = Utc::now();
    fb.log_event(&MonitoringEvent::TaskCompleted {
        task_id: "ok".into(),
        execution_time: std::time::Duration::from_millis(1),
        success: true,
        timestamp: ts,
    });
    fb.log_event(&MonitoringEvent::ServiceStopped {
        service: "svc".into(),
        timestamp: ts,
    });
}

#[test]
fn fallback_logger_warn_health_healthy_skips_warn_branch() {
    let fb = FallbackLogger::new(FallbackConfig {
        log_level: "warn".into(),
        include_health: true,
        ..FallbackConfig::default()
    });
    fb.log_health("c", &HealthStatus::Healthy);
}

#[test]
fn fallback_logger_warn_health_unhealthy_triggers_warn() {
    let fb = FallbackLogger::new(FallbackConfig {
        log_level: "warn".into(),
        include_health: true,
        ..FallbackConfig::default()
    });
    fb.log_health("c", &HealthStatus::Unhealthy);
}

#[test]
fn fallback_logger_performance_empty_parts_join() {
    let fb = FallbackLogger::new(FallbackConfig {
        log_level: "info".into(),
        include_performance: true,
        ..FallbackConfig::default()
    });
    fb.log_performance(
        "c",
        &PerformanceMetrics {
            cpu_usage: None,
            memory_usage: None,
            network_usage: None,
            response_time: None,
            throughput: None,
            error_rate: None,
            queue_length: None,
            active_connections: None,
            custom_metrics: HashMap::new(),
        },
    );
}

#[test]
fn fallback_logger_branches() {
    let fb = FallbackLogger::new(FallbackConfig {
        log_level: "info".into(),
        ..FallbackConfig::default()
    });
    let ts = Utc::now();
    fb.log_event(&MonitoringEvent::ServiceStarted {
        service: "s".into(),
        version: "v".into(),
        timestamp: ts,
    });
    fb.log_event(&MonitoringEvent::TaskCompleted {
        task_id: "1".into(),
        execution_time: std::time::Duration::ZERO,
        success: false,
        timestamp: ts,
    });
    fb.log_event(&MonitoringEvent::ErrorOccurred {
        error_type: "E".into(),
        error_message: "m".into(),
        component: "c".into(),
        timestamp: ts,
    });
    fb.log_metric(&base_metric());

    let hf = FallbackLogger::new(FallbackConfig {
        include_health: true,
        log_level: "warn".into(),
        ..FallbackConfig::default()
    });
    hf.log_health("x", &HealthStatus::Degraded);

    let pf = FallbackLogger::new(FallbackConfig {
        include_performance: true,
        log_level: "info".into(),
        ..FallbackConfig::default()
    });
    pf.log_performance(
        "comp",
        &PerformanceMetrics {
            cpu_usage: Some(0.1),
            memory_usage: Some(0.2),
            network_usage: None,
            response_time: Some(std::time::Duration::from_nanos(1)),
            throughput: None,
            error_rate: None,
            queue_length: None,
            active_connections: None,
            custom_metrics: HashMap::new(),
        },
    );

    let silent = FallbackLogger::new(FallbackConfig {
        log_level: "off".into(),
        include_metrics: true,
        include_health: true,
        include_performance: true,
    });
    silent.log_event(&MonitoringEvent::FederationJoined {
        federation_id: "f".into(),
        node_count: 0,
        timestamp: ts,
    });
}

#[tokio::test]
async fn convenience_record_helpers() {
    let svc = MonitoringService::new(MonitoringConfig::default());
    svc.record_service_started("a", "1")
        .await
        .expect("should succeed");
    svc.record_task_completed("t", std::time::Duration::ZERO, true)
        .await
        .expect("should succeed");
    svc.record_error("T", "msg", "comp")
        .await
        .expect("should succeed");
    svc.record_counter("c", 3, HashMap::new())
        .await
        .expect("should succeed");
    svc.record_gauge(
        "g",
        1.5,
        std::iter::once(("l".into(), "v".into())).collect(),
    )
    .await
    .expect("should succeed");
}

#[test]
fn time_frame_and_capability_serde() {
    let tf = TimeFrame::LastHour;
    let tfb: TimeFrame = serde_json::from_value(serde_json::to_value(&tf).expect("should succeed"))
        .expect("should succeed");
    assert!(matches!(tfb, TimeFrame::LastHour));
    let cap = MonitoringCapability::Custom("x".into());
    let json = serde_json::to_string(&cap).expect("should succeed");
    let capb: MonitoringCapability = serde_json::from_str(&json).expect("should succeed");
    assert!(matches!(capb, MonitoringCapability::Custom(s) if s == "x"));
}

#[tokio::test]
async fn monitoring_provider_impl_test_ok_and_failing_query_methods() {
    let ok = MonitoringProviderImpl::TestOk(TestOkProvider("q"));
    assert!(ok.query_health("c").await.expect("qh").is_none());
    assert!(
        ok.query_metrics("c", TimeFrame::LastHour)
            .await
            .expect("qm")
            .is_empty()
    );

    let bad = MonitoringProviderImpl::TestFailing(TestFailingProvider);
    assert!(bad.query_health("c").await.expect("qh").is_none());
    assert!(
        bad.query_metrics("c", TimeFrame::LastHour)
            .await
            .expect("qm")
            .is_empty()
    );
}

#[tokio::test]
async fn monitoring_provider_default_query_health_and_metrics() {
    let p = TestOkProvider("default-queries");
    assert!(
        p.query_health("any")
            .await
            .expect("should succeed")
            .is_none()
    );
    let m = p
        .query_metrics("any", TimeFrame::LastWeek)
        .await
        .expect("should succeed");
    assert!(m.is_empty());
}

#[test]
fn monitoring_status_and_provider_status_serde() {
    let st = MonitoringStatus {
        enabled: true,
        provider_count: 1,
        providers: vec![ProviderStatus {
            name: "n".into(),
            version: "1".into(),
            health: HealthStatus::Healthy,
            capabilities: vec![MonitoringCapability::Events],
        }],
        fallback_active: false,
    };
    let v = serde_json::to_value(&st).expect("should succeed");
    let back: MonitoringStatus = serde_json::from_value(v).expect("should succeed");
    assert!(back.enabled);
    assert_eq!(back.providers.len(), 1);
}

#[test]
fn time_frame_custom_serde_roundtrip() {
    let from = Utc::now();
    let to = from + chrono::Duration::hours(1);
    let tf = TimeFrame::Custom { from, to };
    let v = serde_json::to_value(&tf).expect("should succeed");
    let back: TimeFrame = serde_json::from_value(v).expect("should succeed");
    assert!(matches!(back, TimeFrame::Custom { .. }));
}

#[test]
fn performance_metrics_serde_preserves_optional_fields() {
    let m = PerformanceMetrics {
        cpu_usage: Some(0.25),
        memory_usage: Some(0.5),
        network_usage: Some(0.1),
        response_time: Some(std::time::Duration::from_millis(30)),
        throughput: Some(100.0),
        error_rate: Some(0.01),
        queue_length: Some(3),
        active_connections: Some(10),
        custom_metrics: std::iter::once(("k".into(), 1.0)).collect(),
    };
    let json = serde_json::to_string(&m).expect("should succeed");
    let back: PerformanceMetrics = serde_json::from_str(&json).expect("should succeed");
    assert_eq!(back.cpu_usage, m.cpu_usage);
    assert_eq!(back.custom_metrics.get("k").copied(), Some(1.0));
}
