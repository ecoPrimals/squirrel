// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! MCP client-specific env vars

/// Client host
pub const HOST: &str = "MCP_CLIENT_HOST";
/// Client port
pub const PORT: &str = "MCP_CLIENT_PORT";
/// Client connect timeout (seconds)
pub const CONNECT_TIMEOUT_SECS: &str = "MCP_CLIENT_CONNECT_TIMEOUT_SECS";
/// Client request timeout (seconds)
pub const REQUEST_TIMEOUT_SECS: &str = "MCP_CLIENT_REQUEST_TIMEOUT_SECS";
/// Client max retries
pub const MAX_RETRIES: &str = "MCP_CLIENT_MAX_RETRIES";
