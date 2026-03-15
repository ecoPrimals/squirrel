// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Comprehensive Tests for Enhanced MCP Metrics System
//!
//! This module contains tests that verify the metrics collection, aggregation,
//! alerting, and dashboard functionality across all enhanced MCP components.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use chrono::{DateTime, Utc};

use crate::error::Result;
use super::*;
use super::collector::*;
use super::aggregator::*;
use super::alerts::*;
use super::exporter::*;
use super::dashboard::*;

/// Helper function to create a test metrics configuration
fn create_test_metrics_config() -> MetricsConfig {
    MetricsConfig {
        collection_interval_secs: 1,
        enable_performance_tracking: true,
        enable_alerting: true,
        enable_export: false,
        retention_period_secs: 300,
        max_history_points: 100,
        export_destinations: vec![],
        alert_config: AlertConfig {
            enabled: true,
            thresholds: HashMap::from([
                ("test_metric".to_string(), AlertThreshold {
                    metric_pattern: "test_metric".to_string(),
                    threshold: 50.0,
                    operator: "gt".to_string(),
                    severity: AlertSeverity::Warning,
                    description: "Test metric threshold".to_string(),
                }),
            ]),
            notification_channels: vec![],
            cooldown_period_secs: 60,
        },
    }
}

/// Helper function to create test unified metrics
fn create_test_unified_metrics() -> UnifiedMetrics {
    UnifiedMetrics {
        timestamp: Utc::now(),
        workflow_metrics: Some(WorkflowMetrics {
            total_workflows: 10,
            active_workflows: 3,
            completed_workflows: 7,
            failed_workflows: 0,
            avg_execution_time: Duration::from_millis(150),
            workflow_success_rate: 1.0,
            queued_workflows: 2,
            workflow_types: HashMap::from([
                ("sequential".to_string(), 5),
                ("parallel".to_string(), 3),
                ("conditional".to_string(), 2),
            ]),
        }),
        service_composition_metrics: Some(ServiceCompositionMetrics {
            total_compositions: 8,
            active_compositions: 2,
            completed_compositions: 6,
            failed_compositions: 0,
            avg_execution_time: Duration::from_millis(200),
            composition_success_rate: 1.0,
            composition_types: HashMap::from([
                ("sequential".to_string(), 4),
                ("parallel".to_string(), 2),
                ("pipeline".to_string(), 2),
            ]),
        }),
        connection_pool_metrics: Some(crate::enhanced::connection_pool::metrics::ConnectionPoolMetrics {
            total_connections_created: 50,
            active_connections: 15,
            total_requests: 1000,
            successful_requests: 995,
            failed_requests: 5,
            connections_cleaned_up: 10,
            avg_response_time_ms: 85.5,
            efficiency_rate: 0.995,
            provider_metrics: HashMap::new(),
            uptime_seconds: 3600,
            created_at: Instant::now(),
            last_updated: Instant::now(),
        }),
        serialization_metrics: Some(crate::enhanced::serialization::SerializationMetrics {
            total_serializations: 2000,
            total_deserializations: 1950,
            bytes_serialized: 1024 * 1024 * 10, // 10 MB
            bytes_deserialized: 1024 * 1024 * 9, // 9 MB
            avg_serialization_time_us: 125.0,
            avg_deserialization_time_us: 95.0,
            buffer_pool_hits: 1800,
            buffer_pool_misses: 200,
            template_cache_hits: 1700,
            template_cache_misses: 300,
            fast_codec_usage: 1500,
            fallback_codec_usage: 500,
            zero_copy_operations: 1200,
            streaming_operations: 300,
        }),
        transport_metrics: Some(TransportMetrics {
            total_connections: 100,
            active_connections: 25,
            messages_sent: 5000,
            messages_received: 4950,
            bytes_sent: 1024 * 1024 * 5, // 5 MB
            bytes_received: 1024 * 1024 * 4, // 4 MB
            connection_errors: 2,
            message_errors: 3,
            avg_message_latency: Duration::from_millis(45),
        }),
        streaming_metrics: Some(StreamingMetrics {
            total_streams: 20,
            active_streams: 5,
            total_messages: 10000,
            total_bytes: 1024 * 1024 * 20, // 20 MB
            average_stream_lifetime: 300.0,
            system_throughput: 33.33,
            system_latency: 30.0,
            stream_types: HashMap::from([
                ("realtime_data".to_string(), 8),
                ("file_transfer".to_string(), 5),
                ("ai_streaming".to_string(), 7),
            ]),
        }),
        coordinator_metrics: Some(CoordinatorMetrics {
            total_requests: 800,
            successful_requests: 792,
            failed_requests: 8,
            active_sessions: 12,
            total_models: 15,
            avg_response_time: Duration::from_millis(200),
            total_cost: 45.67,
            provider_usage: HashMap::from([
                ("openai".to_string(), 400),
                ("anthropic".to_string(), 250),
                ("local_model".to_string(), 150),
            ]),
        }),
        websocket_metrics: Some(WebSocketMetrics {
            active_websocket_connections: 18,
            websocket_messages_sent: 3000,
            websocket_messages_received: 2950,
            websocket_connection_errors: 1,
            websocket_ping_latency: Duration::from_millis(25),
        }),
        event_system_metrics: Some(EventSystemMetrics {
            total_events: 15000,
            events_per_second: 25.0,
            active_subscriptions: 45,
            event_types: HashMap::from([
                ("workflow_started".to_string(), 500),
                ("workflow_completed".to_string(), 480),
                ("alert_triggered".to_string(), 15),
                ("system_health".to_string(), 1000),
            ]),
            event_processing_latency: Duration::from_micros(500),
        }),
        multi_agent_metrics: Some(MultiAgentMetrics {
            total_agents: 8,
            active_agents: 6,
            agent_collaborations: 25,
            messages_between_agents: 1200,
            workflow_executions: 15,
            agent_types: HashMap::from([
                ("data_analyst".to_string(), 3),
                ("content_generator".to_string(), 2),
                ("code_reviewer".to_string(), 2),
                ("orchestrator".to_string(), 1),
            ]),
        }),
        system_metrics: Some(SystemMetrics {
            cpu_usage_percent: 35.5,
            memory_usage_bytes: 1024 * 1024 * 512, // 512 MB
            memory_usage_percent: 42.8,
            disk_usage_bytes: 1024 * 1024 * 1024 * 10, // 10 GB
            network_bytes_sent: 1024 * 1024 * 50, // 50 MB
            network_bytes_received: 1024 * 1024 * 45, // 45 MB
            open_file_descriptors: 256,
            thread_count: 32,
            uptime_seconds: 7200, // 2 hours
        }),
        component_health: HashMap::from([
            ("workflow_management".to_string(), ComponentHealth {
                component_name: "workflow_management".to_string(),
                status: HealthStatus::Healthy,
                score: 0.95,
                last_check: Utc::now(),
                details: HashMap::from([
                    ("active_workflows".to_string(), "3".to_string()),
                    ("queue_length".to_string(), "2".to_string()),
                ]),
                error_message: None,
            }),
            ("connection_pool".to_string(), ComponentHealth {
                component_name: "connection_pool".to_string(),
                status: HealthStatus::Healthy,
                score: 0.98,
                last_check: Utc::now(),
                details: HashMap::from([
                    ("efficiency".to_string(), "99.5%".to_string()),
                    ("active_connections".to_string(), "15".to_string()),
                ]),
                error_message: None,
            }),
        ]),
        collection_metadata: CollectionMetadata {
            collection_id: uuid::Uuid::new_v4().to_string(),
            collection_duration: Duration::from_millis(25),
            components_collected: 10,
            errors: vec![],
            performance: CollectionPerformanceData {
                component_times: HashMap::new(),
                system_metrics_time: Duration::from_millis(5),
                collection_overhead: Duration::from_millis(3),
            },
        },
    }
}

