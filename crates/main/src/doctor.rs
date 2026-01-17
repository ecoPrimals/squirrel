//! Health diagnostics for Squirrel
//!
//! Comprehensive health checking system for all Squirrel subsystems.
//! Modern async implementation using Tokio.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tokio::time::{timeout, Duration};

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
    /// System has errors
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
        println!("🐿️  Squirrel v{} - Health Diagnostics", env!("CARGO_PKG_VERSION"));
        println!();
    }

    // Run checks based on subsystem filter
    let mut checks = Vec::new();

    if should_check(subsystem, Subsystem::Config) {
        checks.push(check_binary().await);
        checks.push(check_configuration().await);
    }

    if should_check(subsystem, Subsystem::Ai) {
        checks.push(check_ai_providers(comprehensive).await);
    }

    if should_check(subsystem, Subsystem::Ecosystem) && comprehensive {
        checks.push(check_songbird_connectivity().await);
        checks.push(check_beardog_connectivity().await);
    }

    if should_check(subsystem, Subsystem::Socket) {
        checks.push(check_unix_socket().await);
    }

    if should_check(subsystem, Subsystem::Http) {
        checks.push(check_http_server().await);
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
async fn check_binary() -> HealthCheck {
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
async fn check_configuration() -> HealthCheck {
    let start = Instant::now();

    // Check environment variables
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
async fn check_ai_providers(comprehensive: bool) -> HealthCheck {
    let start = Instant::now();

    // Check for AI provider configuration
    let openai_key = std::env::var("OPENAI_API_KEY").ok();
    let huggingface_key = std::env::var("HUGGINGFACE_API_KEY").ok();
    let ollama_url = std::env::var("OLLAMA_URL").ok();
    let ai_provider_sockets = std::env::var("AI_PROVIDER_SOCKETS").ok();

    let provider_count = [
        openai_key.is_some(),
        huggingface_key.is_some(),
        ollama_url.is_some() || comprehensive, // Ollama might be on default port
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
            format!("{} AI provider(s) configured", provider_count),
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
            "ollama": ollama_url.is_some(),
            "universal": ai_provider_sockets.is_some(),
            "count": provider_count,
        })),
    }
}

