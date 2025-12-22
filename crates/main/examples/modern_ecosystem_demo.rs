//! # Modern Concurrent Ecosystem Demo
//!
//! Demonstrates truly concurrent, non-blocking ecosystem patterns:
//! - Zero sleep() calls - only real async work
//! - Channel-based synchronization
//! - Barrier patterns for coordination
//! - Pure capability-based discovery
//! - No serial operations except where truly needed

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Barrier, RwLock};
use tokio::time::timeout;
use uuid::Uuid;

use squirrel::ecosystem::{
    initialize_ecosystem_integration, DiscoveredService, EcosystemConfig, EcosystemManager,
    EcosystemPrimalType, EcosystemRegistryConfig, HealthCheckConfig, PrimalApiRequest,
    PrimalApiResponse, ResourceSpec, SecurityConfig,
};
use squirrel::error::PrimalError;
use squirrel::monitoring::metrics::MetricsCollector;
use squirrel::primal_provider::SquirrelPrimalProvider;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🌟 Modern Concurrent Ecosystem Demo");
    println!("====================================");
    println!("✅ Zero sleep() calls");
    println!("✅ Pure async/await patterns");
    println!("✅ Channel-based coordination");
    println!("✅ Concurrent service discovery");
    println!();

    // Initialize metrics (shared across concurrent operations)
    let metrics_collector = Arc::new(MetricsCollector::new());

    // Configure ecosystem with proper struct fields
    let ecosystem_config = create_modern_config();

    // Initialize ecosystem manager
    let ecosystem_manager =
        initialize_ecosystem_integration(ecosystem_config, metrics_collector.clone()).await?;
    let ecosystem_manager = Arc::new(ecosystem_manager);

    println!("✅ Ecosystem manager initialized");
    println!();

    // Create channel for concurrent operations
    let (tx, mut rx) = mpsc::channel(100);

    // Spawn concurrent discovery tasks
    println!("🚀 Launching concurrent service discovery...");
    let discovery_tasks =
        spawn_concurrent_discovery(Arc::clone(&ecosystem_manager), tx.clone()).await;

    // Collect results concurrently
    let mut discovered = Vec::new();
    while let Some(service) = rx.recv().await {
        discovered.push(service);
        if discovered.len() >= 3 {
            break;
        }
    }

    println!("✅ Discovered {} services concurrently", discovered.len());

    // Wait for all discovery tasks
    for task in discovery_tasks {
        if let Err(e) = task.await {
            eprintln!("   ⚠️  Discovery task failed: {}", e);
        }
    }

    // Demonstrate concurrent health checks (no sleep, pure async)
    demonstrate_concurrent_health_checks(&ecosystem_manager).await?;

    // Demonstrate barrier-based coordination
    demonstrate_barrier_coordination(&ecosystem_manager).await?;

    // Graceful shutdown
    ecosystem_manager.shutdown().await?;
    println!("✅ Clean shutdown completed");
    println!();

    println!("🎉 Demo completed - all operations were truly concurrent!");
    println!("📋 Key achievements:");
    println!("   ✅ No sleep() calls - only real async work");
    println!("   ✅ Channel-based service discovery");
    println!("   ✅ Barrier-based coordination");
    println!("   ✅ Concurrent health monitoring");
    println!("   ✅ Pure async/await throughout");

    Ok(())
}

/// Create ecosystem configuration with correct struct fields
fn create_modern_config() -> EcosystemConfig {
    EcosystemConfig {
        service_id: format!("primal-squirrel-modern-{}", Uuid::new_v4()),
        service_name: "Squirrel AI Primal (Modern)".to_string(),
        service_host: "localhost".to_string(),
        service_port: 8002,
        songbird_endpoint: std::env::var("SONGBIRD_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:8001".to_string()),
        biome_id: Some("modern-demo-biome".to_string()),
        registry_config: EcosystemRegistryConfig::default(),
        resource_requirements: ResourceSpec::default(),
        security_config: SecurityConfig {
            auth_required: true,
            encryption_level: "tls".to_string(),
            access_level: "authenticated".to_string(),
            policies: vec!["oauth2".to_string()],
            audit_enabled: true,
            security_level: "high".to_string(),
        },
        health_check: HealthCheckConfig {
            enabled: true,
            interval_secs: 30,
            timeout_secs: 10,
            failure_threshold: 3,
        },
        metadata: {
            let mut m = HashMap::new();
            m.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
            m.insert("mode".to_string(), "concurrent".to_string());
            m.insert("pattern".to_string(), "modern-async".to_string());
            m
        },
    }
}

