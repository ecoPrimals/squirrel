// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Identity domain JSON-RPC handlers

use serde_json::Value;
use tracing::debug;

use super::jsonrpc_server::{JsonRpcError, JsonRpcServer};

impl JsonRpcServer {
    /// Handle `identity.get` — returns primal self-knowledge per CAPABILITY_BASED_DISCOVERY_STANDARD v1.0
    pub(crate) async fn handle_identity_get(&self) -> Result<Value, JsonRpcError> {
        debug!("identity.get");

        let response = serde_json::json!({
            "primal_id": universal_constants::identity::PRIMAL_ID,
            "domain": universal_constants::identity::PRIMAL_DOMAIN,
            "version": env!("CARGO_PKG_VERSION"),
            "transport": "unix-socket",
            "protocol": "json-rpc-2.0",
            "license": "AGPL-3.0-or-later",
            "jwt_issuer": universal_constants::identity::JWT_ISSUER,
            "jwt_audience": universal_constants::identity::JWT_AUDIENCE,
        });

        Ok(response)
    }
}
