//! Chaos Engineering Tests
//!
//! Comprehensive test suite for validating chaos engineering capabilities including
//! fault injection, recovery orchestration, and system resilience testing.

use squirrel::chaos::{
    recovery::{RecoveryStrategy, ValidationStep},
    ChaosEngineer, ExperimentConfig, FaultType, MonitoringConfig, NetworkErrorType, RecoveryConfig,
    ResourceType, SuccessCriterion,
};
use std::time::Duration;
use tokio::test;

/// Test basic chaos engineering experiment execution
#[tokio::test]
async fn test_basic_experiment_execution() {
    let chaos = ChaosEngineer::new();

    let experiment = ExperimentConfig::new()
        .with_name(
            "Basic Network Fault Test".to_string(),
            "Test system response to network failures".to_string(),
        )
        .with_fault(FaultType::NetworkFailure {
            rate: 0.1,
            latency_ms: Some(500),
            error_type: NetworkErrorType::Timeout,
        })
        .with_duration(Duration::from_secs(5))
        .with_success_criterion(SuccessCriterion::SystemResponsive {
            max_response_time_ms: 2000,
            success_rate_threshold: 0.9,
        });

    let result = chaos.run_experiment(experiment).await;
    assert!(
        result.is_ok(),
        "Basic experiment should complete successfully"
    );

    let experiment_result = result.unwrap();
    assert!(!experiment_result.config.id.is_empty());
    assert!(experiment_result.duration >= Duration::from_secs(5));
}

/// Test memory pressure fault injection
#[tokio::test]
async fn test_memory_pressure_fault() {
    let chaos = ChaosEngineer::new();

    let experiment = ExperimentConfig::new()
        .with_name(
            "Memory Pressure Test".to_string(),
            "Test system behavior under memory pressure".to_string(),
        )
        .with_fault(FaultType::MemoryPressure {
            allocation_mb: 50, // Allocate 50MB
            duration: Duration::from_secs(3),
            gradual: true,
        })
        .with_duration(Duration::from_secs(5))
        .with_success_criterion(SuccessCriterion::MetricBounds {
            metric_name: "memory_usage_percent".to_string(),
            min_value: 0.0,
            max_value: 95.0,
        });

    let result = chaos.run_experiment(experiment).await;
    assert!(result.is_ok(), "Memory pressure experiment should complete");

    let experiment_result = result.unwrap();
    assert!(experiment_result.summary.health_score >= 0.0);
    assert!(experiment_result.summary.health_score <= 1.0);
}

/// Test CPU starvation fault injection
#[tokio::test]
async fn test_cpu_starvation_fault() {
    let chaos = ChaosEngineer::new();

    let experiment = ExperimentConfig::new()
        .with_name(
            "CPU Starvation Test".to_string(),
            "Test system behavior under CPU pressure".to_string(),
        )
        .with_fault(FaultType::CpuStarvation {
            cpu_percentage: 50.0,
            duration: Duration::from_secs(3),
            threads: 2,
        })
        .with_duration(Duration::from_secs(5))
        .with_success_criterion(SuccessCriterion::SystemResponsive {
            max_response_time_ms: 5000,
            success_rate_threshold: 0.8,
        });

    let result = chaos.run_experiment(experiment).await;
    assert!(result.is_ok(), "CPU starvation experiment should complete");
}