/// Spawn concurrent service discovery tasks
async fn spawn_concurrent_discovery(
    manager: Arc<EcosystemManager>,
    tx: mpsc::Sender<DiscoveredService>,
) -> Vec<tokio::task::JoinHandle<()>> {
    let mut tasks = Vec::new();

    // Discovery task 1: AI services
    let manager1 = Arc::clone(&manager);
    let tx1 = tx.clone();
    tasks.push(tokio::spawn(async move {
        println!("   🔍 Task 1: Discovering AI services...");
        match timeout(Duration::from_secs(5), manager1.discover_services()).await {
            Ok(Ok(services)) => {
                for service in services {
                    let _ = tx1.send(service).await;
                }
                println!("   ✅ Task 1: AI services discovered");
            }
            Ok(Err(e)) => println!("   ⚠️  Task 1: Discovery error: {}", e),
            Err(_) => println!("   ⚠️  Task 1: Discovery timeout"),
        }
    }));

    // Discovery task 2: Storage services
    let manager2 = Arc::clone(&manager);
    let tx2 = tx.clone();
    tasks.push(tokio::spawn(async move {
        println!("   🔍 Task 2: Discovering storage services...");
        match timeout(Duration::from_secs(5), manager2.discover_services()).await {
            Ok(Ok(services)) => {
                for service in services {
                    let _ = tx2.send(service).await;
                }
                println!("   ✅ Task 2: Storage services discovered");
            }
            Ok(Err(e)) => println!("   ⚠️  Task 2: Discovery error: {}", e),
            Err(_) => println!("   ⚠️  Task 2: Discovery timeout"),
        }
    }));

    // Discovery task 3: Security services
    let manager3 = manager;
    let tx3 = tx;
    tasks.push(tokio::spawn(async move {
        println!("   🔍 Task 3: Discovering security services...");
        match timeout(Duration::from_secs(5), manager3.discover_services()).await {
            Ok(Ok(services)) => {
                for service in services {
                    let _ = tx3.send(service).await;
                }
                println!("   ✅ Task 3: Security services discovered");
            }
            Ok(Err(e)) => println!("   ⚠️  Task 3: Discovery error: {}", e),
            Err(_) => println!("   ⚠️  Task 3: Discovery timeout"),
        }
    }));

    tasks
}

/// Demonstrate concurrent health checks without sleep
async fn demonstrate_concurrent_health_checks(
    manager: &Arc<EcosystemManager>,
) -> Result<(), PrimalError> {
    println!();
    println!("💓 Concurrent Health Check Demo");
    println!("--------------------------------");

    // Use channels for concurrent health collection
    let (tx, mut rx) = mpsc::channel(10);
    let health_data = Arc::new(RwLock::new(Vec::new()));

    // Spawn health check collectors
    let tasks: Vec<_> = (0..5)
        .map(|i| {
            let tx = tx.clone();
            let manager = Arc::clone(manager);
            tokio::spawn(async move {
                println!("   🔍 Health checker {} starting...", i);
                match timeout(Duration::from_secs(2), manager.get_manager_status()).await {
                    Ok(_status) => {
                        let _ = tx.send(format!("Checker {}: OK", i)).await;
                        println!("   ✅ Health checker {} completed", i);
                    }
                    Err(_) => {
                        let _ = tx.send(format!("Checker {}: Timeout", i)).await;
                    }
                }
            })
        })
        .collect();

    // Close sender
    drop(tx);

    // Collect results concurrently
    let health_data_clone = health_data.clone();
    let collector = tokio::spawn(async move {
        let mut count = 0;
        while let Some(result) = rx.recv().await {
            health_data_clone.write().await.push(result);
            count += 1;
        }
        count
    });

    // Wait for all tasks
    for task in tasks {
        task.await.ok();
    }

    let count = collector
        .await
        .map_err(|e| PrimalError::Internal(format!("Collector error: {}", e)))?;
    let results = health_data.read().await;

    println!("✅ Collected {} health check results concurrently", count);
    println!("   Results: {:?}", &results[..results.len().min(3)]);

    Ok(())
}

/// Demonstrate barrier-based coordination (no sleep)
async fn demonstrate_barrier_coordination(
    manager: &Arc<EcosystemManager>,
) -> Result<(), PrimalError> {
    println!();
    println!("🔄 Barrier-Based Coordination Demo");
    println!("-----------------------------------");

    // Create barrier for N coordinated tasks
    let n_tasks = 4;
    let barrier = Arc::new(Barrier::new(n_tasks));

    let tasks: Vec<_> = (0..n_tasks)
        .map(|i| {
            let barrier = Arc::clone(&barrier);
            let manager = Arc::clone(manager);
            tokio::spawn(async move {
                println!("   🎯 Coordinator {} preparing...", i);

                // Do real async work here
                match timeout(Duration::from_secs(1), manager.get_manager_status()).await {
                    Ok(_status) => println!("   ✅ Coordinator {} ready", i),
                    Err(_) => println!("   ⚠️  Coordinator {} had timeout", i),
                }

                // Wait at barrier - all tasks synchronize here
                barrier.wait().await;

                println!("   🚀 Coordinator {} proceeding after synchronization", i);
            })
        })
        .collect();

    // Wait for all coordinators
    for task in tasks {
        task.await.ok();
    }

    println!("✅ All coordinators synchronized via barrier (no sleep!)");

    Ok(())
}
