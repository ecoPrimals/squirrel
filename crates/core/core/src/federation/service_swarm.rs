// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! [`SwarmManager`] implementation for [`FederationService`].
//!
//! Extracted from `service.rs` for module size management.

use chrono::Utc;
use std::collections::HashMap;

use super::service::FederationService;
use crate::{
    FederationResult, FederationStatus, InstanceStatus, LoadBalanceResult, LoadMetrics, NodeSpec,
    Result, SquirrelConfig, SquirrelInstance, SwarmManager,
};

impl SwarmManager for FederationService {
    async fn spawn_squirrel(&self, config: SquirrelConfig) -> Result<SquirrelInstance> {
        let instance_id = uuid::Uuid::new_v4().to_string();
        let instance_port = config.port;

        tracing::info!("Spawning new Squirrel instance: {}", instance_id);

        let node_host = Self::resolve_node_host();
        let instance = SquirrelInstance {
            id: instance_id.clone(),
            node_id: self.config.node_id.clone(),
            endpoint: universal_constants::builders::build_http_url(&node_host, instance_port),
            capabilities: vec![
                "mcp".to_string(),
                "ai-task-routing".to_string(),
                "context-management".to_string(),
            ],
            health: InstanceStatus::Starting,
            last_seen: Utc::now(),
            capacity: config.capacity,
            current_load: 0,
            region: config.region.clone(),
            zone: config.zone.clone(),
            metadata: config.metadata,
        };

        tracing::info!("Successfully created instance: {}", instance_id);
        self.instances.insert(instance_id, instance.clone());
        Ok(instance)
    }

    async fn federate_nodes(&self, nodes: Vec<NodeSpec>) -> Result<FederationResult> {
        let mut nodes_joined = 0u32;
        let mut joined_capacity = 0u32;

        for node_spec in nodes {
            let cap = self.add_federation_node(node_spec);
            nodes_joined += 1;
            joined_capacity += cap;
        }

        let status = if nodes_joined == 0 {
            FederationStatus::Inactive
        } else {
            FederationStatus::Active
        };

        Ok(FederationResult {
            federation_id: (*self.state.federation_id).to_string(),
            nodes_joined,
            total_capacity: joined_capacity.max(1),
            status,
        })
    }

    async fn balance_load(&self, _load: LoadMetrics) -> Result<LoadBalanceResult> {
        let current_utilization = self.calculate_overall_utilization();
        *self.state.current_utilization.write() = current_utilization;

        let _action = if current_utilization > 0.8 {
            "scale_up".to_string()
        } else if current_utilization < 0.3 {
            "scale_down".to_string()
        } else {
            "maintain".to_string()
        };

        Ok(LoadBalanceResult {
            distribution: std::collections::HashMap::new(),
            balance_score: current_utilization,
            rebalance_time: std::time::Duration::from_millis(100),
        })
    }
}

impl FederationService {
    /// Add a new node to the federation
    pub(super) fn add_federation_node(&self, node_spec: NodeSpec) -> u32 {
        let cap = node_spec.capacity;
        let node = SquirrelInstance {
            id: node_spec.id.clone(),
            node_id: self.config.node_id.clone(),
            endpoint: node_spec.endpoint,
            capabilities: node_spec.capabilities,
            health: InstanceStatus::Running,
            last_seen: Utc::now(),
            capacity: node_spec.capacity,
            current_load: 0,
            region: node_spec.region,
            zone: node_spec.zone,
            metadata: HashMap::new(),
        };

        self.instances.insert(node_spec.id, node);
        cap
    }
}
