// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

use super::*;
use crate::HealthStatus;
use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;

/// Minimal provider that always succeeds.
struct OkProvider(&'static str);

#[async_trait]
impl MonitoringProvider for OkProvider {
    fn provider_name(&self) -> &'static str {
        self.0
    }

    fn provider_version(&self) -> &'static str {
        "test"
    }

    async fn record_event(&self, _: MonitoringEvent) -> crate::Result<()> {
        Ok(())
    }

    async fn record_metric(&self, _: Metric) -> crate::Result<()> {
        Ok(())
    }

    async fn record_health(&self, _: &str, _: HealthStatus) -> crate::Result<()> {
        Ok(())
    }

    async fn record_performance(&self, _: &str, _: PerformanceMetrics) -> crate::Result<()> {
        Ok(())
    }

    async fn provider_health(&self) -> crate::Result<HealthStatus> {
        Ok(HealthStatus::Healthy)
    }

    async fn provider_capabilities(&self) -> crate::Result<Vec<MonitoringCapability>> {
        Ok(vec![MonitoringCapability::Metrics])
    }
}

/// Provider that errors on every record call (exercises best-effort paths).
struct FailingProvider;

#[async_trait]
impl MonitoringProvider for FailingProvider {
    fn provider_name(&self) -> &'static str {
        "failing"
    }

    fn provider_version(&self) -> &'static str {
        "0"
    }

    async fn record_event(&self, _: MonitoringEvent) -> crate::Result<()> {
        Err(crate::Error::Monitoring("e".into()))
    }

    async fn record_metric(&self, _: Metric) -> crate::Result<()> {
        Err(crate::Error::Monitoring("e".into()))
    }

    async fn record_health(&self, _: &str, _: HealthStatus) -> crate::Result<()> {
        Err(crate::Error::Monitoring("e".into()))
    }

    async fn record_performance(&self, _: &str, _: PerformanceMetrics) -> crate::Result<()> {
        Err(crate::Error::Monitoring("e".into()))
    }

    async fn provider_health(&self) -> crate::Result<HealthStatus> {
        Err(crate::Error::Monitoring("e".into()))
    }

    async fn provider_capabilities(&self) -> crate::Result<Vec<MonitoringCapability>> {
        Err(crate::Error::Monitoring("e".into()))
    }
}

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
    assert!(mc.songbird_config.is_none());
    assert!(mc.provider_configs.is_empty());
    let fc = FallbackConfig::default();
    assert_eq!(fc.log_level, "info");
    assert!(fc.include_metrics && fc.include_health && fc.include_performance);
}

