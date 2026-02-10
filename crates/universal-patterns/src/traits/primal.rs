// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Core Primal trait definition.

use async_trait::async_trait;
use std::collections::HashMap;

use crate::config::PrimalConfig;

use super::{HealthStatus, MetricValue, PrimalError, PrimalInfo, PrimalState};

/// Core primal trait - foundational interface for all primals
#[async_trait]
pub trait Primal: Send + Sync {
    /// Get primal information
    fn info(&self) -> &PrimalInfo;

    /// Get current state of the primal
    async fn state(&self) -> PrimalState;

    /// Start the primal
    async fn start(&mut self) -> Result<(), PrimalError>;

    /// Stop the primal
    async fn stop(&mut self) -> Result<(), PrimalError>;

    /// Restart the primal
    async fn restart(&mut self) -> Result<(), PrimalError> {
        self.stop().await?;
        self.start().await?;
        Ok(())
    }

    /// Check if the primal is healthy
    async fn health_check(&self) -> Result<HealthStatus, PrimalError>;

    /// Get configuration
    fn config(&self) -> &PrimalConfig;

    /// Update configuration
    async fn update_config(&mut self, config: PrimalConfig) -> Result<(), PrimalError>;

    /// Get metrics
    async fn metrics(&self) -> Result<HashMap<String, MetricValue>, PrimalError>;

    /// Handle shutdown signal
    async fn shutdown(&mut self) -> Result<(), PrimalError>;
}