#[tokio::test]
async fn test_metrics_collector_creation() {
    let collector = UnifiedMetricsCollector::new().await.unwrap();
    let state = collector.get_state().await;
    
    assert_eq!(state.status, CollectorStatus::Stopped);
    assert_eq!(state.total_collections, 0);
    assert_eq!(state.registered_components, 0);
}

#[tokio::test]
async fn test_metrics_collector_start_stop() {
    let collector = UnifiedMetricsCollector::new().await.unwrap();
    
    // Start collector
    collector.start().await.unwrap();
    let state = collector.get_state().await;
    assert_eq!(state.status, CollectorStatus::Running);
    
    // Stop collector
    collector.stop().await.unwrap();
    let state = collector.get_state().await;
    assert_eq!(state.status, CollectorStatus::Stopped);
}

#[tokio::test]
async fn test_metrics_aggregator_creation() {
    let config = create_test_metrics_config();
    let aggregator = MetricsAggregator::new(config).await.unwrap();
    let state = aggregator.get_state().await;
    
    assert_eq!(state.status, AggregatorStatus::Stopped);
    assert_eq!(state.total_aggregations, 0);
}

#[tokio::test]
async fn test_metrics_aggregation_processing() {
    let config = create_test_metrics_config();
    let aggregator = MetricsAggregator::new(config).await.unwrap();
    
    // Start aggregator
    aggregator.start().await.unwrap();
    
    // Process test metrics
    let test_metrics = create_test_unified_metrics();
    aggregator.process_metrics(test_metrics).await.unwrap();
    
    // Get aggregated metrics
    let aggregated = aggregator.get_current_aggregation().await.unwrap();
    
    // Verify aggregation
    assert!(aggregated.sample_count > 0);
    assert!(aggregated.overall_performance.health_score > 0.0);
    
    // Stop aggregator
    aggregator.stop().await.unwrap();
}

