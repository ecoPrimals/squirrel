// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! [`PrimalCoordinator`] implementation for [`crate::ecosystem::EcosystemService`].

use chrono::Utc;

use crate::{
    EcosystemMode, HealthStatus, MonitoringEvent, PrimalCoordinator, PrimalEndpoint, Result, Task,
    TaskResult,
};
use crate::ecosystem::EcosystemService;

impl PrimalCoordinator for EcosystemService {
    async fn register_with_ecosystem(&self) -> Result<()> {
        self.config
            .discovery
            .discovery_endpoint
            .as_ref()
            .map_or_else(
                || {
                    tracing::debug!("No discovery endpoint configured, skipping registration");
                    Ok(())
                },
                |discovery_endpoint| {
                    tracing::info!(
                        "Attempting to register with discovery service at: {}",
                        discovery_endpoint
                    );

                    // NOTE: Registration uses Unix socket discovery via ecosystem patterns
                    // Pattern: Capability-based service registry via Unix sockets
                    tracing::info!(
                        "Discovery service registration not yet implemented (requires Unix socket discovery)"
                    );
                    tracing::debug!("Discovery endpoint: {}", discovery_endpoint);

                    // For now, succeed silently (registration will use file-based or Unix socket discovery)
                    Ok(())
                },
            )
    }

    async fn discover_primals(&self) -> Result<Vec<PrimalEndpoint>> {
        let mut discovered = Vec::new();

        // Try service mesh discovery first
        if let Some(ref discovery_endpoint) = self.config.discovery.discovery_endpoint {
            let mut primals = Self::discover_via_service_mesh(discovery_endpoint);
            discovered.append(&mut primals);
        }

        // Fallback to direct endpoint probing
        if discovered.is_empty() || self.config.discovery.auto_discovery {
            match self.discover_via_direct_probing().await {
                Ok(mut primals) => {
                    discovered.append(&mut primals);
                }
                Err(e) => {
                    tracing::debug!("Direct probing failed: {}", e);
                    let _ = self
                        .monitoring
                        .record_error("direct_discovery", &e.to_string(), "ecosystem")
                        .await;
                }
            }
        }

        let count = u32::try_from(discovered.len()).unwrap_or(u32::MAX);
        let returned = discovered.clone();

        // Record discovery events (borrowed pass), then move primals into the cache.
        for primal in &discovered {
            let _ = self
                .monitoring
                .record_event(MonitoringEvent::PrimalDiscovered {
                    primal_id: primal.id.clone(),
                    primal_type: format!("{:?}", primal.primal_type),
                    endpoint: primal.endpoint.clone(),
                    timestamp: Utc::now(),
                })
                .await;
        }
        for primal in discovered {
            let id = primal.id.clone();
            self.discovered_primals.insert(id, primal);
        }

        // Update stats
        {
            let mut stats = self.state.coordination_stats.write().unwrap_or_else(std::sync::PoisonError::into_inner);
            stats.primals_discovered = count;
        }

        tracing::debug!("Discovered {count} primals");
        Ok(returned)
    }

    async fn coordinate_task(&self, task: Task) -> Result<TaskResult> {
        let start_time = Utc::now();
        tracing::debug!("Coordinating task: {}", task.id);

        // Update coordination stats
        {
            let mut stats = self.state.coordination_stats.write().unwrap_or_else(std::sync::PoisonError::into_inner);
            stats.tasks_coordinated += 1;
            stats.last_coordination = Some(Utc::now());
        }

        // Record task submission event
        let _ = self
            .monitoring
            .record_event(MonitoringEvent::TaskSubmitted {
                task_id: task.id.clone(),
                task_type: format!("{:?}", task.task_type),
                priority: format!("{:?}", task.priority),
                timestamp: Utc::now(),
            })
            .await;

        // For now, this is a basic implementation
        // In a real system, this would route the task to appropriate primals
        // based on the task requirements and available capabilities

        let (result, executed_by) = match self.route_task_to_primal(&task) {
            Ok(result) => {
                let primal_id = result
                    .get("primal_id")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                (result, primal_id)
            }
            Err(e) => {
                // Update failure stats
                {
                    let mut stats = self.state.coordination_stats.write().unwrap_or_else(std::sync::PoisonError::into_inner);
                    stats.coordination_failures += 1;
                }

                let _ = self
                    .monitoring
                    .record_error("task_coordination", &e.to_string(), "ecosystem")
                    .await;

                // In sovereign mode, fall back to local execution
                if matches!(self.config.mode, EcosystemMode::Sovereign) {
                    tracing::warn!(
                        "Task coordination failed, falling back to local execution: {}",
                        e
                    );
                    let local_result = self.execute_task_locally(&task).await?;
                    (local_result, None)
                } else {
                    return Err(e);
                }
            }
        };

        let execution_time = (Utc::now() - start_time)
            .to_std()
            .unwrap_or(std::time::Duration::from_secs(0));

        // Record task completion event
        let _ = self
            .monitoring
            .record_task_completed(&task.id, execution_time, true)
            .await;

        Ok(TaskResult {
            id: task.id,
            status: crate::TaskStatus::Completed,
            result: Some(result),
            error: None,
            execution_time,
            executed_by,
        })
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        let health = self.get_current_health();
        let _ = self.monitoring.record_health("ecosystem", health).await;
        Ok(health)
    }
}