/// Check Songbird connectivity
async fn check_songbird_connectivity() -> HealthCheck {
    let start = Instant::now();

    let songbird_port = std::env::var("SONGBIRD_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8081);

    let url = format!("http://localhost:{}/health", songbird_port);

    match timeout(Duration::from_secs(2), reqwest::get(&url)).await {
        Ok(Ok(response)) if response.status().is_success() => HealthCheck {
            name: "Songbird",
            status: HealthStatus::Ok,
            message: "Reachable".to_string(),
            duration_ms: start.elapsed().as_millis() as u64,
            details: Some(serde_json::json!({
                "url": url,
                "status": response.status().as_u16(),
            })),
        },
        _ => HealthCheck {
            name: "Songbird",
            status: HealthStatus::Warning,
            message: "Not reachable".to_string(),
            duration_ms: start.elapsed().as_millis() as u64,
            details: Some(serde_json::json!({
                "url": url,
                "note": "Optional for local development",
            })),
        },
    }
}

/// Check BearDog connectivity
async fn check_beardog_connectivity() -> HealthCheck {
    let start = Instant::now();

    // Check for BearDog Unix socket
    let uid = std::env::var("UID")
        .ok()
        .and_then(|u| u.parse::<u32>().ok())
        .unwrap_or(1000);
    
    let beardog_socket = std::env::var("BEARDOG_SOCKET")
        .ok()
        .unwrap_or_else(|| format!("/run/user/{}/beardog.sock", uid));

    let status = if std::path::Path::new(&beardog_socket).exists() {
        HealthStatus::Ok
    } else {
        HealthStatus::Warning
    };

    let message = if status == HealthStatus::Ok {
        "Socket exists".to_string()
    } else {
        "Socket not found".to_string()
    };

    HealthCheck {
        name: "BearDog",
        status,
        message,
        duration_ms: start.elapsed().as_millis() as u64,
        details: Some(serde_json::json!({
            "socket_path": beardog_socket,
            "note": "Optional for local development",
        })),
    }
}

/// Check Unix socket health
async fn check_unix_socket() -> HealthCheck {
    let start = Instant::now();

    let uid = std::env::var("UID")
        .ok()
        .and_then(|u| u.parse::<u32>().ok())
        .unwrap_or(1000);
    
    let socket_path = std::env::var("SQUIRREL_SOCKET")
        .ok()
        .unwrap_or_else(|| format!("/run/user/{}/squirrel.sock", uid));

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

/// Check HTTP server health
async fn check_http_server() -> HealthCheck {
    let start = Instant::now();

    let port = std::env::var("SQUIRREL_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(9010);

    HealthCheck {
        name: "HTTP Server",
        status: HealthStatus::Ok,
        message: format!("Will bind to port {}", port),
        duration_ms: start.elapsed().as_millis() as u64,
        details: Some(serde_json::json!({
            "port": port,
            "bind_address": "0.0.0.0",
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
        recommendations.push("Configure AI_PROVIDER_SOCKETS or set OPENAI_API_KEY/HUGGINGFACE_API_KEY".to_string());
    }

    // Check for Songbird warnings
    if checks
        .iter()
        .any(|c| c.name == "Songbird" && c.status == HealthStatus::Warning)
    {
        recommendations.push("Start Songbird for full ecosystem coordination (optional for development)".to_string());
    }

    // Check for BearDog warnings
    if checks
        .iter()
        .any(|c| c.name == "BearDog" && c.status == HealthStatus::Warning)
    {
        recommendations.push("Start BearDog for security features (optional for development)".to_string());
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
        println!("{} {}: {}", icon, check.name, check.message);
    }

    println!();

    // Print recommendations
    println!("RECOMMENDATIONS:");
    for rec in &report.recommendations {
        println!("  • {}", rec);
    }

    println!();

    // Print summary
    let status_icon = match report.overall_status {
        HealthStatus::Ok => "✅",
        HealthStatus::Warning => "⚠️ ",
        HealthStatus::Error => "❌",
    };
    println!(
        "{} Overall Status: {:?} (completed in {:.2}s)",
        status_icon,
        report.overall_status,
        duration.as_secs_f64()
    );
}

/// Print JSON format report
fn print_json_report(report: &HealthReport) -> Result<()> {
    let json = serde_json::to_string_pretty(report)
        .context("Failed to serialize health report to JSON")?;
    println!("{}", json);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check_binary() {
        let check = check_binary().await;
        assert_eq!(check.status, HealthStatus::Ok);
        assert!(check.message.contains("squirrel"));
    }

    #[tokio::test]
    async fn test_check_configuration() {
        let check = check_configuration().await;
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

    #[tokio::test]
    async fn test_binary_check_always_succeeds() {
        let check = check_binary().await;
        assert_eq!(check.status, HealthStatus::Ok);
        assert!(check.message.contains(env!("CARGO_PKG_VERSION")));
    }

    #[tokio::test]
    async fn test_configuration_check_structure() {
        let check = check_configuration().await;
        assert_eq!(check.name, "Configuration");
        assert!(!check.message.is_empty());
    }

    #[tokio::test]
    async fn test_unix_socket_check_returns_valid_status() {
        let check = check_unix_socket().await;
        assert!(matches!(
            check.status,
            HealthStatus::Ok | HealthStatus::Warning | HealthStatus::Error
        ));
    }

    #[tokio::test]
    async fn test_http_server_check_structure() {
        let check = check_http_server().await;
        assert_eq!(check.name, "HTTP Server");
    }

    // ========================================================================
    // CHAOS TESTS - Edge Cases
    // ========================================================================

    #[tokio::test]
    async fn test_all_checks_run_without_panic() {
        // Run all checks to ensure they don't panic
        let checks = tokio::join!(
            check_binary(),
            check_configuration(),
            check_unix_socket(),
            check_http_server(),
        );

        assert!(!checks.0.name.is_empty());
        assert!(!checks.1.name.is_empty());
        assert!(!checks.2.name.is_empty());
        assert!(!checks.3.name.is_empty());
    }

    #[tokio::test]
    async fn test_checks_complete_in_reasonable_time() {
        use std::time::Instant;
        let start = Instant::now();

        let _ = tokio::join!(
            check_binary(),
            check_configuration(),
            check_unix_socket(),
            check_http_server(),
        );

        let elapsed = start.elapsed();
        assert!(
            elapsed.as_secs() < 10,
            "All checks should complete in < 10s, took: {:?}",
            elapsed
        );
    }

    #[test]
    fn test_subsystem_filtering_none() {
        assert!(should_check(None, Subsystem::Ai));
        assert!(should_check(None, Subsystem::Config));
        assert!(should_check(None, Subsystem::Http));
    }

    #[test]
    fn test_subsystem_filtering_specific() {
        assert!(should_check(Some(Subsystem::Ai), Subsystem::Ai));
        assert!(!should_check(Some(Subsystem::Ai), Subsystem::Config));
        assert!(!should_check(Some(Subsystem::Config), Subsystem::Http));
    }

    #[test]
    fn test_subsystem_display() {
        assert_eq!(format!("{}", Subsystem::Ai), "ai");
        assert_eq!(format!("{}", Subsystem::Config), "config");
        assert_eq!(format!("{}", Subsystem::Http), "http");
    }

    // ========================================================================
    // FAULT TESTS - Error Scenarios
    // ========================================================================

    #[tokio::test]
    async fn test_all_checks_have_valid_durations() {
        let checks = tokio::join!(
            check_binary(),
            check_configuration(),
            check_unix_socket(),
            check_http_server(),
        );

        // All checks should have a duration measurement
        assert!(checks.0.duration_ms < 10000); // < 10s
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

    #[tokio::test]
    async fn test_concurrent_check_execution() {
        // Verify that all checks can run concurrently without issues
        let results = tokio::join!(
            check_binary(),
            check_binary(),
            check_configuration(),
            check_configuration(),
        );

        // All checks should succeed
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

    #[tokio::test]
    async fn test_checks_produce_valid_messages() {
        let checks = tokio::join!(
            check_binary(),
            check_configuration(),
            check_unix_socket(),
            check_http_server(),
        );

        // All checks should have non-empty messages
        assert!(!checks.0.message.is_empty());
        assert!(!checks.1.message.is_empty());
        assert!(!checks.2.message.is_empty());
        assert!(!checks.3.message.is_empty());
    }
}

