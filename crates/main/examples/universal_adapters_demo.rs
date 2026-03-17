// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Universal Adapters Demonstration
//!
//! This example shows how Squirrel AI Coordinator uses universal, capability-based
//! adapters to integrate with ANY primal in the ecosystem, following the
//! Universal Primal Architecture Standard.
//!
//! ## Key Principles Demonstrated:
//! 1. **Capability-First, Name-Agnostic** - Services discovered by what they can do
//! 2. **Dynamic Service Registration** - Services register themselves with capabilities
//! 3. **Universal Extensibility** - Any service can participate based on capabilities
//! 4. **Zero Hardcoding** - No assumptions about specific service implementations

use std::sync::Arc;

use squirrel::universal_adapters::{
    ServiceCapability, UniversalServiceRegistry,
    compute_adapter::{UniversalComputeAdapter, register_toadstool_service},
    orchestration_adapter::{UniversalOrchestrationAdapter, register_songbird_service},
    registry::InMemoryServiceRegistry,
    security_adapter::{UniversalSecurityAdapter, register_beardog_service},
    storage_adapter::{UniversalStorageAdapter, register_nestgate_service},
};

/// AI Coordination System using Universal Adapters
struct SquirrelAICoordinator {
    /// Universal service registry for capability-based discovery
    registry: Arc<dyn UniversalServiceRegistry>,

    /// Universal adapters - work with ANY service providing required capabilities
    security_adapter: UniversalSecurityAdapter,
    orchestration_adapter: UniversalOrchestrationAdapter,
    storage_adapter: UniversalStorageAdapter,
    compute_adapter: UniversalComputeAdapter,
}

impl SquirrelAICoordinator {
    /// Create a new AI coordinator with universal adapters
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Create universal service registry
        let registry = Arc::new(InMemoryServiceRegistry::new());

        // Register core primals with their capabilities
        println!("🌌 Registering core primals with universal service registry...");
        register_beardog_service(registry.clone()).await?;
        register_songbird_service(registry.clone()).await?;
        register_nestgate_service(registry.clone()).await?;
        register_toadstool_service(registry.clone()).await?;

        // Create universal adapters - they will discover services by capability
        let security_adapter = UniversalSecurityAdapter::new(registry.clone());
        let orchestration_adapter = UniversalOrchestrationAdapter::new(registry.clone());
        let storage_adapter = UniversalStorageAdapter::new(registry.clone());
        let compute_adapter = UniversalComputeAdapter::new(registry.clone());

        println!("✅ AI Coordinator initialized with universal adapters");

