// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Environment Variable Names — Single Source of Truth
//!
//! All environment variable names used throughout the Squirrel system.
//! Using named constants prevents typos, enables refactoring, and makes
//! it trivial to audit which env vars the system reads.
//!
//! # Organization
//!
//! Variables are organized by domain. Each domain module re-exports its
//! constants so consumers can use either:
//! ```ignore
//! use universal_constants::env_vars::squirrel::SOCKET;
//! // or via the flat re-export:
//! use universal_constants::env_vars;

pub mod ai;
pub mod btsp;
pub mod compute;
pub mod database;
pub mod deploy;
pub mod discovery;
pub mod ecosystem;
pub mod federation;
pub mod flags;
pub mod http;
pub mod ipc;
pub mod limits;
pub mod logging;
pub mod mcp;
pub mod monitoring;
pub mod network;
pub mod performance;
pub mod primal;
pub mod primals;
pub mod sandbox;
pub mod security;
pub mod session;
pub mod squirrel;
pub mod storage;
pub mod sys;
pub mod task;
pub mod timeout;

// Backward-compatible flat re-exports (old env_vars::BIND_ADDRESS style)
// ============================================================================

pub use limits::{BUFFER_SIZE, MAX_MESSAGE_SIZE, SERVICE_MESH_MAX_SERVICES};
pub use logging::RUST_LOG as LOG_LEVEL;
pub use network::{
    ADMIN_PORT, BIND_ADDRESS, HTTP_PORT, MAX_CONNECTIONS, METRICS_PORT, WEBSOCKET_PORT,
};
pub use timeout::{
    CONNECTION as CONNECTION_TIMEOUT, DATABASE as DATABASE_TIMEOUT, HEARTBEAT_INTERVAL,
    INITIAL_DELAY, OPERATION as OPERATION_TIMEOUT, REQUEST as REQUEST_TIMEOUT,
};

/// Ecosystem registration URL (capability-first; legacy `BIOMEOS_REGISTRATION_URL` read as fallback)
pub const ECOSYSTEM_REGISTRATION_URL: &str = "ECOSYSTEM_REGISTRATION_URL";
/// Ecosystem health URL (capability-first; legacy `BIOMEOS_HEALTH_URL` read as fallback)
pub const ECOSYSTEM_HEALTH_URL: &str = "ECOSYSTEM_HEALTH_URL";
/// Ecosystem metrics URL (capability-first; legacy `BIOMEOS_METRICS_URL` read as fallback)
pub const ECOSYSTEM_METRICS_URL: &str = "ECOSYSTEM_METRICS_URL";

/// Debug mode (flat re-export for backward compat)
pub const DEBUG_MODE: &str = "SQUIRREL_DEBUG";
/// Verbose logging (flat re-export for backward compat)
pub const VERBOSE_LOGGING: &str = "SQUIRREL_VERBOSE";

#[cfg(test)]
#[path = "../env_vars_tests.rs"]
mod tests;