/// Test multiple fault types in a single experiment
#[tokio::test]
async fn test_multiple_fault_injection() {
    let chaos = ChaosEngineer::new();

    let experiment = ExperimentConfig::new()
        .with_name(
            "Multi-Fault Test".to_string(),
            "Test system resilience with multiple simultaneous faults".to_string(),
        )
        .with_fault(FaultType::NetworkFailure {
            rate: 0.05,
            latency_ms: Some(200),
            error_type: NetworkErrorType::ConnectionRefused,
        })
        .with_fault(FaultType::MemoryPressure {
            allocation_mb: 25,
            duration: Duration::from_secs(4),
            gradual: false,
        })
        .with_fault(FaultType::ServiceUnavailable {
            service_name: "test-service".to_string(),
            duration: Duration::from_secs(2),
            error_response: Some("Service temporarily unavailable".to_string()),
        })
        .with_duration(Duration::from_secs(6))
        .with_success_criterion(SuccessCriterion::ErrorRate {
            max_error_rate: 0.2,
            window_duration: Duration::from_secs(30),
        });

    let result = chaos.run_experiment(experiment).await;
    assert!(result.is_ok(), "Multi-fault experiment should complete");

    let experiment_result = result.unwrap();
    assert_eq!(experiment_result.config.faults.len(), 3);
    assert!(experiment_result.summary.total_requests >= 0);
}

/// Test recovery orchestration
#[tokio::test]
async fn test_recovery_orchestration() {
    let chaos = ChaosEngineer::new();

    let recovery_config = RecoveryConfig {
        auto_recovery: true,
        recovery_timeout: Duration::from_secs(30),
        strategies: vec![
            RecoveryStrategy::ClearCaches,
            RecoveryStrategy::ResetConnections,
            RecoveryStrategy::MemoryCleanup,
        ],
        validation_steps: vec![
            ValidationStep::MemoryUsage {
                max_usage_percent: 80.0,
            },
            ValidationStep::CpuUsage {
                max_usage_percent: 70.0,
            },
            ValidationStep::HealthCheck {
                endpoint: "http://localhost:8080/health".to_string(),
                expected_status: 200,
            },
        ],
    };

    let experiment = ExperimentConfig::new()
        .with_name(
            "Recovery Test".to_string(),
            "Test automatic recovery after fault injection".to_string(),
        )
        .with_fault(FaultType::ResourceExhaustion {
            resource: ResourceType::Memory,
            level: 0.8,
            duration: Duration::from_secs(3),
        })
        .with_duration(Duration::from_secs(5))
        .with_recovery(recovery_config)
        .with_success_criterion(SuccessCriterion::RecoveryTime {
            max_recovery_time: Duration::from_secs(15),
        });

    let result = chaos.run_experiment(experiment).await;
    assert!(
        result.is_ok(),
        "Recovery experiment should complete successfully"
    );

    let experiment_result = result.unwrap();
    assert!(experiment_result.config.recovery.auto_recovery);
    assert_eq!(experiment_result.config.recovery.strategies.len(), 3);
    assert_eq!(experiment_result.config.recovery.validation_steps.len(), 3);
}

/// Test experiment monitoring and metrics collection
#[tokio::test]
async fn test_experiment_monitoring() {
    let chaos = ChaosEngineer::new();

    let monitoring_config = MonitoringConfig {
        metrics: vec![
            "cpu_usage".to_string(),
            "memory_usage".to_string(),
            "response_time_ms".to_string(),
            "error_count".to_string(),
            "request_count".to_string(),
        ],
        collection_interval: Duration::from_millis(500),
        detailed_logging: true,
        alert_thresholds: [
            ("cpu_usage".to_string(), 80.0),
            ("memory_usage".to_string(), 90.0),
            ("response_time_ms".to_string(), 2000.0),
        ]
        .iter()
        .cloned()
        .collect(),
    };

    let experiment = ExperimentConfig::new()
        .with_name(
            "Monitoring Test".to_string(),
            "Test comprehensive monitoring during experiments".to_string(),
        )
        .with_fault(FaultType::NetworkFailure {
            rate: 0.1,
            latency_ms: Some(300),
            error_type: NetworkErrorType::Timeout,
        })
        .with_duration(Duration::from_secs(4))
        .with_monitoring(monitoring_config)
        .with_success_criterion(SuccessCriterion::SystemResponsive {
            max_response_time_ms: 3000,
            success_rate_threshold: 0.85,
        });

    let result = chaos.run_experiment(experiment).await;
    assert!(result.is_ok(), "Monitoring experiment should complete");

    let experiment_result = result.unwrap();
    assert_eq!(experiment_result.config.monitoring.metrics.len(), 5);
    assert!(experiment_result.config.monitoring.detailed_logging);
    assert!(!experiment_result.metrics.is_empty());
}