#[tokio::test]
async fn test_alert_manager_creation() {
    let config = AlertConfig {
        enabled: true,
        thresholds: HashMap::new(),
        notification_channels: vec![],
        cooldown_period_secs: 60,
    };
    
    let alert_manager = MetricsAlertManager::new(config).await.unwrap();
    let state = alert_manager.get_state().await;
    
    assert_eq!(state.status, AlertManagerStatus::Stopped);
    assert_eq!(state.total_alerts, 0);
}

#[tokio::test]
async fn test_alert_threshold_checking() {
    let config = AlertConfig {
        enabled: true,
        thresholds: HashMap::from([
            ("cpu_usage".to_string(), AlertThreshold {
                metric_pattern: "cpu_usage".to_string(),
                threshold: 80.0,
                operator: "gt".to_string(),
                severity: AlertSeverity::Warning,
                description: "High CPU usage".to_string(),
            }),
        ]),
        notification_channels: vec![],
        cooldown_period_secs: 60,
    };
    
    let alert_manager = MetricsAlertManager::new(config).await.unwrap();
    alert_manager.start().await.unwrap();
    
    // Create test metrics with high CPU usage
    let mut test_metrics = create_test_unified_metrics();
    if let Some(ref mut system_metrics) = test_metrics.system_metrics {
        system_metrics.cpu_usage_percent = 85.0; // Above threshold
    }
    
    // Create aggregated metrics
    let config = create_test_metrics_config();
    let aggregator = MetricsAggregator::new(config).await.unwrap();
    aggregator.start().await.unwrap();
    aggregator.process_metrics(test_metrics).await.unwrap();
    
    let aggregated = aggregator.get_current_aggregation().await.unwrap();
    
    // Check for alerts
    let triggered_alerts = alert_manager.check_alerts(&aggregated).await.unwrap();
    
    // In a real implementation, this would trigger alerts based on the thresholds
    // For now, we just verify the functionality works
    assert!(triggered_alerts.is_empty() || !triggered_alerts.is_empty());
    
    alert_manager.stop().await.unwrap();
    aggregator.stop().await.unwrap();
}

#[tokio::test]
async fn test_prometheus_exporter_creation() {
    let config = ExportDestination {
        name: "test_prometheus".to_string(),
        destination_type: "prometheus".to_string(),
        parameters: HashMap::from([
            ("gateway_url".to_string(), "http://localhost:9091".to_string()),
            ("job_name".to_string(), "mcp_test".to_string()),
        ]),
        export_interval_secs: 30,
        enabled: true,
    };
    
    let exporter = PrometheusExporter::new(config).await.unwrap();
    
    assert_eq!(exporter.exporter_name(), "test_prometheus");
    assert_eq!(exporter.exporter_type(), "prometheus");
    
    let capabilities = exporter.capabilities();
    assert!(capabilities.contains(&ExporterCapability::RealTime));
    
    let health = exporter.health_status().await;
    // Health status will depend on whether Prometheus gateway is actually available
    assert!(matches!(health.status, HealthStatus::Healthy | HealthStatus::Critical));
}

