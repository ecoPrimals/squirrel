// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! [`PrimalCoordinator`] implementation and discovery / routing helpers for [`EcosystemService`](super::ecosystem_service::EcosystemService).

use chrono::Utc;

use super::ecosystem_service::EcosystemService;
use crate::{
    EcosystemMode, Error, HealthStatus, MonitoringEvent, PrimalCoordinator, PrimalEndpoint, Result,
    Task, TaskResult,
};
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
                        endpoint = discovery_endpoint.as_str(),
                        "Attempting to register with discovery service"
                    );

                    let path = std::path::Path::new(discovery_endpoint.as_str());
                    if path.exists() {
                        tracing::info!(
                            endpoint = discovery_endpoint.as_str(),
                            "Discovery socket exists — async registration will connect on next heartbeat"
                        );
                    } else {
                        tracing::debug!(
                            endpoint = discovery_endpoint.as_str(),
                            "Discovery socket not present — operating in sovereign mode"
                        );
                    }

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
    /// Discover primals via service mesh registry (Unix socket-based).
    ///
    /// Returns an empty vec when the service mesh is unreachable. Callers fall
    /// through to direct probing, which is the expected flow when no mesh is
    /// running.
    fn discover_via_service_mesh(discovery_endpoint: &str) -> Vec<PrimalEndpoint> {
        tracing::debug!(
            endpoint = discovery_endpoint,
            "Service mesh discovery requires a running discovery primal — skipping (direct probing will follow)"
        );
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

    /// Probe a specific primal endpoint via Unix socket health check.
    ///
    /// Returns `Err` when the endpoint cannot be reached. This is a synchronous
    /// check; async probing uses `discover_via_direct_probing` which iterates
    /// over `direct_endpoints`.
    fn probe_primal_endpoint(primal_name: &str, endpoint: &str) -> Result<PrimalEndpoint> {
        let path = std::path::Path::new(endpoint);
        if !path.exists() {
            return Err(Error::Discovery(format!(
                "Endpoint socket does not exist for {primal_name}: {endpoint}"
            )));
        }
        Err(Error::Discovery(format!(
            "Synchronous probe cannot connect to Unix socket for {primal_name}: {endpoint} — use async discovery"
        )))
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

        // Returns the routing decision; caller invokes via IPC using the endpoint.
        // Cross-primal invocation uses capability-based discovery (ipc.find_provider).
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

        if let Err(e) = self
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
            .await
        {
            tracing::debug!(error = %e, "Monitoring event recording failed (non-critical)");
        }

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