/// Test experiment success criteria validation
#[tokio::test]
async fn test_success_criteria_validation() {
    let chaos = ChaosEngineer::new();

    let experiment = ExperimentConfig::new()
        .with_name(
            "Success Criteria Test".to_string(),
            "Test various success criteria validation".to_string(),
        )
        .with_fault(FaultType::NetworkFailure {
            rate: 0.05, // Low failure rate
            latency_ms: Some(100),
            error_type: NetworkErrorType::Timeout,
        })
        .with_duration(Duration::from_secs(3))
        .with_success_criterion(SuccessCriterion::SystemResponsive {
            max_response_time_ms: 1000,
            success_rate_threshold: 0.95,
        })
        .with_success_criterion(SuccessCriterion::ErrorRate {
            max_error_rate: 0.1,
            window_duration: Duration::from_secs(10),
        })
        .with_success_criterion(SuccessCriterion::MetricBounds {
            metric_name: "cpu_usage".to_string(),
            min_value: 0.0,
            max_value: 100.0,
        });

    let result = chaos.run_experiment(experiment).await;
    assert!(
        result.is_ok(),
        "Success criteria experiment should complete"
    );

    let experiment_result = result.unwrap();
    assert_eq!(experiment_result.success_criteria_met.len(), 3);

    // Check that criteria were evaluated
    for (criterion, met) in &experiment_result.success_criteria_met {
        match criterion {
            SuccessCriterion::SystemResponsive { .. } => {
                // System should be responsive with low failure rate
            }
            SuccessCriterion::ErrorRate { .. } => {
                // Error rate should be within bounds
            }
            SuccessCriterion::MetricBounds { .. } => {
                // CPU usage should be within reasonable bounds
            }
            _ => {}
        }
    }
}

/// Test disk I/O fault injection
#[tokio::test]
async fn test_disk_io_fault() {
    let chaos = ChaosEngineer::new();

    let experiment = ExperimentConfig::new()
        .with_name(
            "Disk I/O Fault Test".to_string(),
            "Test system behavior under disk I/O failures".to_string(),
        )
        .with_fault(FaultType::DiskIoFailure {
            failure_rate: 0.1,
            latency_ms: Some(1000),
            target_paths: vec!["/tmp".to_string(), "/var/log".to_string()],
        })
        .with_duration(Duration::from_secs(3))
        .with_success_criterion(SuccessCriterion::SystemResponsive {
            max_response_time_ms: 5000,
            success_rate_threshold: 0.8,
        });

    let result = chaos.run_experiment(experiment).await;
    assert!(result.is_ok(), "Disk I/O fault experiment should complete");
}