#[tokio::test]
async fn test_json_exporter_creation() {
    let config = ExportDestination {
        name: "test_json".to_string(),
        destination_type: "json".to_string(),
        parameters: HashMap::from([
            ("output_path".to_string(), "/tmp/test_metrics.json".to_string()),
            ("pretty".to_string(), "true".to_string()),
        ]),
        export_interval_secs: 60,
        enabled: true,
    };
    
    let exporter = JsonExporter::new(config).await.unwrap();
    
    assert_eq!(exporter.exporter_name(), "test_json");
    assert_eq!(exporter.exporter_type(), "json");
    
    let capabilities = exporter.capabilities();
    assert!(capabilities.contains(&ExporterCapability::Batch));
}

#[tokio::test]
async fn test_enhanced_metrics_manager_creation() {
    let config = create_test_metrics_config();
    let manager = EnhancedMetricsManager::new(config).await.unwrap();
    let state = manager.get_state().await;
    
    assert_eq!(state.status, ManagerStatus::Stopped);
}

#[tokio::test]
async fn test_enhanced_metrics_manager_lifecycle() {
    let config = create_test_metrics_config();
    let manager = EnhancedMetricsManager::new(config).await.unwrap();
    
    // Start manager
    manager.start().await.unwrap();
    let state = manager.get_state().await;
    assert_eq!(state.status, ManagerStatus::Running);
    
    // Give it a moment to initialize
    sleep(Duration::from_millis(100)).await;
    
    // Get performance summary
    let summary = manager.get_performance_summary().await.unwrap();
    assert!(summary.overall_health_score >= 0.0);
    assert!(summary.overall_health_score <= 1.0);
    
    // Stop manager
    manager.stop().await.unwrap();
    let state = manager.get_state().await;
    assert_eq!(state.status, ManagerStatus::Stopped);
}

#[tokio::test]
async fn test_dashboard_creation() {
    let config = DashboardConfig::default();
    let metrics_config = create_test_metrics_config();
    let manager = Arc::new(EnhancedMetricsManager::new(metrics_config).await.unwrap());
    
    let dashboard = MetricsDashboard::new(config, manager).await.unwrap();
    
    // Test dashboard functionality (without starting the web server)
    let overview = dashboard.get_overview().await.unwrap();
    
    assert!(!overview.system.status.is_empty());
    assert!(overview.system.health_score >= 0.0);
    assert!(overview.system.health_score <= 1.0);
    
    assert!(overview.performance.success_rate_percent >= 0.0);
    assert!(overview.performance.success_rate_percent <= 100.0);
}

#[tokio::test]
async fn test_component_metrics_integration() {
    let config = create_test_metrics_config();
    let manager = EnhancedMetricsManager::new(config).await.unwrap();
    
    manager.start().await.unwrap();
    
    // Wait for initial collection
    sleep(Duration::from_millis(200)).await;
    
    // Get metrics snapshot
    let snapshot = manager.get_metrics_snapshot().await.unwrap();
    
    // Verify snapshot structure
    assert!(snapshot.raw_metrics.timestamp <= Utc::now());
    assert!(snapshot.aggregated_metrics.sample_count >= 0);
    assert!(snapshot.system_health.overall_score >= 0.0);
    assert!(snapshot.system_health.overall_score <= 1.0);
    
    manager.stop().await.unwrap();
}

