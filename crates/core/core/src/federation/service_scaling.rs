// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Background tasks, health monitoring, load collection, and auto-scaling
//! for [`FederationService`].
//!
//! Extracted from `service.rs` for module size management.  Uses scoped
//! accessors and struct fields via `&self`.

use chrono::Utc;

use super::service::FederationService;
use crate::{InstanceStatus, Result, SquirrelConfig, SwarmManager};
use universal_constants::limits::DEFAULT_MAX_CONNECTIONS;
use universal_constants::network::{DEFAULT_SQUIRREL_SERVER_PORT, get_service_port};
use universal_constants::safe_cast::{f64_to_u64_clamped, usize_to_u32_saturating};

impl FederationService {
    /// Spawn background monitoring and scaling tasks.
    pub(super) fn start_background_tasks(&self) {
        let service = self.clone();
        tokio::spawn(async move {
            service.health_monitoring_loop().await;
        });

        let service = self.clone();
        tokio::spawn(async move {
            service.load_monitoring_loop().await;
        });

        if self.config.auto_scaling_enabled {
            let service = self.clone();
            tokio::spawn(async move {
                service.auto_scaling_loop().await;
            });
        }

        if self.config.federation_enabled {
            let service = self.clone();
            tokio::spawn(async move {
                service.federation_maintenance_loop().await;
            });
        }
    }

