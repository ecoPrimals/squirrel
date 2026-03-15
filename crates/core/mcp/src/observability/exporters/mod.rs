// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! # Observability Exporters
//! 
//! This module contains various exporters for observability data, including:
//! - Dashboard exporter for visualizing traces in the dashboard
//! - Dashboard integration for connecting to dashboard-core
//! - (Future exporters can be added here)

// Re-export the dashboard exporter
pub mod dashboard_exporter;

// Re-export the dashboard integration
pub mod dashboard_integration;

// Re-export key types and functions
pub use dashboard_exporter::{
    DashboardExporter, DashboardExporterConfig, create_dashboard_exporter
};

// Re-export integration types and functions
pub use dashboard_integration::{
    DashboardIntegrationAdapter, DashboardIntegrationConfig, create_default_dashboard_integration
}; 