// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Health diagnostics for Squirrel
//!
//! Comprehensive health checking system for all Squirrel subsystems.
//! Modern async implementation using Tokio.

mod checks;

use anyhow::{Context, Result};
use serde::Serialize;
use std::time::{Duration, Instant};

use crate::cli::{OutputFormat, Subsystem};

/// Health check result for a single subsystem
#[derive(Debug, Serialize)]
pub struct HealthCheck {
    pub name: &'static str,
    pub status: HealthStatus,
    pub message: String,
    pub duration_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Health status levels
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// System is healthy
    Ok,
    /// System has warnings but is functional
    Warning,
    /// System has errors (used in match arms, JSON deserialization, and test fixtures)
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "Constructed via serde/deserialization; unit tests construct this variant"
        )
    )]
    Error,
}

/// Complete health report
#[derive(Debug, Serialize)]
pub struct HealthReport {
    pub version: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub overall_status: HealthStatus,
    pub checks: Vec<HealthCheck>,
    pub recommendations: Vec<String>,
}

/// Run health diagnostics
pub fn run_doctor(
    comprehensive: bool,
    format: OutputFormat,
    subsystem: Option<Subsystem>,
) -> Result<()> {
    let start = Instant::now();

    // Print header (text mode only)
    if matches!(format, OutputFormat::Text) {
        println!(
            "🐿️  Squirrel v{} - Health Diagnostics",
            env!("CARGO_PKG_VERSION")
        );
        println!();
    }

    // Run checks based on subsystem filter
    let mut checks = Vec::new();

    if should_check(subsystem, Subsystem::Config) {
        checks.push(checks::check_binary());
        checks.push(checks::check_configuration());
    }

    if should_check(subsystem, Subsystem::Ai) {
        checks.push(checks::check_ai_providers(comprehensive));
    }

    if should_check(subsystem, Subsystem::Ecosystem) && comprehensive {
        checks.push(checks::check_discovered_services());
    }

    if should_check(subsystem, Subsystem::Socket) {
        checks.push(checks::check_unix_socket());
    }

    if should_check(subsystem, Subsystem::Rpc) {
        checks.push(checks::check_rpc_server());
    }

    // Determine overall status
    let overall_status = checks
        .iter()
        .map(|c| c.status)
        .max_by_key(|s| match s {
            HealthStatus::Error => 2,
            HealthStatus::Warning => 1,
            HealthStatus::Ok => 0,
        })
        .unwrap_or(HealthStatus::Ok);

    // Generate recommendations
    let recommendations = generate_recommendations(&checks);

    // Create report
    let report = HealthReport {
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now(),
        overall_status,
        checks,
        recommendations,
    };

    // Output report
    match format {
        OutputFormat::Text => print_text_report(&report, start.elapsed()),
        OutputFormat::Json => print_json_report(&report)?,
    }

    // Exit with appropriate code
    match overall_status {
        HealthStatus::Ok | HealthStatus::Warning => Ok(()), // Warnings don't fail
        HealthStatus::Error => anyhow::bail!("Health check failed"),
    }
}

/// Check if we should run a specific subsystem check
fn should_check(filter: Option<Subsystem>, target: Subsystem) -> bool {
    match filter {
        None => true,
        Some(f) => std::mem::discriminant(&f) == std::mem::discriminant(&target),
    }
}

/// Generate recommendations based on check results
fn generate_recommendations(checks: &[HealthCheck]) -> Vec<String> {
    let mut recommendations = Vec::new();

    // Check for AI provider warnings
    if checks
        .iter()
        .any(|c| c.name == "AI Providers" && c.status == HealthStatus::Warning)
    {
        recommendations.push(
            "Store AI keys in the security provider (secrets.store), configure AI_PROVIDER_SOCKETS, or set OPENAI_API_KEY env var".to_string(),
        );
    }

    // Check for ecosystem service warnings (capability-based discovery)
    // Doctor discovers primals at runtime from registry/sockets, not hardcoded names
    if checks
        .iter()
        .any(|c| c.name == "Ecosystem Services" && c.status == HealthStatus::Warning)
    {
        recommendations.push(
            "Start ecosystem registry (service mesh) for coordination (optional for development)"
                .to_string(),
        );
        recommendations.push(
            "Start security provider for auth/crypto features (optional for development)"
                .to_string(),
        );
    }

    if recommendations.is_empty() {
        recommendations.push("All systems operational! 🎉".to_string());
    }

    recommendations
}

/// Print text format report
fn print_text_report(report: &HealthReport, duration: Duration) {
    // Print checks
    for check in &report.checks {
        let icon = match check.status {
            HealthStatus::Ok => "✅",
            HealthStatus::Warning => "⚠️ ",
            HealthStatus::Error => "❌",
        };
        println!("{icon} {}: {}", check.name, check.message);
    }

    println!();

    // Print recommendations
    println!("RECOMMENDATIONS:");
    for rec in &report.recommendations {
        println!("  • {rec}");
    }

    println!();

    // Print summary
    let status_icon = match report.overall_status {
        HealthStatus::Ok => "✅",
        HealthStatus::Warning => "⚠️ ",
        HealthStatus::Error => "❌",
    };
    println!(
        "{status_icon} Overall Status: {:?} (completed in {:.2}s)",
        report.overall_status,
        duration.as_secs_f64()
    );
}

/// Print JSON format report
fn print_json_report(report: &HealthReport) -> Result<()> {
    let json = serde_json::to_string_pretty(report)
        .context("Failed to serialize health report to JSON")?;
    println!("{json}");
    Ok(())
}

#[cfg(test)]
mod doctor_tests;
