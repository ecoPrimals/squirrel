//! Comprehensive Ecosystem Demo
//!
//! This demo showcases the complete ecosystem with benchmarking, graceful shutdown,
//! and self-healing capabilities working together.

use squirrel::{
    benchmarking::{run_ecosystem_benchmarks, BenchmarkConfig},
    initialize_complete_ecosystem,
    self_healing::{initialize_self_healing, RecoveryStrategy, SelfHealingConfig},
    shutdown::{initialize_shutdown_system, ShutdownConfig},
};
use std::time::Duration;
use tokio::signal;
use tokio::time::{sleep, timeout};
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("🚀 Starting Comprehensive Ecosystem Demo");

    // Initialize the complete ecosystem
    let ecosystem = initialize_complete_ecosystem().await?;

    // Demonstrate benchmarking capabilities
    demo_benchmarking(&ecosystem).await?;

    // Demonstrate self-healing capabilities
    demo_self_healing(&ecosystem).await?;

    // Set up graceful shutdown handling
    let shutdown_handle = tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for ctrl-c");
        info!("🛑 Received shutdown signal");
    });

    // Run main application loop
    info!("📱 Starting main application loop");
    let main_loop_handle = tokio::spawn(async move {
        simulate_application_workload().await;
    });

    // Wait for shutdown signal or main loop completion
    tokio::select! {
        _ = shutdown_handle => {
            info!("🔄 Initiating graceful shutdown");

            // Cancel main loop
            main_loop_handle.abort();

            // Perform graceful shutdown
            if let Err(e) = ecosystem.shutdown().await {
                error!("❌ Shutdown failed: {}", e);
            } else {
                info!("✅ Graceful shutdown completed successfully");
            }
        }
        _ = main_loop_handle => {
            info!("🏁 Main loop completed");
        }
    }

    info!("🎉 Comprehensive Ecosystem Demo completed");
    Ok(())
}

/// Demonstrate benchmarking capabilities
async fn demo_benchmarking(
    ecosystem: &squirrel::EcosystemManager,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("📊 Demonstrating benchmarking capabilities");

    // Run comprehensive benchmarks
    info!("🔍 Running comprehensive ecosystem benchmarks");
    let benchmark_report = ecosystem.run_benchmarks().await?;

    // Display benchmark results
    info!("📈 Benchmark Results Summary:");
    info!(
        "  - Total benchmarks: {}",
        benchmark_report.total_benchmarks
    );
    info!("  - Total duration: {:?}", benchmark_report.total_duration);
    info!("  - System metrics: {:?}", benchmark_report.system_metrics);

    let summary = benchmark_report.generate_summary();
    info!("📋 Performance Summary:");
    info!("  - Total operations: {}", summary.total_operations);
    info!("  - Average ops/sec: {:.2}", summary.average_ops_per_second);
    info!(
        "  - Success rate: {:.2}%",
        summary.average_success_rate * 100.0
    );
    info!(
        "  - Peak memory usage: {:.2} MB",
        summary.peak_memory_usage_mb
    );
    info!("  - Peak CPU usage: {:.2}%", summary.peak_cpu_usage_percent);

    // Show top performing operations
    let mut sorted_results = benchmark_report.results.clone();
    sorted_results.sort_by(|a, b| b.ops_per_second.partial_cmp(&a.ops_per_second).unwrap());

    info!("🏆 Top performing operations:");
    for (i, result) in sorted_results.iter().take(5).enumerate() {
        info!(
            "  {}. {}: {:.2} ops/sec",
            i + 1,
            result.operation_name,
            result.ops_per_second
        );
    }

    Ok(())
}

