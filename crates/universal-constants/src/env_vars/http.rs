// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! HTTP env vars

/// Default timeout (ms)
pub const DEFAULT_TIMEOUT_MS: &str = "HTTP_DEFAULT_TIMEOUT_MS";
/// Max redirects
pub const MAX_REDIRECTS: &str = "HTTP_MAX_REDIRECTS";
/// Max request size
pub const MAX_REQUEST_SIZE: &str = "HTTP_MAX_REQUEST_SIZE";
/// Max response size
pub const MAX_RESPONSE_SIZE: &str = "HTTP_MAX_RESPONSE_SIZE";
/// User agent
pub const USER_AGENT: &str = "HTTP_USER_AGENT";
/// HTTP capability socket
pub const CAPABILITY_SOCKET: &str = "HTTP_CAPABILITY_SOCKET";
/// Web UI URL override
pub const WEB_UI_URL: &str = "WEB_UI_URL";
/// Web UI port override
pub const WEB_UI_PORT: &str = "WEB_UI_PORT";