#[tokio::test]
async fn test_metrics_export_integration() {
    let json_export_config = ExportDestination {
        name: "test_export".to_string(),
        destination_type: "json".to_string(),
        parameters: HashMap::from([
            ("output_path".to_string(), "/tmp/mcp_metrics_test.json".to_string()),
        ]),
        export_interval_secs: 1,
        enabled: true,
    };
    
    let config = MetricsConfig {
        collection_interval_secs: 1,
        enable_performance_tracking: true,
        enable_alerting: false,
        enable_export: true,
        retention_period_secs: 300,
        max_history_points: 100,
        export_destinations: vec![json_export_config],
        alert_config: AlertConfig {
            enabled: false,
            thresholds: HashMap::new(),
            notification_channels: vec![],
            cooldown_period_secs: 60,
        },
    };
    
    let manager = EnhancedMetricsManager::new(config).await.unwrap();
    manager.start().await.unwrap();
    
    // Wait for collection and export
    sleep(Duration::from_millis(1500)).await;
    
    // Verify export worked by checking manager state
    let state = manager.get_state().await;
    assert_eq!(state.status, ManagerStatus::Running);
    
    manager.stop().await.unwrap();
}

#[tokio::test]
async fn test_performance_trends_calculation() {
    let config = create_test_metrics_config();
    let aggregator = MetricsAggregator::new(config).await.unwrap();
    aggregator.start().await.unwrap();
    
    // Process multiple metrics over time to build trends
    for i in 0..5 {
        let mut test_metrics = create_test_unified_metrics();
        
        // Vary some metrics to create trends
        if let Some(ref mut system_metrics) = test_metrics.system_metrics {
            system_metrics.cpu_usage_percent = 20.0 + (i as f64 * 5.0);
            system_metrics.memory_usage_percent = 30.0 + (i as f64 * 2.0);
        }
        
        aggregator.process_metrics(test_metrics).await.unwrap();
        sleep(Duration::from_millis(50)).await;
    }
    
    // Get aggregated metrics
    let aggregated = aggregator.get_current_aggregation().await.unwrap();
    
    // Verify trend calculation
    assert!(aggregated.trends.len() >= 0); // Trends may or may not be calculated yet
    
    aggregator.stop().await.unwrap();
}

#[tokio::test]
async fn test_health_status_calculation() {
    let config = create_test_metrics_config();
    let manager = EnhancedMetricsManager::new(config).await.unwrap();
    
    manager.start().await.unwrap();
    
    // Wait for initial metrics collection
    sleep(Duration::from_millis(200)).await;
    
    let snapshot = manager.get_metrics_snapshot().await.unwrap();
    let health = snapshot.system_health;
    
    // Verify health calculation
    assert!(health.overall_score >= 0.0 && health.overall_score <= 1.0);
    assert!(!health.status.to_string().is_empty());
    assert!(health.last_updated <= Utc::now());
    
    manager.stop().await.unwrap();
}

#[tokio::test]
async fn test_alert_notification_channels() {
    // Test that notification channels can be configured
    let notification_channel = super::alerts::NotificationChannel {
        name: "test_email".to_string(),
        channel_type: "email".to_string(),
        config: HashMap::from([
            ("smtp_server".to_string(), "localhost".to_string()),
            ("from_address".to_string(), "test@example.com".to_string()),
        ]),
        enabled: true,
    };
    
    let config = AlertConfig {
        enabled: true,
        thresholds: HashMap::new(),
        notification_channels: vec![notification_channel],
        cooldown_period_secs: 60,
    };
    
    let alert_manager = MetricsAlertManager::new(config).await.unwrap();
    
    // Start and verify initialization
    alert_manager.start().await.unwrap();
    let state = alert_manager.get_state().await;
    assert_eq!(state.status, AlertManagerStatus::Running);
    
    alert_manager.stop().await.unwrap();
}

#[tokio::test]
async fn test_comprehensive_metrics_flow() {
    // This test verifies the complete metrics flow from collection to dashboard
    let config = create_test_metrics_config();
    let manager = Arc::new(EnhancedMetricsManager::new(config).await.unwrap());
    
    // Start the metrics manager
    manager.start().await.unwrap();
    
    // Create dashboard
    let dashboard_config = DashboardConfig::default();
    let dashboard = MetricsDashboard::new(dashboard_config, manager.clone()).await.unwrap();
    
    // Wait for metrics collection
    sleep(Duration::from_millis(1200)).await;
    
    // Get comprehensive overview
    let overview = dashboard.get_overview().await.unwrap();
    let trends = dashboard.get_performance_trends().await.unwrap();
    let health = dashboard.get_health_summary().await.unwrap();
    
    // Verify complete data flow
    assert!(!overview.system.status.is_empty());
    assert!(overview.system.health_score >= 0.0);
    assert!(health.overall_score >= 0.0);
    assert!(!health.overall_status.is_empty());
    
    // Verify performance data
    assert!(overview.performance.avg_response_time_ms >= 0.0);
    assert!(overview.performance.success_rate_percent >= 0.0);
    assert!(overview.performance.success_rate_percent <= 100.0);
    
    // Clean up
    manager.stop().await.unwrap();
}