/// Demonstrate self-healing capabilities
async fn demo_self_healing(
    ecosystem: &squirrel::EcosystemManager,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("🏥 Demonstrating self-healing capabilities");

    // Get current health status
    let health_status = ecosystem.get_health_status().await?;

    info!("🔍 Current system health:");
    for (component_id, health) in &health_status {
        info!(
            "  - {}: {:?} ({})",
            component_id, health.status, health.component_name
        );
    }

    // Subscribe to self-healing events
    if let Some(self_healing) = &ecosystem.self_healing {
        let mut event_receiver = self_healing.subscribe_to_events();

        // Spawn a task to monitor self-healing events
        let event_monitor = tokio::spawn(async move {
            while let Ok(event) = event_receiver.recv().await {
                match event {
                    squirrel::self_healing::SelfHealingEvent::HealthStatusChanged {
                        component_id,
                        old_status,
                        new_status,
                        ..
                    } => {
                        info!(
                            "🔄 Health status changed for {}: {:?} -> {:?}",
                            component_id, old_status, new_status
                        );
                    }
                    squirrel::self_healing::SelfHealingEvent::CircuitBreakerStateChanged {
                        component_id,
                        old_state,
                        new_state,
                        ..
                    } => {
                        info!(
                            "⚡ Circuit breaker state changed for {}: {:?} -> {:?}",
                            component_id, old_state, new_state
                        );
                    }
                    squirrel::self_healing::SelfHealingEvent::RecoveryInitiated {
                        component_id,
                        strategy,
                        ..
                    } => {
                        info!(
                            "🚑 Recovery initiated for {} with strategy: {:?}",
                            component_id, strategy
                        );
                    }
                    squirrel::self_healing::SelfHealingEvent::RecoveryCompleted {
                        component_id,
                        result,
                        ..
                    } => {
                        if result.success {
                            info!("✅ Recovery completed for {}", component_id);
                        } else {
                            warn!(
                                "❌ Recovery failed for {}: {:?}",
                                component_id, result.error_message
                            );
                        }
                    }
                    squirrel::self_healing::SelfHealingEvent::ServiceDegraded {
                        component_id,
                        reason,
                        ..
                    } => {
                        warn!("⚠️  Service degraded for {}: {}", component_id, reason);
                    }
                    squirrel::self_healing::SelfHealingEvent::ServiceRecovered {
                        component_id,
                        ..
                    } => {
                        info!("🎉 Service recovered for {}", component_id);
                    }
                }
            }
        });

        // Simulate a component failure and recovery
        info!("🧪 Simulating component failure and recovery");

        // Simulate recovery action
        if let Ok(recovery_result) = self_healing
            .perform_recovery("ai_intelligence", RecoveryStrategy::Restart)
            .await
        {
            info!("🔧 Recovery action completed: {:?}", recovery_result);
        }

        // Let the event monitor run for a bit
        sleep(Duration::from_secs(2)).await;

        // Stop the event monitor
        event_monitor.abort();
    }

    // Show circuit breaker metrics
    if let Some(self_healing) = &ecosystem.self_healing {
        let circuit_breaker_metrics = self_healing.get_circuit_breaker_metrics().await;

        info!("⚡ Circuit breaker metrics:");
        for (component_id, metrics) in &circuit_breaker_metrics {
            info!(
                "  - {}: {:?} (failures: {}, successes: {})",
                component_id, metrics.state, metrics.failure_count, metrics.success_count
            );
        }

        // Show recovery history
        let recovery_history = self_healing.get_recovery_history().await;
        info!("📜 Recovery history ({} entries):", recovery_history.len());
        for (i, recovery) in recovery_history.iter().take(5).enumerate() {
            info!(
                "  {}. {}: {:?} ({})",
                i + 1,
                recovery.component_id,
                recovery.strategy,
                if recovery.success { "✅" } else { "❌" }
            );
        }
    }

    Ok(())
}

/// Simulate application workload
async fn simulate_application_workload() {
    info!("💼 Simulating application workload");

    for i in 0..100 {
        // Simulate different types of operations
        match i % 5 {
            0 => {
                // AI processing operation
                info!("🤖 Processing AI request #{}", i);
                sleep(Duration::from_millis(100)).await;
            }
            1 => {
                // Storage operation
                info!("💾 Performing storage operation #{}", i);
                sleep(Duration::from_millis(50)).await;
            }
            2 => {
                // Compute operation
                info!("⚙️  Executing compute task #{}", i);
                sleep(Duration::from_millis(200)).await;
            }
            3 => {
                // Orchestration operation
                info!("🎼 Orchestrating services #{}", i);
                sleep(Duration::from_millis(75)).await;
            }
            4 => {
                // Security operation
                info!("🔒 Performing security check #{}", i);
                sleep(Duration::from_millis(25)).await;
            }
            _ => {}
        }

        // Simulate some processing time
        sleep(Duration::from_millis(500)).await;

        // Occasionally simulate a failure
        if i % 20 == 0 && i > 0 {
            warn!("⚠️  Simulated failure in operation #{}", i);
            sleep(Duration::from_millis(1000)).await;
        }
    }

    info!("🏁 Application workload simulation completed");
}

/// Example of custom benchmark configuration
#[allow(dead_code)]
async fn custom_benchmark_demo() -> Result<(), Box<dyn std::error::Error>> {
    info!("🎯 Running custom benchmark demo");

    let benchmark_suite = squirrel::benchmarking::initialize_benchmarking().await?;

    // Create custom benchmark configuration
    let custom_config = BenchmarkConfig {
        name: "custom_demo_benchmark".to_string(),
        duration: Duration::from_secs(10),
        operation_count: 1000,
        warm_up_duration: Duration::from_secs(2),
        cool_down_duration: Duration::from_secs(1),
        memory_monitoring: true,
        cpu_monitoring: true,
        collect_detailed_metrics: true,
        ..Default::default()
    };

    // Run custom benchmark
    let result = benchmark_suite
        .run_benchmark("custom_operation", custom_config, || async {
            // Custom operation to benchmark
            let mut data = Vec::new();
            for i in 0..100 {
                data.push(i * i);
            }

            // Simulate some processing
            sleep(Duration::from_millis(1)).await;

            Ok(())
        })
        .await?;

    info!("📊 Custom benchmark result:");
    info!("  - Operation: {}", result.operation_name);
    info!("  - Duration: {:.2} ms", result.duration_ms);
    info!("  - Ops/sec: {:.2}", result.ops_per_second);
    info!("  - Success rate: {:.2}%", result.success_rate * 100.0);
    info!("  - Memory usage: {:.2} MB", result.memory_usage_mb);
    info!("  - CPU usage: {:.2}%", result.cpu_usage_percent);

    Ok(())
}