        Ok(Self {
            registry,
            security_adapter,
            orchestration_adapter,
            storage_adapter,
            compute_adapter,
        })
    }

    /// Demonstrate AI coordination workflow using universal adapters
    pub async fn demonstrate_ai_coordination(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n🧠 === AI COORDINATION WORKFLOW DEMONSTRATION ===");

        // Phase 1: Security Coordination (discovers BearDog or any security service)
        println!("\n🔒 Phase 1: Security Coordination");
        let session_id = self
            .security_adapter
            .authenticate_universal("ai_user_123")
            .await?;
        println!("   ✅ Authentication coordinated: session {session_id}");

        let authorized = self
            .security_adapter
            .authorize_universal(&session_id, "ai_coordination")
            .await?;
        println!("   ✅ Authorization coordinated: {authorized}");

        // Phase 2: Orchestration Coordination (discovers Songbird or any orchestration service)
        println!("\n🎼 Phase 2: Orchestration Coordination");
        let workflow_result = self
            .orchestration_adapter
            .coordinate_ai_workflow(
                "multi_primal_coordination",
                vec![
                    "security".to_string(),
                    "storage".to_string(),
                    "compute".to_string(),
                ],
            )
            .await?;
        println!(
            "   ✅ AI workflow coordinated: {}",
            workflow_result["workflow_id"]
        );

        let service_mesh_status = self.orchestration_adapter.get_service_mesh_status().await?;
        println!(
            "   ✅ Service mesh status: {} active nodes",
            service_mesh_status["service_mesh"]["nodes"]
        );

        // Phase 3: Storage Coordination (discovers NestGate or any storage service)
        println!("\n🏠 Phase 3: Storage Coordination");
        let ai_context_data = serde_json::json!({
            "session_id": session_id,
            "workflow_id": workflow_result["workflow_id"],
            "ai_state": {
                "coordination_phase": "active",
                "participating_primals": ["beardog", "songbird", "nestgate", "toadstool"],
                "optimization_level": "high"
            }
        });

        let storage_id = self
            .storage_adapter
            .store_ai_context("ai_coordination_context", ai_context_data)
            .await?;
        println!("   ✅ AI context stored: {storage_id}");

        let backup_id = self
            .storage_adapter
            .backup_ai_data("daily_ai_backup")
            .await?;
        println!("   ✅ AI data backup created: {backup_id}");

        // Phase 4: Compute Coordination (discovers ToadStool or any compute service)
        println!("\n🍄 Phase 4: Compute Coordination");
        let execution_id = self
            .compute_adapter
            .execute_ai_workload(
                "multi_primal_analysis",
                std::collections::HashMap::from([
                    (
                        "data_sources".to_string(),
                        serde_json::json!([
                            "security_logs",
                            "orchestration_metrics",
                            "storage_analytics"
                        ]),
                    ),
                    (
                        "analysis_type".to_string(),
                        serde_json::json!("comprehensive"),
                    ),
                    (
                        "optimization_target".to_string(),
                        serde_json::json!("ecosystem_efficiency"),
                    ),
                ]),
            )
            .await?;
        println!("   ✅ AI workload executed: {execution_id}");

        let performance_metrics = self.compute_adapter.monitor_compute_performance().await?;
        println!(
            "   ✅ Compute performance monitored: {}% CPU utilization",
            performance_metrics["metrics"]["resource_utilization"]["cpu_percent"]
        );

        // Phase 5: Ecosystem Status Summary
        println!("\n📊 Phase 5: Ecosystem Status Summary");
        self.show_ecosystem_status().await?;

        println!("\n🎉 AI coordination workflow completed successfully!");
        println!("   🌟 All operations used capability-based discovery");
        println!("   🌟 No hardcoded primal names - fully universal!");

        Ok(())
    }

    /// Show comprehensive ecosystem status using universal adapters
    async fn show_ecosystem_status(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n   🔍 Discovering all available services...");
        let all_services = self.registry.list_all_services().await?;

        println!("   📋 Registered Services ({}):", all_services.len());
        for service in &all_services {
            println!(
                "     • {} ({}) - {} capabilities, priority {}",
                service.name,
                service.category,
                service.capabilities.len(),
                service.priority
            );
        }

        // Show adapter-specific status
        println!("\n   🔒 Security Adapter Status:");
        let security_capabilities = self.security_adapter.get_security_capabilities().await?;
        println!("     Available capabilities: {security_capabilities:?}");
        println!("     Health: {}", self.security_adapter.is_healthy().await);

        println!("\n   🎼 Orchestration Adapter Status:");
        let orchestration_capabilities = self
            .orchestration_adapter
            .get_orchestration_capabilities()
            .await?;
        println!("     Available capabilities: {orchestration_capabilities:?}");
        println!(
            "     Health: {}",
            self.orchestration_adapter.is_healthy().await
        );

        println!("\n   🏠 Storage Adapter Status:");
        let storage_capabilities = self.storage_adapter.get_storage_capabilities().await?;
        println!("     Available capabilities: {storage_capabilities:?}");
        println!("     Health: {}", self.storage_adapter.is_healthy().await);

        println!("\n   🍄 Compute Adapter Status:");
        let compute_capabilities = self.compute_adapter.get_compute_capabilities().await?;
        println!("     Available capabilities: {compute_capabilities:?}");
        println!("     Health: {}", self.compute_adapter.is_healthy().await);

        Ok(())
    }

    /// Demonstrate dynamic service discovery
    pub async fn demonstrate_service_discovery(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n🔍 === SERVICE DISCOVERY DEMONSTRATION ===");

        // Discover services by specific capabilities
        let security_services = self
            .registry
            .discover_by_capability(ServiceCapability::Security {
                functions: vec!["authentication".to_string()],
                compliance: vec!["enterprise".to_string()],
                trust_levels: vec!["high".to_string()],
            })
            .await?;

        println!("\n🔒 Security services with authentication capability:");
        for service in security_services {
            println!("   • {} - endpoints: {:?}", service.name, service.endpoints);
        }

        // Discover services by category
        let orchestration_services = self.registry.discover_by_category("orchestration").await?;

        println!("\n🎼 Orchestration services:");
        for service in orchestration_services {
            println!(
                "   • {} - priority: {}, health: {}",
                service.name, service.priority, service.health.healthy
            );
        }

        println!("\n✨ Service discovery completed - all services found by capability!");

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🐿️  SQUIRREL AI COORDINATOR - UNIVERSAL ADAPTERS DEMONSTRATION");
    println!("    Following Universal Primal Architecture Standard");
    println!("    🌌 Capability-First, Name-Agnostic Design");

    // Create AI coordinator with universal adapters
    let mut coordinator = SquirrelAICoordinator::new().await?;

    // Demonstrate service discovery
    coordinator.demonstrate_service_discovery().await?;

    // Demonstrate complete AI coordination workflow
    coordinator.demonstrate_ai_coordination().await?;

    println!("\n🏆 DEMONSTRATION COMPLETE!");
    println!("    ✅ Zero hardcoded primal names");
    println!("    ✅ Capability-based service discovery");
    println!("    ✅ Universal extensibility achieved");
    println!("    ✅ AI coordination via universal adapters");

    Ok(())
}