/// Test experiment with custom recovery strategy
#[tokio::test]
async fn test_custom_recovery_strategy() {
    let chaos = ChaosEngineer::new();

    let recovery_config = RecoveryConfig {
        auto_recovery: true,
        recovery_timeout: Duration::from_secs(20),
        strategies: vec![
            RecoveryStrategy::CustomScript {
                script_path: "/usr/local/bin/recovery-script.sh".to_string(),
                args: vec!["--mode".to_string(), "fast".to_string()],
            },
            RecoveryStrategy::WaitForRecovery {
                duration: Duration::from_secs(2),
            },
            RecoveryStrategy::ScaleOut {
                resource: "web-servers".to_string(),
                factor: 1.5,
            },
        ],
        validation_steps: vec![
            ValidationStep::CustomValidation {
                script_path: "/usr/local/bin/validate-health.sh".to_string(),
                expected_exit_code: 0,
            },
            ValidationStep::ServiceAvailability {
                service_names: vec![
                    "web-server".to_string(),
                    "database".to_string(),
                    "cache".to_string(),
                ],
            },
        ],
    };

    let experiment = ExperimentConfig::new()
        .with_name(
            "Custom Recovery Test".to_string(),
            "Test custom recovery strategies and validation".to_string(),
        )
        .with_fault(FaultType::ServiceUnavailable {
            service_name: "critical-service".to_string(),
            duration: Duration::from_secs(4),
            error_response: Some("Service down for maintenance".to_string()),
        })
        .with_duration(Duration::from_secs(6))
        .with_recovery(recovery_config);

    let result = chaos.run_experiment(experiment).await;
    assert!(result.is_ok(), "Custom recovery experiment should complete");

    let experiment_result = result.unwrap();
    assert_eq!(experiment_result.config.recovery.strategies.len(), 3);
    assert_eq!(experiment_result.config.recovery.validation_steps.len(), 2);
}

/// Test concurrent experiments
#[tokio::test]
async fn test_concurrent_experiments() {
    let chaos = ChaosEngineer::new();

    // Start multiple experiments concurrently
    let mut experiment_futures = Vec::new();

    for i in 0..3 {
        let experiment = ExperimentConfig::new()
            .with_name(
                format!("Concurrent Test {}", i),
                format!("Concurrent experiment number {}", i),
            )
            .with_fault(FaultType::NetworkFailure {
                rate: 0.05,
                latency_ms: Some(200 + i * 100),
                error_type: NetworkErrorType::Timeout,
            })
            .with_duration(Duration::from_secs(2 + i));

        experiment_futures.push(chaos.run_experiment(experiment));
    }

    // Wait for all experiments to complete
    let results = futures::future::join_all(experiment_futures).await;

    // Verify all experiments completed successfully
    for (i, result) in results.iter().enumerate() {
        assert!(result.is_ok(), "Concurrent experiment {} should succeed", i);
    }

    assert_eq!(
        results.len(),
        3,
        "Should have 3 concurrent experiment results"
    );
}

/// Test experiment with all fault types
#[tokio::test]
async fn test_comprehensive_fault_types() {
    let chaos = ChaosEngineer::new();

    let experiment = ExperimentConfig::new()
        .with_name(
            "Comprehensive Fault Test".to_string(),
            "Test system with various fault types".to_string(),
        )
        // Network faults
        .with_fault(FaultType::NetworkFailure {
            rate: 0.02,
            latency_ms: Some(150),
            error_type: NetworkErrorType::DnsFailure,
        })
        // Resource exhaustion
        .with_fault(FaultType::ResourceExhaustion {
            resource: ResourceType::FileDescriptors,
            level: 0.7,
            duration: Duration::from_secs(2),
        })
        // Service unavailability
        .with_fault(FaultType::ServiceUnavailable {
            service_name: "api-gateway".to_string(),
            duration: Duration::from_secs(1),
            error_response: None,
        })
        // Memory pressure (small amount)
        .with_fault(FaultType::MemoryPressure {
            allocation_mb: 10,
            duration: Duration::from_secs(2),
            gradual: true,
        })
        // CPU starvation (light)
        .with_fault(FaultType::CpuStarvation {
            cpu_percentage: 20.0,
            duration: Duration::from_secs(1),
            threads: 1,
        })
        // Disk I/O issues
        .with_fault(FaultType::DiskIoFailure {
            failure_rate: 0.05,
            latency_ms: Some(500),
            target_paths: vec!["/tmp".to_string()],
        })
        .with_duration(Duration::from_secs(8))
        .with_success_criterion(SuccessCriterion::ErrorRate {
            max_error_rate: 0.3, // Allow higher error rate due to multiple faults
            window_duration: Duration::from_secs(15),
        });

    let result = chaos.run_experiment(experiment).await;
    assert!(
        result.is_ok(),
        "Comprehensive fault experiment should complete"
    );

    let experiment_result = result.unwrap();
    assert_eq!(experiment_result.config.faults.len(), 6);

    // Verify we got some metrics and events
    assert!(
        !experiment_result.metrics.is_empty(),
        "Should have collected metrics"
    );
    assert!(
        !experiment_result.events.is_empty(),
        "Should have recorded events"
    );

    // Verify recommendations were generated
    assert!(
        !experiment_result.recommendations.is_empty(),
        "Should have recommendations"
    );
}