/// Example of custom shutdown configuration
#[allow(dead_code)]
async fn custom_shutdown_demo() -> Result<(), Box<dyn std::error::Error>> {
    info!("🔄 Running custom shutdown demo");

    // Create custom shutdown configuration
    let shutdown_config = ShutdownConfig {
        graceful_timeout: Duration::from_secs(60),
        forceful_timeout: Duration::from_secs(15),
        persist_state: true,
        wait_for_operations: true,
        parallel_shutdown: true,
        max_concurrent_shutdowns: 10,
    };

    // Initialize shutdown system with custom config
    let shutdown_manager = squirrel::shutdown::ShutdownManager::new(shutdown_config);

    // Register custom components
    shutdown_manager
        .register_component(
            "custom_component".to_string(),
            "Custom Component".to_string(),
            squirrel::shutdown::ShutdownPriority::Medium,
        )
        .await;

    // Get shutdown status
    let (phase, components) = shutdown_manager.get_shutdown_status().await;
    info!("🔍 Shutdown status: {:?}", phase);
    info!("🔍 Registered components: {}", components.len());

    Ok(())
}

/// Example of custom self-healing configuration
#[allow(dead_code)]
async fn custom_self_healing_demo() -> Result<(), Box<dyn std::error::Error>> {
    info!("🏥 Running custom self-healing demo");

    // Create custom self-healing configuration
    let self_healing_config = SelfHealingConfig {
        enabled: true,
        health_check_interval: Duration::from_secs(15),
        recovery_timeout: Duration::from_secs(30),
        max_recovery_attempts: 5,
        auto_recovery_enabled: true,
        circuit_breaker_enabled: true,
        degradation_enabled: true,
    };

    // Initialize self-healing system with custom config
    let self_healing_manager = squirrel::self_healing::SelfHealingManager::new(self_healing_config);

    // Register custom component
    self_healing_manager
        .register_component("custom_service".to_string(), "Custom Service".to_string())
        .await?;

    // Start self-healing
    self_healing_manager.start().await?;

    // Get health status
    let health_status = self_healing_manager.get_health_status().await;
    info!("🔍 Health status: {} components", health_status.len());

    // Stop self-healing
    self_healing_manager.stop().await?;

    Ok(())
}

/// Performance monitoring example
#[allow(dead_code)]
async fn performance_monitoring_demo() -> Result<(), Box<dyn std::error::Error>> {
    info!("📈 Running performance monitoring demo");

    let benchmark_suite = squirrel::benchmarking::initialize_benchmarking().await?;

    // Run a series of benchmarks and monitor performance
    let benchmark_names = vec![
        "ai_processing",
        "storage_operations",
        "compute_tasks",
        "orchestration_work",
        "security_checks",
    ];

    let mut all_results = Vec::new();

    for benchmark_name in benchmark_names {
        info!("🔍 Running benchmark: {}", benchmark_name);

        let config = BenchmarkConfig {
            name: benchmark_name.to_string(),
            duration: Duration::from_secs(5),
            operation_count: 100,
            ..Default::default()
        };

        let result = benchmark_suite
            .run_benchmark(benchmark_name, config, || async {
                // Simulate different operation types
                let delay = match benchmark_name {
                    "ai_processing" => Duration::from_millis(20),
                    "storage_operations" => Duration::from_millis(10),
                    "compute_tasks" => Duration::from_millis(50),
                    "orchestration_work" => Duration::from_millis(15),
                    "security_checks" => Duration::from_millis(5),
                    _ => Duration::from_millis(10),
                };

                sleep(delay).await;
                Ok(())
            })
            .await?;

        all_results.push(result);
    }

    // Analyze performance trends
    info!("📊 Performance analysis:");
    for result in &all_results {
        info!(
            "  - {}: {:.2} ops/sec ({:.2} ms avg)",
            result.operation_name,
            result.ops_per_second,
            result.duration_ms / result.operations_count as f64
        );
    }

    // Find performance bottlenecks
    let slowest = all_results
        .iter()
        .min_by(|a, b| a.ops_per_second.partial_cmp(&b.ops_per_second).unwrap());
    let fastest = all_results
        .iter()
        .max_by(|a, b| a.ops_per_second.partial_cmp(&b.ops_per_second).unwrap());

    if let (Some(slowest), Some(fastest)) = (slowest, fastest) {
        info!(
            "🐌 Slowest operation: {} ({:.2} ops/sec)",
            slowest.operation_name, slowest.ops_per_second
        );
        info!(
            "🚀 Fastest operation: {} ({:.2} ops/sec)",
            fastest.operation_name, fastest.ops_per_second
        );

        let performance_ratio = fastest.ops_per_second / slowest.ops_per_second;
        info!("⚡ Performance ratio: {:.2}x", performance_ratio);
    }

    Ok(())
}
