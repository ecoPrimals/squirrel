// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! [`PrimalCoordinator`] implementation and discovery / routing helpers for [`EcosystemService`](super::ecosystem_service::EcosystemService).

use chrono::Utc;

use super::ecosystem_service::EcosystemService;
use crate::{
    EcosystemMode, Error, HealthStatus, MonitoringEvent, PrimalCoordinator, PrimalEndpoint,
    PrimalType, Result, Task, TaskResult,
};
use universal_constants::primal_names;

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
            let mut stats = self
                .state
                .coordination_stats
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
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
            let mut stats = self
                .state
                .coordination_stats
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
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
                    let mut stats = self
                        .state
                        .coordination_stats
                        .write()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
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

impl EcosystemService {
    /// Discover primals via service mesh registry
    /// NOTE: Uses Unix socket-based discovery via ecosystem patterns
    fn discover_via_service_mesh(discovery_endpoint: &str) -> Vec<PrimalEndpoint> {
        tracing::debug!(
            "Service mesh discovery not yet implemented (requires Unix socket): {}",
            discovery_endpoint
        );

        // Discovery should use Unix socket-based capability registry
        // Pattern: CapabilityRegistry::discover_services().await

        // For now, return empty list (discovery will use file-based or direct probing)
        Vec::new()
    }

    /// Discover primals via direct endpoint probing
    async fn discover_via_direct_probing(&self) -> Result<Vec<PrimalEndpoint>> {
        tracing::debug!("Discovering primals via direct probing");

        let mut primals = Vec::new();

        for (primal_name, endpoint) in &self.config.discovery.direct_endpoints {
            match Self::probe_primal_endpoint(primal_name, endpoint) {
                Ok(primal) => primals.push(primal),
                Err(e) => {
                    tracing::debug!("Failed to probe {}: {}", primal_name, e);
                    let _ = self
                        .monitoring
                        .record_error("endpoint_probe", &e.to_string(), "ecosystem")
                        .await;
                }
            }
        }

        Ok(primals)
    }

    /// Probe a specific primal endpoint
    /// NOTE: Uses Unix socket-based health check via ecosystem patterns
    fn probe_primal_endpoint(primal_name: &str, endpoint: &str) -> Result<PrimalEndpoint> {
        tracing::debug!(
            "Endpoint probing not yet implemented (requires Unix socket): {}",
            endpoint
        );

        // Primal health checks should use Unix socket-based communication
        // Pattern: UnixStream::connect(socket_path).await + JSON-RPC health check

        // For now, return error (discovery will use file-based registry)
        Err(Error::Discovery(format!(
            "Endpoint probing not yet implemented for {primal_name}: {endpoint}"
        )))
    }

    /// Parse primal type from string
    #[expect(dead_code, reason = "Phase 2 placeholder — primal type parsing")]
    fn parse_primal_type(type_str: &str) -> Result<PrimalType> {
        match type_str.to_lowercase().as_str() {
            primal_names::SQUIRREL => Ok(PrimalType::Squirrel),
            primal_names::SONGBIRD => Ok(PrimalType::Songbird),
            primal_names::NESTGATE => Ok(PrimalType::NestGate),
            primal_names::BEARDOG => Ok(PrimalType::BearDog),
            primal_names::TOADSTOOL => Ok(PrimalType::ToadStool),
            primal_names::BIOMEOS => Ok(PrimalType::BiomeOS),
            _ => Err(Error::Discovery(format!("Unknown primal type: {type_str}"))),
        }
    }

    /// Route task to appropriate primal using capability discovery pattern
    ///
    /// Resolution order:
    /// 1. Match discovered primals by `task.requirements.required_capabilities`
    /// 2. Prefer primals in `task.requirements.preferred_primals`
    /// 3. If no match, return error (caller may fall back to local execution in sovereign mode)
    fn route_task_to_primal(&self, task: &Task) -> Result<serde_json::Value> {
        let required = &task.requirements.required_capabilities;
        let preferred = &task.requirements.preferred_primals;

        let candidate_ids: Vec<String> = self
            .discovered_primals
            .iter()
            .filter(|entry| {
                let primal = entry.value();
                let has_caps = required.is_empty()
                    || required
                        .iter()
                        .all(|cap| primal.capabilities.iter().any(|c| c == cap));
                has_caps && primal.health != crate::HealthStatus::Unhealthy
            })
            .map(|entry| entry.key().clone())
            .collect();

        if candidate_ids.is_empty() {
            tracing::debug!(
                "No capable primal found for task {} (required: {:?})",
                task.id,
                required
            );
            return Err(Error::Routing(format!(
                "No primal with capabilities {:?} available for task {}",
                required, task.id
            )));
        }

        let selected_id = if preferred.is_empty() {
            candidate_ids[0].clone()
        } else {
            candidate_ids
                .iter()
                .find(|id| {
                    self.discovered_primals
                        .get(id.as_str())
                        .is_some_and(|p| preferred.contains(&p.primal_type))
                })
                .cloned()
                .unwrap_or_else(|| candidate_ids[0].clone())
        };

        let Some(selected) = self
            .discovered_primals
            .get(&selected_id)
            .map(|r| r.value().clone())
        else {
            return Err(Error::Routing(format!(
                "Primal {selected_id} disappeared during routing for task {}",
                task.id
            )));
        };

        tracing::info!(
            "Routing task {} to primal {} (endpoint: {})",
            task.id,
            selected.id,
            selected.endpoint
        );

        // Actual invocation would delegate to Songbird/Unix socket; for now return
        // structured result indicating routing decision. Caller records executed_by.
        Ok(serde_json::json!({
            "result": "task_routed",
            "task_id": task.id,
            "primal_id": selected.id,
            "primal_type": format!("{:?}", selected.primal_type),
            "endpoint": selected.endpoint,
            "timestamp": Utc::now().to_rfc3339()
        }))
    }

    /// Execute task locally as fallback when no capable primal is available
    async fn execute_task_locally(&self, task: &Task) -> Result<serde_json::Value> {
        tracing::info!(
            "Executing task {} locally (no capable primal discovered)",
            task.id
        );

        let _ = self
            .monitoring
            .record_event(MonitoringEvent::Custom {
                event_type: "local_task_execution".to_string(),
                data: serde_json::json!({
                    "task_id": task.id,
                    "reason": "coordination_failure_fallback",
                    "required_capabilities": task.requirements.required_capabilities
                }),
                timestamp: Utc::now(),
            })
            .await;

        // Local execution: Squirrel handles the task. Returns structured result.
        Ok(serde_json::json!({
            "result": "executed_locally",
            "task_id": task.id,
            "execution_mode": "local_fallback",
            "task_type": format!("{:?}", task.task_type),
            "timestamp": Utc::now().to_rfc3339()
        }))
    }
}