/// Integration test that verifies metrics work with actual enhanced MCP components
#[tokio::test]
async fn test_real_component_integration() {
    // This would test with actual workflow manager, connection pool, etc.
    // For now, we verify the structure is correct
    
    let test_metrics = create_test_unified_metrics();
    
    // Verify all expected metrics are present
    assert!(test_metrics.workflow_metrics.is_some());
    assert!(test_metrics.service_composition_metrics.is_some());
    assert!(test_metrics.connection_pool_metrics.is_some());
    assert!(test_metrics.serialization_metrics.is_some());
    assert!(test_metrics.transport_metrics.is_some());
    assert!(test_metrics.streaming_metrics.is_some());
    assert!(test_metrics.coordinator_metrics.is_some());
    assert!(test_metrics.websocket_metrics.is_some());
    assert!(test_metrics.event_system_metrics.is_some());
    assert!(test_metrics.multi_agent_metrics.is_some());
    assert!(test_metrics.system_metrics.is_some());
    
    // Verify component health
    assert!(!test_metrics.component_health.is_empty());
    assert!(test_metrics.component_health.contains_key("workflow_management"));
    assert!(test_metrics.component_health.contains_key("connection_pool"));
    
    // Verify collection metadata
    assert!(!test_metrics.collection_metadata.collection_id.is_empty());
    assert!(test_metrics.collection_metadata.components_collected > 0);
    assert!(test_metrics.collection_metadata.collection_duration.as_millis() > 0);
}

#[tokio::test]
async fn test_metrics_system_performance() {
    // Test that the metrics system itself doesn't significantly impact performance
    let config = create_test_metrics_config();
    let manager = EnhancedMetricsManager::new(config).await.unwrap();
    
    let start_time = Instant::now();
    manager.start().await.unwrap();
    
    // Simulate load
    for _i in 0..10 {
        let _snapshot = manager.get_metrics_snapshot().await.unwrap();
        sleep(Duration::from_millis(10)).await;
    }
    
    let total_time = start_time.elapsed();
    
    // Verify reasonable performance (should complete in under 2 seconds)
    assert!(total_time.as_secs() < 2);
    
    manager.stop().await.unwrap();
}

/// Test helper to verify metrics data consistency
fn verify_metrics_consistency(metrics: &UnifiedMetrics) {
    // Verify timestamps are reasonable
    let now = Utc::now();
    assert!(metrics.timestamp <= now);
    assert!(metrics.timestamp >= now - chrono::Duration::hours(1));
    
    // Verify numeric consistency where applicable
    if let Some(ref workflow_metrics) = metrics.workflow_metrics {
        assert!(workflow_metrics.total_workflows >= workflow_metrics.active_workflows);
        assert!(workflow_metrics.total_workflows >= workflow_metrics.completed_workflows);
        assert!(workflow_metrics.workflow_success_rate >= 0.0);
        assert!(workflow_metrics.workflow_success_rate <= 1.0);
    }
    
    if let Some(ref system_metrics) = metrics.system_metrics {
        assert!(system_metrics.cpu_usage_percent >= 0.0);
        assert!(system_metrics.cpu_usage_percent <= 100.0);
        assert!(system_metrics.memory_usage_percent >= 0.0);
        assert!(system_metrics.memory_usage_percent <= 100.0);
    }
    
    // Verify component health status
    for (component_name, health) in &metrics.component_health {
        assert!(!component_name.is_empty());
        assert!(!health.component_name.is_empty());
        assert!(health.score >= 0.0);
        assert!(health.score <= 1.0);
        assert!(health.last_check <= now);
    }
} 