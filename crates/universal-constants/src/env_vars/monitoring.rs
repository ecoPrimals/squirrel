// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Monitoring env vars

/// Monitoring enabled flag
pub const ENABLED: &str = "MONITORING_ENABLED";
/// Monitoring auth token
pub const AUTH_TOKEN: &str = "MONITORING_AUTH_TOKEN";
/// Monitoring batch size
pub const BATCH_SIZE: &str = "MONITORING_BATCH_SIZE";
/// Monitoring flush interval
pub const FLUSH_INTERVAL: &str = "MONITORING_FLUSH_INTERVAL";
/// Require monitoring provider
pub const REQUIRE_PROVIDER: &str = "MONITORING_REQUIRE_PROVIDER";
/// Metrics exporter endpoint
pub const METRICS_EXPORTER_ENDPOINT: &str = "METRICS_EXPORTER_ENDPOINT";
/// Metrics exporter port
pub const METRICS_EXPORTER_PORT: &str = "METRICS_EXPORTER_PORT";
/// Metrics port
pub const METRICS_PORT: &str = "METRICS_PORT";
/// Health check interval (seconds)
pub const HEALTH_CHECK_INTERVAL_SECS: &str = "HEALTH_CHECK_INTERVAL_SECS";