/// Test chaos engineering with realistic AI system scenario
#[tokio::test]
async fn test_ai_system_resilience() {
    let chaos = ChaosEngineer::new();

    // Simulate a realistic AI system scenario
    let experiment = ExperimentConfig::new()
        .with_name(
            "AI System Resilience Test".to_string(),
            "Test AI system resilience under various failure conditions".to_string(),
        )
        .with_target("ai-inference-service".to_string())
        .with_target("context-analyzer".to_string())
        .with_target("session-manager".to_string())
        // AI provider network issues
        .with_fault(FaultType::NetworkFailure {
            rate: 0.1,
            latency_ms: Some(2000), // High latency to AI providers
            error_type: NetworkErrorType::Timeout,
        })
        // Memory pressure from context processing
        .with_fault(FaultType::MemoryPressure {
            allocation_mb: 100,
            duration: Duration::from_secs(5),
            gradual: true,
        })
        // Session service unavailability
        .with_fault(FaultType::ServiceUnavailable {
            service_name: "session-manager".to_string(),
            duration: Duration::from_secs(3),
            error_response: Some("Session service temporarily unavailable".to_string()),
        })
        .with_duration(Duration::from_secs(10))
        .with_success_criterion(SuccessCriterion::SystemResponsive {
            max_response_time_ms: 8000,  // Allow longer response times
            success_rate_threshold: 0.7, // Lower threshold due to AI complexity
        })
        .with_success_criterion(SuccessCriterion::ErrorRate {
            max_error_rate: 0.25,
            window_duration: Duration::from_secs(20),
        })
        .with_recovery(RecoveryConfig {
            auto_recovery: true,
            recovery_timeout: Duration::from_secs(60),
            strategies: vec![
                RecoveryStrategy::ClearCaches,
                RecoveryStrategy::ResetConnections,
                RecoveryStrategy::MemoryCleanup,
                RecoveryStrategy::RestartServices,
            ],
            validation_steps: vec![
                ValidationStep::MemoryUsage {
                    max_usage_percent: 85.0,
                },
                ValidationStep::ResponseTimeCheck {
                    max_response_time_ms: 5000,
                },
                ValidationStep::ServiceAvailability {
                    service_names: vec![
                        "ai-inference-service".to_string(),
                        "context-analyzer".to_string(),
                        "session-manager".to_string(),
                    ],
                },
            ],
        });

    let result = chaos.run_experiment(experiment).await;
    assert!(result.is_ok(), "AI system resilience test should complete");

    let experiment_result = result.unwrap();

    // Verify experiment structure
    assert_eq!(experiment_result.config.targets.len(), 3);
    assert_eq!(experiment_result.config.faults.len(), 3);
    assert!(experiment_result.config.recovery.auto_recovery);

    // Verify we have realistic results
    assert!(experiment_result.summary.health_score >= 0.0);
    assert!(experiment_result.summary.health_score <= 1.0);

    // Should have detailed recommendations for AI systems
    let recommendations = &experiment_result.recommendations;
    assert!(
        !recommendations.is_empty(),
        "Should have AI-specific recommendations"
    );

    // Check for AI-specific recommendations
    let has_ai_recommendations = recommendations
        .iter()
        .any(|rec| rec.contains("AI") || rec.contains("provider") || rec.contains("inference"));
    assert!(
        has_ai_recommendations,
        "Should have AI-specific recommendations"
    );
}
