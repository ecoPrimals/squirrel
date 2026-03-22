// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Primal API endpoints.

use serde::{Deserialize, Serialize};

/// Primal API endpoints
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimalEndpoints {
    /// Primary API endpoint
    pub primary: String,
    /// Health check endpoint
    pub health: String,
    /// Metrics endpoint
    pub metrics: Option<String>,
    /// Admin endpoint
    pub admin: Option<String>,
    /// WebSocket endpoint
    pub websocket: Option<String>,
    /// Additional custom endpoints
    pub custom: String, // Changed from HashMap to String to fix Hash issues
}

impl Default for PrimalEndpoints {
    fn default() -> Self {
        // Multi-tier primal endpoint resolution
        // 1. PRIMAL_ENDPOINT (full endpoint)
        // 2. PRIMAL_PORT (port override)
        // 3. Default: http://localhost:8080
        let port = std::env::var("PRIMAL_PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(8080); // Default primal port
        let primary = std::env::var("PRIMAL_ENDPOINT")
            .unwrap_or_else(|_| format!("http://localhost:{}", port));
        let health = format!("{}/health", primary);

        Self {
            primary,
            health,
            metrics: None,
            admin: None,
            websocket: None,
            custom: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_uses_primal_port_when_endpoint_unset() {
        temp_env::with_vars(
            [
                ("PRIMAL_ENDPOINT", None::<&str>),
                ("PRIMAL_PORT", Some("9123")),
            ],
            || {
                let e = PrimalEndpoints::default();
                assert!(e.primary.contains("9123"));
                assert_eq!(e.health, format!("{}/health", e.primary));
            },
        );
    }

    #[test]
    fn test_default_prefers_primal_endpoint_over_port() {
        temp_env::with_vars(
            [
                ("PRIMAL_ENDPOINT", Some("http://explicit:7000")),
                ("PRIMAL_PORT", Some("9999")),
            ],
            || {
                let e = PrimalEndpoints::default();
                assert_eq!(e.primary, "http://explicit:7000");
                assert!(e.health.ends_with("/health"));
            },
        );
    }

    #[test]
    fn test_default_invalid_primal_port_falls_back_to_8080() {
        temp_env::with_vars(
            [
                ("PRIMAL_ENDPOINT", None::<&str>),
                ("PRIMAL_PORT", Some("not-a-port")),
            ],
            || {
                let e = PrimalEndpoints::default();
                assert!(e.primary.contains("8080"));
            },
        );
    }

    #[test]
    fn test_primal_endpoints_serde() {
        let endpoints = PrimalEndpoints {
            primary: "http://test:8080".to_string(),
            health: "http://test:8080/health".to_string(),
            metrics: Some("http://test:8080/metrics".to_string()),
            admin: None,
            websocket: Some("ws://test:8081".to_string()),
            custom: "extra".to_string(),
        };
        let json = serde_json::to_string(&endpoints).expect("serialize");
        let deser: PrimalEndpoints = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser, endpoints);
    }

    #[test]
    fn test_primal_endpoints_fields() {
        let endpoints = PrimalEndpoints {
            primary: "http://localhost:8080".to_string(),
            health: "http://localhost:8080/health".to_string(),
            metrics: None,
            admin: None,
            websocket: None,
            custom: String::new(),
        };
        assert_eq!(endpoints.primary, "http://localhost:8080");
        assert_eq!(endpoints.health, "http://localhost:8080/health");
        assert!(endpoints.metrics.is_none());
        assert!(endpoints.custom.is_empty());
    }
}
