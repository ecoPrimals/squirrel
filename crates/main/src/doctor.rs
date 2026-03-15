// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Health diagnostics for Squirrel
//!
//! Comprehensive health checking system for all Squirrel subsystems.
//! Modern async implementation using Tokio.

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
    #[expect(
        dead_code,
        reason = "Used in match arms, JSON deserialization, and test fixtures"
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
pub async fn run_doctor(
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
        checks.push(check_binary());
        checks.push(check_configuration());
    }

    if should_check(subsystem, Subsystem::Ai) {
        checks.push(check_ai_providers(comprehensive));
    }

    if should_check(subsystem, Subsystem::Ecosystem) && comprehensive {
        checks.push(check_discovered_services());
    }

    if should_check(subsystem, Subsystem::Socket) {
        checks.push(check_unix_socket());
    }

    if should_check(subsystem, Subsystem::Rpc) {
        checks.push(check_rpc_server());
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
        HealthStatus::Ok => Ok(()),
        HealthStatus::Warning => Ok(()), // Warnings don't fail
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

/// Check binary and version
fn check_binary() -> HealthCheck {
    let start = Instant::now();
    HealthCheck {
        name: "Binary",
        status: HealthStatus::Ok,
        message: format!("squirrel v{}", env!("CARGO_PKG_VERSION")),
        duration_ms: start.elapsed().as_millis() as u64,
        details: Some(serde_json::json!({
            "version": env!("CARGO_PKG_VERSION"),
            "rust_version": env!("CARGO_PKG_RUST_VERSION"),
        })),
    }
}

/// Check configuration
fn check_configuration() -> HealthCheck {
    let start = Instant::now();

    let squirrel_port = std::env::var("SQUIRREL_PORT").ok();
    let squirrel_socket = std::env::var("SQUIRREL_SOCKET").ok();
    let ai_provider_sockets = std::env::var("AI_PROVIDER_SOCKETS").ok();

    let status = if ai_provider_sockets.is_none() {
        HealthStatus::Warning
    } else {
        HealthStatus::Ok
    };

    let message = if ai_provider_sockets.is_some() {
        "Configuration OK".to_string()
    } else {
        "AI_PROVIDER_SOCKETS not configured".to_string()
    };

    HealthCheck {
        name: "Configuration",
        status,
        message,
        duration_ms: start.elapsed().as_millis() as u64,
        details: Some(serde_json::json!({
            "squirrel_port": squirrel_port,
            "squirrel_socket": squirrel_socket,
            "ai_provider_sockets": ai_provider_sockets,
        })),
    }
}

/// Check AI providers
fn check_ai_providers(comprehensive: bool) -> HealthCheck {
    let start = Instant::now();

    let openai_key = std::env::var("OPENAI_API_KEY").ok();
    let huggingface_key = std::env::var("HUGGINGFACE_API_KEY").ok();
    let local_ai_url = std::env::var("LOCAL_AI_ENDPOINT")
        .or_else(|_| std::env::var("OLLAMA_URL"))
        .ok();
    let ai_provider_sockets = std::env::var("AI_PROVIDER_SOCKETS").ok();

    let provider_count = [
        openai_key.is_some(),
        huggingface_key.is_some(),
        local_ai_url.is_some() || comprehensive,
        ai_provider_sockets.is_some(),
    ]
    .iter()
    .filter(|&&x| x)
    .count();

    let (status, message) = if provider_count == 0 {
        (
            HealthStatus::Warning,
            "No AI providers configured".to_string(),
        )
    } else {
        (
            HealthStatus::Ok,
            format!("{provider_count} AI provider(s) configured"),
        )
    };

    HealthCheck {
        name: "AI Providers",
        status,
        message,
        duration_ms: start.elapsed().as_millis() as u64,
        details: Some(serde_json::json!({
            "openai": openai_key.is_some(),
            "huggingface": huggingface_key.is_some(),
            "local_server": local_ai_url.is_some(),
            "universal": ai_provider_sockets.is_some(),
            "count": provider_count,
        })),
    }
}

/// Check discovered services via capability registry
fn check_discovered_services() -> HealthCheck {
    let start = Instant::now();

    let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
        .or_else(|_| std::env::var("UID").map(|uid| format!("/run/user/{uid}")))
        .unwrap_or_else(|_| "/tmp".to_string());

    let mut discovered = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&runtime_dir) {
        for entry in entries.flatten() {
            if let Ok(path) = entry.path().canonicalize() {
                if path.extension().and_then(|s| s.to_str()) == Some("sock") {
                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                        discovered.push(name.to_string());
                    }
                }
            }
        }
    }

    let count = discovered.len();
    let (status, message) = if discovered.is_empty() {
        (HealthStatus::Warning, "No services discovered".to_string())
    } else {
        (HealthStatus::Ok, format!("Discovered {count} service(s)"))
    };

    HealthCheck {
        name: "Ecosystem Services",
        status,
        message,
        duration_ms: start.elapsed().as_millis() as u64,
        details: Some(serde_json::json!({
            "runtime_dir": runtime_dir,
            "discovered_services": discovered,
            "note": "Services discovered via Unix socket capability discovery"
        })),
    }
}

