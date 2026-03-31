// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! In-memory registry state for registrations and discovery cache.

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;

use super::discovered::DiscoveredService;

/// Registry state tracking with ``Arc<str>`` optimization
#[derive(Debug, Default)]
pub struct RegistryState {
    /// Service registrations with `Arc<str>` keys for zero-copy performance
    pub registered_services: HashMap<Arc<str>, Arc<crate::ecosystem::EcosystemServiceRegistration>>,
    /// Service discovery cache with `Arc<str>` keys and `Arc<DiscoveredService>` values
    pub service_discovery_cache: HashMap<Arc<str>, Arc<DiscoveredService>>,
    /// Timestamp of last discovery sync
    pub last_discovery_sync: Option<DateTime<Utc>>,
    /// Number of registration attempts made
    pub registration_attempts: u32,
}
