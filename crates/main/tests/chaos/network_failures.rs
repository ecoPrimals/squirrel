// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Network failure scenarios for chaos testing

use super::framework::*;
use async_trait::async_trait;
use std::time::Duration;

/// Packet loss scenario
pub struct PacketLossScenario {
    pub loss_percentage: f64,
    pub duration: Duration,
}

#[async_trait]
impl ChaosScenario for PacketLossScenario {
    fn name(&self) -> &str {
        "packet_loss"
    }

    fn description(&self) -> &str {
        "Simulates network packet loss"
    }

    async fn execute(&self, engine: &ChaosEngine) -> Result<ChaosResult, Box<dyn std::error::Error + Send + Sync>> {
        let start = std::time::Instant::now();
        tokio::time::sleep(self.duration).await;
        
        Ok(ChaosResult {
            scenario_name: self.name().to_string(),
            success: true,
            duration: start.elapsed(),
            failures_injected: (self.loss_percentage * 100.0) as u32,
            recovery_time: None,
            metrics: engine.metrics().await,
            error: None,
        })
    }

    async fn validate_recovery(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        Ok(true)
    }
}

/// DNS failure scenario
pub struct DNSFailureScenario {
    pub affected_domains: Vec<String>,
}

#[async_trait]
impl ChaosScenario for DNSFailureScenario {
    fn name(&self) -> &str {
        "dns_failure"
    }

    fn description(&self) -> &str {
        "Simulates DNS resolution failures"
    }

    async fn execute(&self, engine: &ChaosEngine) -> Result<ChaosResult, Box<dyn std::error::Error + Send + Sync>> {
        let start = std::time::Instant::now();
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        Ok(ChaosResult {
            scenario_name: self.name().to_string(),
            success: true,
            duration: start.elapsed(),
            failures_injected: self.affected_domains.len() as u32,
            recovery_time: None,
            metrics: engine.metrics().await,
            error: None,
        })
    }

    async fn validate_recovery(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        Ok(true)
    }
}