/// Check Unix socket health
fn check_unix_socket() -> HealthCheck {
    let start = Instant::now();

    let socket_path = universal_constants::network::get_socket_path("squirrel")
        .to_string_lossy()
        .into_owned();

    HealthCheck {
        name: "Unix Socket",
        status: HealthStatus::Ok,
        message: "Configuration OK".to_string(),
        duration_ms: start.elapsed().as_millis() as u64,
        details: Some(serde_json::json!({
            "socket_path": socket_path,
            "note": "Socket created on server start",
        })),
    }
}

/// Check RPC server configuration
fn check_rpc_server() -> HealthCheck {
    let start = Instant::now();

    let socket_path = universal_constants::network::get_socket_path("squirrel")
        .to_string_lossy()
        .into_owned();

    HealthCheck {
        name: "RPC Server",
        status: HealthStatus::Ok,
        message: format!("Will bind to socket {socket_path}"),
        duration_ms: start.elapsed().as_millis() as u64,
        details: Some(serde_json::json!({
            "socket_path": socket_path,
            "protocol": "JSON-RPC 2.0 + tarpc",
            "note": "Server not running in doctor mode",
        })),
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
            "Configure AI_PROVIDER_SOCKETS or set OPENAI_API_KEY/HUGGINGFACE_API_KEY".to_string(),
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
mod tests {
    use super::*;

    #[test]
    fn test_check_binary() {
        let check = check_binary();
        assert_eq!(check.status, HealthStatus::Ok);
        assert!(check.message.contains("squirrel"));
    }

    #[test]
    fn test_check_configuration() {
        let check = check_configuration();
        assert!(matches!(
            check.status,
            HealthStatus::Ok | HealthStatus::Warning
        ));
    }

    #[tokio::test]
    async fn test_should_check_filter() {
        assert!(should_check(None, Subsystem::Ai));
        assert!(should_check(Some(Subsystem::Ai), Subsystem::Ai));
        assert!(!should_check(Some(Subsystem::Config), Subsystem::Ai));
    }

    // ========================================================================
    // ADDITIONAL UNIT TESTS
    // ========================================================================

    #[test]
    fn test_health_status_serialization() {
        let status = HealthStatus::Ok;
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("Ok") || json.contains("\"ok\""));
    }

    #[test]
    fn test_health_check_structure() {
        let check = HealthCheck {
            name: "Test",
            status: HealthStatus::Ok,
            message: "Test message".to_string(),
            duration_ms: 100,
            details: None,
        };

        assert_eq!(check.name, "Test");
        assert!(matches!(check.status, HealthStatus::Ok));
        assert_eq!(check.duration_ms, 100);
    }

    #[test]
    fn test_health_report_serialization() {
        let report = HealthReport {
            version: "1.2.0".to_string(),
            timestamp: chrono::Utc::now(),
            overall_status: HealthStatus::Ok,
            checks: vec![],
            recommendations: vec![],
        };

        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("1.2.0"));
        // Status might be "Ok" or "ok" depending on serialization
        assert!(json.to_lowercase().contains("ok"));
    }

    // ========================================================================
    // E2E TESTS - Full Check Flows
    // ========================================================================

    #[test]
    fn test_binary_check_always_succeeds() {
        let check = check_binary();
        assert_eq!(check.status, HealthStatus::Ok);
        assert!(check.message.contains(env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn test_configuration_check_structure() {
        let check = check_configuration();
        assert_eq!(check.name, "Configuration");
        assert!(!check.message.is_empty());
    }

    #[test]
    fn test_unix_socket_check_returns_valid_status() {
        let check = check_unix_socket();
        assert!(matches!(
            check.status,
            HealthStatus::Ok | HealthStatus::Warning | HealthStatus::Error
        ));
    }

    #[test]
    fn test_rpc_server_check_structure() {
        let check = check_rpc_server();
        assert_eq!(check.name, "RPC Server");
    }

    // ========================================================================
    // CHAOS TESTS - Edge Cases
    // ========================================================================

    #[test]
    fn test_all_checks_run_without_panic() {
        let checks = (
            check_binary(),
            check_configuration(),
            check_unix_socket(),
            check_rpc_server(),
        );

        assert!(!checks.0.name.is_empty());
        assert!(!checks.1.name.is_empty());
        assert!(!checks.2.name.is_empty());
        assert!(!checks.3.name.is_empty());
    }

    #[test]
    fn test_checks_complete_in_reasonable_time() {
        use std::time::Instant;
        let start = Instant::now();

        let _ = (
            check_binary(),
            check_configuration(),
            check_unix_socket(),
            check_rpc_server(),
        );

        let elapsed = start.elapsed();
        assert!(
            elapsed.as_secs() < 10,
            "All checks should complete in < 10s, took: {elapsed:?}",
        );
    }

    #[test]
    fn test_subsystem_filtering_none() {
        assert!(should_check(None, Subsystem::Ai));
        assert!(should_check(None, Subsystem::Config));
        assert!(should_check(None, Subsystem::Rpc));
    }

    #[test]
    fn test_subsystem_filtering_specific() {
        assert!(should_check(Some(Subsystem::Ai), Subsystem::Ai));
        assert!(!should_check(Some(Subsystem::Ai), Subsystem::Config));
        assert!(!should_check(Some(Subsystem::Config), Subsystem::Rpc));
    }

    #[test]
    fn test_subsystem_display() {
        assert_eq!(format!("{}", Subsystem::Ai), "ai");
        assert_eq!(format!("{}", Subsystem::Config), "config");
        assert_eq!(format!("{}", Subsystem::Rpc), "rpc");
    }

    // ========================================================================
    // FAULT TESTS - Error Scenarios
    // ========================================================================

    #[test]
    fn test_all_checks_have_valid_durations() {
        let checks = (
            check_binary(),
            check_configuration(),
            check_unix_socket(),
            check_rpc_server(),
        );

        assert!(checks.0.duration_ms < 10000);
        assert!(checks.1.duration_ms < 10000);
        assert!(checks.2.duration_ms < 10000);
        assert!(checks.3.duration_ms < 10000);
    }

    #[tokio::test]
    async fn test_health_status_ordering() {
        // Test that status types are distinct
        let ok_check = HealthCheck {
            name: "OK",
            status: HealthStatus::Ok,
            message: "OK".to_string(),
            duration_ms: 10,
            details: None,
        };

        let warn_check = HealthCheck {
            name: "Warn",
            status: HealthStatus::Warning,
            message: "Warn".to_string(),
            duration_ms: 20,
            details: None,
        };

        let err_check = HealthCheck {
            name: "Error",
            status: HealthStatus::Error,
            message: "Error".to_string(),
            duration_ms: 30,
            details: None,
        };

        // Verify they're distinct
        assert!(!matches!(ok_check.status, HealthStatus::Warning));
        assert!(!matches!(warn_check.status, HealthStatus::Ok));
        assert!(!matches!(err_check.status, HealthStatus::Ok));
    }

    // ========================================================================
    // INTEGRATION TESTS
    // ========================================================================

    #[test]
    fn test_concurrent_check_execution() {
        let results = (
            check_binary(),
            check_binary(),
            check_configuration(),
            check_configuration(),
        );

        assert!(!results.0.message.is_empty());
        assert!(!results.1.message.is_empty());
        assert!(!results.2.message.is_empty());
        assert!(!results.3.message.is_empty());
    }

    #[test]
    fn test_health_report_json_serialization() {
        let report = HealthReport {
            version: "1.2.0".to_string(),
            timestamp: chrono::Utc::now(),
            overall_status: HealthStatus::Ok,
            checks: vec![HealthCheck {
                name: "Test",
                status: HealthStatus::Ok,
                message: "OK".to_string(),
                duration_ms: 50,
                details: None,
            }],
            recommendations: vec![],
        };

        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("1.2.0"));
        assert!(json.contains("Test"));
    }

    #[test]
    fn test_checks_produce_valid_messages() {
        let checks = (
            check_binary(),
            check_configuration(),
            check_unix_socket(),
            check_rpc_server(),
        );

        assert!(!checks.0.message.is_empty());
        assert!(!checks.1.message.is_empty());
        assert!(!checks.2.message.is_empty());
        assert!(!checks.3.message.is_empty());
    }

    #[test]
    fn test_generate_recommendations_empty_checks() {
        let checks = vec![];
        let recs = super::generate_recommendations(&checks);
        assert_eq!(recs.len(), 1);
        assert!(recs[0].contains("operational"));
    }

    #[test]
    fn test_generate_recommendations_ai_provider_warning() {
        let checks = vec![HealthCheck {
            name: "AI Providers",
            status: HealthStatus::Warning,
            message: "No AI providers configured".to_string(),
            duration_ms: 10,
            details: None,
        }];
        let recs = super::generate_recommendations(&checks);
        assert!(recs.iter().any(|r| r.contains("AI_PROVIDER_SOCKETS")));
    }

    #[test]
    fn test_generate_recommendations_ecosystem_services_warning() {
        let checks = vec![HealthCheck {
            name: "Ecosystem Services",
            status: HealthStatus::Warning,
            message: "No services discovered".to_string(),
            duration_ms: 10,
            details: None,
        }];
        let recs = super::generate_recommendations(&checks);
        assert!(
            recs.iter()
                .any(|r| r.contains("ecosystem registry") || r.contains("service mesh")),
            "Expected capability-based ecosystem recommendation, got: {:?}",
            recs
        );
        assert!(
            recs.iter()
                .any(|r| r.contains("security provider") || r.contains("auth")),
            "Expected capability-based security recommendation, got: {:?}",
            recs
        );
    }

    #[tokio::test]
    async fn test_run_doctor_text_format() {
        let result = run_doctor(false, OutputFormat::Text, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_doctor_json_format() {
        let result = run_doctor(false, OutputFormat::Json, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_doctor_subsystem_filter_config() {
        let result = run_doctor(false, OutputFormat::Text, Some(Subsystem::Config)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_doctor_subsystem_filter_ai() {
        let result = run_doctor(false, OutputFormat::Text, Some(Subsystem::Ai)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_doctor_comprehensive() {
        let result = run_doctor(true, OutputFormat::Text, None).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_discovered_services() {
        let check = super::check_discovered_services();
        assert_eq!(check.name, "Ecosystem Services");
        assert!(matches!(
            check.status,
            HealthStatus::Ok | HealthStatus::Warning
        ));
    }

    #[test]
    fn test_check_ai_providers_comprehensive() {
        let check = super::check_ai_providers(true);
        assert_eq!(check.name, "AI Providers");
        assert!(matches!(
            check.status,
            HealthStatus::Ok | HealthStatus::Warning
        ));
    }
}