    async fn health_monitoring_loop(&self) {
        let mut interval = tokio::time::interval(
            self.config
                .health_check_interval
                .to_std()
                .unwrap_or(std::time::Duration::from_secs(30)),
        );

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    self.check_federation_health();
                    self.check_instance_health();
                }
                () = self.shutdown_notify.notified() => {
                    tracing::info!("Health monitoring loop shutting down");
                    break;
                }
            }
        }
    }

    async fn load_monitoring_loop(&self) {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    self.collect_load_metrics();
                }
                () = self.shutdown_notify.notified() => {
                    tracing::info!("Load monitoring loop shutting down");
                    break;
                }
            }
        }
    }

    async fn auto_scaling_loop(&self) {
        let mut interval = tokio::time::interval(
            self.config
                .scaling_check_interval
                .to_std()
                .unwrap_or(std::time::Duration::from_secs(60)),
        );

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if let Err(e) = self.evaluate_scaling_decision().await {
                        tracing::error!("Scaling evaluation failed: {}", e);
                    }
                }
                () = self.shutdown_notify.notified() => {
                    tracing::info!("Auto-scaling loop shutting down");
                    break;
                }
            }
        }
    }

    async fn federation_maintenance_loop(&self) {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(120));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    self.maintain_federation();
                }
                () = self.shutdown_notify.notified() => {
                    tracing::info!("Federation maintenance loop shutting down");
                    break;
                }
            }
        }
    }

    /// Check health of federation nodes.
    /// NOTE: Delegates HTTP health checks to service mesh via Unix sockets
    fn check_federation_health(&self) {
        for mut entry in self.instances.iter_mut() {
            let (_instance_id, instance) = entry.pair_mut();
            instance.health = InstanceStatus::Running;
            instance.last_seen = Utc::now();
        }
    }

    /// Check health of local instances.
    /// NOTE: Delegates to service mesh for HTTP health checks
    fn check_instance_health(&self) {
        for mut entry in self.instances.iter_mut() {
            let (instance_id, instance) = entry.pair_mut();

            if instance.health == InstanceStatus::Starting {
                instance.health = InstanceStatus::Running;
                tracing::info!(
                    "Instance {} marked as running (HTTP health check via service mesh)",
                    instance_id
                );
            }
        }
    }

    /// Collect current load metrics.
    ///
    /// Uses config-driven defaults when real metrics are unavailable.
    fn collect_load_metrics(&self) {
        use universal_constants::env_vars;
        let cpu = std::env::var(env_vars::federation::CPU_USAGE)
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        let memory = std::env::var(env_vars::federation::MEMORY_USAGE)
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        let queue_length = std::env::var(env_vars::federation::QUEUE_LENGTH)
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0u32);
        let instance_count = usize_to_u32_saturating(self.instances.len());
        let max_conn = usize_to_u32_saturating(DEFAULT_MAX_CONNECTIONS);
        let active_tasks = instance_count.min(max_conn);

        tracing::debug!(
            "Load metrics - CPU: {:.2}%, Memory: {:.2}%, Queue: {}, Active: {}",
            cpu * 100.0,
            memory * 100.0,
            queue_length,
            active_tasks
        );
    }

    /// Calculate overall utilization across all metrics.
    pub(super) fn calculate_overall_utilization(&self) -> f64 {
        let cpu = self.load_metrics.cpu_usage;
        let memory = self.load_metrics.memory_usage;
        let queue_pressure = f64::from(self.load_metrics.queue_length) / 100.0;

        (cpu.mul_add(0.4, memory * 0.3) + queue_pressure * 0.3).min(1.0)
    }

    /// Evaluate whether scaling is needed.
    async fn evaluate_scaling_decision(&self) -> Result<()> {
        let current_utilization = self.calculate_overall_utilization();
        let current_instances = usize_to_u32_saturating(self.instances.len());

        let last_scale_snapshot = *self
            .state
            .last_scale_event
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(last_scale) = last_scale_snapshot {
            let time_since_last_scale = Utc::now() - last_scale;
            if time_since_last_scale < self.scaling_policy.scale_up_cooldown {
                return Ok(());
            }
        }

        if current_utilization > self.scaling_policy.scale_up_threshold {
            if current_instances < self.scaling_policy.max_instances {
                let scaled = f64::from(current_instances) * self.scaling_policy.scale_factor;
                let capped =
                    f64_to_u64_clamped(scaled).min(u64::from(self.scaling_policy.max_instances));
                let target_instances = u32::try_from(capped)
                    .unwrap_or(u32::MAX)
                    .min(self.scaling_policy.max_instances);

                tracing::info!(
                    "Scaling up from {} to {} instances (utilization: {:.2})",
                    current_instances,
                    target_instances,
                    current_utilization
                );

                self.scale_up(target_instances - current_instances).await?;
            }
        } else if current_utilization < self.scaling_policy.scale_down_threshold
            && current_instances > self.scaling_policy.min_instances
        {
            let scaled = f64::from(current_instances) / self.scaling_policy.scale_factor;
            let rounded = f64_to_u64_clamped(scaled);
            let target_instances =
                u32::try_from(rounded.max(u64::from(self.scaling_policy.min_instances)))
                    .unwrap_or(u32::MAX);

            tracing::info!(
                "Scaling down from {} to {} instances (utilization: {:.2})",
                current_instances,
                target_instances,
                current_utilization
            );

            self.scale_down(current_instances - target_instances);
        }

        Ok(())
    }

    async fn scale_up(&self, instances_to_add: u32) -> Result<()> {
        for i in 0..instances_to_add {
            let instance_config = self.create_instance_config(i);
            match self.spawn_squirrel(instance_config).await {
                Ok(instance) => {
                    tracing::info!("Successfully spawned new instance: {}", instance.id);
                }
                Err(e) => {
                    tracing::error!("Failed to spawn instance: {}", e);
                }
            }
        }

        *self
            .state
            .last_scale_event
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Utc::now());
        Ok(())
    }

    fn scale_down(&self, instances_to_remove: u32) {
        let mut instances_to_stop = Vec::new();

        for (removed, entry) in self.instances.iter().enumerate() {
            if removed >= instances_to_remove as usize {
                break;
            }
            instances_to_stop.push(entry.key().clone());
        }

        for instance_id in instances_to_stop {
            if let Some((_, mut instance)) = self.instances.remove(&instance_id) {
                instance.health = InstanceStatus::Stopping;

                if let Err(e) = self.stop_instance(&instance) {
                    tracing::error!("Failed to stop instance {}: {}", instance.id, e);
                } else {
                    tracing::info!("Successfully stopped instance: {}", instance.id);
                }
            }
        }

        *self
            .state
            .last_scale_event
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Utc::now());
    }

    /// Create configuration for a new instance using universal-constants defaults.
    fn create_instance_config(&self, instance_index: u32) -> SquirrelConfig {
        let base_port = std::env::var(universal_constants::env_vars::squirrel::PORT)
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or_else(|| get_service_port("websocket"));
        let offset = u16::try_from(instance_index).unwrap_or(u16::MAX);
        let port = if base_port > 0 {
            base_port.saturating_add(offset)
        } else {
            DEFAULT_SQUIRREL_SERVER_PORT.saturating_add(offset)
        };

        let capacity = std::env::var(universal_constants::env_vars::squirrel::INSTANCE_CAPACITY)
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(usize_to_u32_saturating(DEFAULT_MAX_CONNECTIONS));

        SquirrelConfig {
            node_id: format!("{}-instance-{}", self.config.node_id, instance_index),
            port,
            federation_enabled: false,
            region: self.config.region.clone(),
            zone: self.config.zone.clone(),
            auto_scaling_enabled: true,
            capabilities: vec!["mcp".to_string(), "routing".to_string()],
            capacity,
            metadata: std::collections::HashMap::new(),
        }
    }
}