#[test]
fn monitoring_config_roundtrip_serde_without_songbird() {
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
    let v = serde_json::to_value(&ev).unwrap();
    let back: MonitoringEvent = serde_json::from_value(v).unwrap();
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
    let mv = serde_json::to_value(&m).unwrap();
    let mb: Metric = serde_json::from_value(mv).unwrap();
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
        songbird_config: None,
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
async fn initialize_with_songbird_config_adds_provider() {
    let cfg = MonitoringConfig {
        require_provider: false,
        songbird_config: Some(SongbirdConfig {
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
    assert!(names.iter().any(|n| n == "songbird"));
}

#[tokio::test]
async fn initialize_unknown_provider_is_skipped_with_warning_path() {
    let cfg = MonitoringConfig {
        require_provider: false,
        provider_configs: std::iter::once(("not-songbird".into(), serde_json::json!({}))).collect(),
        ..MonitoringConfig::default()
    };
    let svc = MonitoringService::new(cfg);
    svc.initialize().expect("init");
    assert!(svc.get_providers().is_empty());
}

#[tokio::test]
async fn initialize_provider_configs_songbird_branch() {
    let cfg = MonitoringConfig {
        songbird_config: None,
        provider_configs: std::iter::once((
            "songbird".into(),
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
    assert!(svc.get_providers().iter().any(|n| n == "songbird"));
}

#[tokio::test]
async fn songbird_provider_new_succeeds_and_unknown_provider_config_skipped() {
    let res = SongbirdProvider::new(SongbirdConfig {
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
        songbird_config: None,
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
    .unwrap();
    svc.record_metric(base_metric()).await.unwrap();
    svc.record_health("c", HealthStatus::Healthy).await.unwrap();
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
    .unwrap();
    let st = svc.get_status().await;
    assert!(st.fallback_active);
    assert_eq!(st.provider_count, 0);
}

#[tokio::test]
async fn record_with_failing_provider_still_ok() {
    let svc = MonitoringService::new(MonitoringConfig::default());
    svc.add_provider(Arc::new(FailingProvider) as Arc<dyn MonitoringProvider>);
    svc.record_event(MonitoringEvent::Custom {
        event_type: "t".into(),
        data: serde_json::json!({}),
        timestamp: Utc::now(),
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn add_remove_get_providers_and_status() {
    let svc = MonitoringService::new(MonitoringConfig::default());
    svc.add_provider(Arc::new(OkProvider("p1")) as Arc<dyn MonitoringProvider>);
    svc.add_provider(Arc::new(OkProvider("p2")) as Arc<dyn MonitoringProvider>);
    assert_eq!(svc.get_providers().len(), 2);
    svc.remove_provider("p1");
    assert_eq!(svc.get_providers(), vec!["p2"]);
    let st = svc.get_status().await;
    assert_eq!(st.provider_count, 1);
    assert!(!st.fallback_active);
    assert_eq!(st.providers[0].name, "p2");
}

#[tokio::test]
async fn songbird_provider_trait_methods() {
    let p = SongbirdProvider::new(SongbirdConfig {
        endpoint: "e".into(),
        service_name: "n".into(),
        auth_token: Some("t".into()),
        batch_size: 2,
        flush_interval: std::time::Duration::from_secs(2),
    })
    .expect("new");
    assert_eq!(p.provider_name(), "songbird");
    assert_eq!(p.provider_version(), "1.0.0");
    p.record_event(MonitoringEvent::PrimalDiscovered {
        primal_id: "i".into(),
        primal_type: "t".into(),
        endpoint: "ep".into(),
        timestamp: Utc::now(),
    })
    .await
    .unwrap();
    p.record_metric(base_metric()).await.unwrap();
    p.record_health("c", HealthStatus::Degraded).await.unwrap();
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
    .unwrap();
    assert!(matches!(
        p.provider_health().await.unwrap(),
        HealthStatus::Unknown
    ));
    let caps = p.provider_capabilities().await.unwrap();
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
        ..FallbackConfig::default()
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
    svc.record_service_started("a", "1").await.unwrap();
    svc.record_task_completed("t", std::time::Duration::ZERO, true)
        .await
        .unwrap();
    svc.record_error("T", "msg", "comp").await.unwrap();
    svc.record_counter("c", 3, HashMap::new()).await.unwrap();
    svc.record_gauge(
        "g",
        1.5,
        std::iter::once(("l".into(), "v".into())).collect(),
    )
    .await
    .unwrap();
}

#[test]
fn time_frame_and_capability_serde() {
    let tf = TimeFrame::LastHour;
    let tfb: TimeFrame = serde_json::from_value(serde_json::to_value(&tf).unwrap()).unwrap();
    assert!(matches!(tfb, TimeFrame::LastHour));
    let cap = MonitoringCapability::Custom("x".into());
    let json = serde_json::to_string(&cap).unwrap();
    let capb: MonitoringCapability = serde_json::from_str(&json).unwrap();
    assert!(matches!(capb, MonitoringCapability::Custom(s) if s == "x"));
}

#[tokio::test]
async fn monitoring_provider_default_query_health_and_metrics() {
    let p = OkProvider("default-queries");
    assert!(p.query_health("any").await.unwrap().is_none());
    let m = p.query_metrics("any", TimeFrame::LastWeek).await.unwrap();
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
    let v = serde_json::to_value(&st).unwrap();
    let back: MonitoringStatus = serde_json::from_value(v).unwrap();
    assert!(back.enabled);
    assert_eq!(back.providers.len(), 1);
}

#[test]
fn time_frame_custom_serde_roundtrip() {
    let from = Utc::now();
    let to = from + chrono::Duration::hours(1);
    let tf = TimeFrame::Custom { from, to };
    let v = serde_json::to_value(&tf).unwrap();
    let back: TimeFrame = serde_json::from_value(v).unwrap();
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
    let json = serde_json::to_string(&m).unwrap();
    let back: PerformanceMetrics = serde_json::from_str(&json).unwrap();
    assert_eq!(back.cpu_usage, m.cpu_usage);
    assert_eq!(back.custom_metrics.get("k").copied(), Some(1.0));
}
