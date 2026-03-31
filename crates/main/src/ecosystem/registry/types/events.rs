// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Ecosystem registry events emitted during discovery, registration, and health transitions.

use std::sync::Arc;

use crate::ecosystem::EcosystemPrimalType;

use super::health::ServiceHealthStatus;

/// Ecosystem registry events with ``Arc<str>`` optimization for efficient event sharing
#[derive(Debug, Clone)]
pub enum EcosystemRegistryEvent {
    /// Service discovered in the ecosystem
    ServiceDiscovered {
        /// Discovered service ID
        service_id: Arc<str>,
        /// Type of primal
        primal_type: crate::EcosystemPrimalType,
        /// Service endpoint
        endpoint: Arc<str>,
        /// Service capabilities
        capabilities: Vec<Arc<str>>,
    },

    /// Service registered with ecosystem
    ServiceRegistered {
        /// Registered service ID
        service_id: Arc<str>,
        /// Type of primal
        primal_type: crate::EcosystemPrimalType,
        /// Service endpoint
        endpoint: Arc<str>,
    },

    /// Service error occurred
    ServiceError {
        /// Service ID where error occurred
        service_id: Arc<str>,
        /// Error message
        error: Arc<str>,
        /// When the error occurred
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Service health status changed
    ServiceHealthChanged {
        /// Service ID
        service_id: Arc<str>,
        /// Type of primal
        primal_type: EcosystemPrimalType,
        /// Previous health status
        old_status: ServiceHealthStatus,
        /// New health status
        new_status: ServiceHealthStatus,
    },
    /// Service went offline
    ServiceOffline {
        /// Service ID
        service_id: Arc<str>,
        /// Type of primal
        primal_type: EcosystemPrimalType,
        /// Reason for going offline
        reason: Arc<str>,
    },
}
