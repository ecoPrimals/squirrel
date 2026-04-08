// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Identity domain JSON-RPC handlers

use serde_json::Value;
use tracing::debug;

use super::jsonrpc_server::{JsonRpcError, JsonRpcServer};

impl JsonRpcServer {
    /// Handle `identity.get` — Wire Standard L2 per CAPABILITY_WIRE_STANDARD v1.0.
    ///
    /// Fields per spec: `primal` (canonical name), `version`, `domain`, `license`.
    pub(crate) async fn handle_identity_get(&self) -> Result<Value, JsonRpcError> {
        debug!("identity.get (Wire Standard L2)");

        Ok(serde_json::json!({
            "primal": universal_constants::identity::PRIMAL_ID,
            "version": env!("CARGO_PKG_VERSION"),
            "domain": universal_constants::identity::PRIMAL_DOMAIN,
            "license": "AGPL-3.0-or-later",
        }))
    }
}
