// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Types and small helpers for [`super::service::FederationService`].

use chrono::{DateTime, Utc};
use std::sync::{Arc, RwLock};

use crate::{Error, FederationStatus};

/// Local federation code path requires a capability that must be discovered on another primal via IPC.
pub(in crate::federation) fn capability_unavailable_federation(
    capability: &str,
    operation: &str,
) -> Error {
    let hint = format!(
        "This primal does not embed `{capability}`. Discover a peer that advertises it through the IPC capability registry (e.g. HTTP delegation to a network primal, often via `http.client`). Operation: {operation}"
    );
    tracing::warn!(
        capability = %capability,
        operation = %operation,
        "Federation: capability not satisfied locally; use IPC discovery to find a provider"
    );
    Error::CapabilityUnavailable {
        capability: capability.to_string(),
        hint,
    }
}

#[derive(Debug)]
pub(in crate::federation) struct FederationState {
    pub(in crate::federation) status: RwLock<FederationStatus>,
    pub(in crate::federation) federation_id: Arc<str>,
    pub(in crate::federation) leader_node: RwLock<Option<Arc<str>>>,
    pub(in crate::federation) last_scale_event: RwLock<Option<DateTime<Utc>>>,
    pub(in crate::federation) total_capacity: RwLock<u32>,
    pub(in crate::federation) current_utilization: RwLock<f64>,
}
