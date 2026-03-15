// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Comprehensive tests for PrimalEndpoints
//!
//! Coverage goal: 90%+
//! Strategy: Test struct creation, defaults, serialization, edge cases

use super::endpoints::PrimalEndpoints;

#[cfg(test)]
mod endpoints_tests {
    use super::*;

    #[test]
    fn test_primal_endpoints_default() {
        let endpoints = PrimalEndpoints::default();

        assert_eq!(endpoints.primary, "http://localhost:8080");
        assert_eq!(endpoints.health, "http://localhost:8080/health");
        assert_eq!(endpoints.metrics, None);
        assert_eq!(endpoints.admin, None);
        assert_eq!(endpoints.websocket, None);
        assert_eq!(endpoints.custom, "");
    }

    #[test]
    fn test_primal_endpoints_custom_creation() {
        let endpoints = PrimalEndpoints {
            primary: "https://api.example.com".to_string(),
            health: "https://api.example.com/health".to_string(),
            metrics: Some("https://api.example.com/metrics".to_string()),
            admin: Some("https://api.example.com/admin".to_string()),
            websocket: Some("wss://api.example.com/ws".to_string()),
            custom: "custom=value".to_string(),
        };

        assert_eq!(endpoints.primary, "https://api.example.com");
        assert_eq!(endpoints.health, "https://api.example.com/health");
        assert_eq!(
            endpoints.metrics,
            Some("https://api.example.com/metrics".to_string())
        );
        assert_eq!(
            endpoints.admin,
            Some("https://api.example.com/admin".to_string())
        );
        assert_eq!(
            endpoints.websocket,
            Some("wss://api.example.com/ws".to_string())
        );
        assert_eq!(endpoints.custom, "custom=value");
    }

    #[test]
    fn test_primal_endpoints_clone() {
        let endpoints1 = PrimalEndpoints::default();
        let endpoints2 = endpoints1.clone();

        assert_eq!(endpoints1, endpoints2);
        assert_eq!(endpoints1.primary, endpoints2.primary);
    }

    #[test]
    fn test_primal_endpoints_equality() {
        let endpoints1 = PrimalEndpoints::default();
        let endpoints2 = PrimalEndpoints::default();

        assert_eq!(endpoints1, endpoints2);

        let endpoints3 = PrimalEndpoints {
            primary: "https://different.com".to_string(),
            health: "https://different.com/health".to_string(),
            metrics: None,
            admin: None,
            websocket: None,
            custom: String::new(),
        };

        assert_ne!(endpoints1, endpoints3);
    }

    #[test]
    fn test_primal_endpoints_debug() {
        let endpoints = PrimalEndpoints::default();
        let debug_str = format!("{:?}", endpoints);

        assert!(debug_str.contains("PrimalEndpoints"));
        assert!(debug_str.contains("primary"));
        assert!(debug_str.contains("health"));
    }

    #[test]
    fn test_primal_endpoints_serialization() {
        let endpoints = PrimalEndpoints {
            primary: "http://test.com".to_string(),
            health: "http://test.com/health".to_string(),
            metrics: Some("http://test.com/metrics".to_string()),
            admin: None,
            websocket: None,
            custom: String::new(),
        };

        let json = serde_json::to_string(&endpoints).unwrap();
        assert!(json.contains("http://test.com"));

        let deserialized: PrimalEndpoints = serde_json::from_str(&json).unwrap();
        assert_eq!(endpoints, deserialized);
    }

    #[test]
    fn test_primal_endpoints_partial_optional() {
        let endpoints = PrimalEndpoints {
            primary: "http://api.local".to_string(),
            health: "http://api.local/health".to_string(),
            metrics: Some("http://api.local/metrics".to_string()),
            admin: None, // Some fields optional
            websocket: Some("ws://api.local/ws".to_string()),
            custom: String::new(),
        };

        assert!(endpoints.metrics.is_some());
        assert!(endpoints.admin.is_none());
        assert!(endpoints.websocket.is_some());
    }

    #[test]
    fn test_primal_endpoints_empty_custom() {
        let endpoints = PrimalEndpoints::default();
        assert_eq!(endpoints.custom, "");
        assert!(endpoints.custom.is_empty());
    }

    #[test]
    fn test_primal_endpoints_various_protocols() {
        let http = PrimalEndpoints {
            primary: "http://http.example.com".to_string(),
            health: "http://http.example.com/health".to_string(),
            metrics: None,
            admin: None,
            websocket: None,
            custom: String::new(),
        };

        let https = PrimalEndpoints {
            primary: "https://https.example.com".to_string(),
            health: "https://https.example.com/health".to_string(),
            metrics: None,
            admin: None,
            websocket: None,
            custom: String::new(),
        };

        let ws = PrimalEndpoints {
            primary: "ws://ws.example.com".to_string(),
            health: "ws://ws.example.com/health".to_string(),
            websocket: Some("wss://ws.example.com/secure".to_string()),
            metrics: None,
            admin: None,
            custom: String::new(),
        };

        assert!(http.primary.starts_with("http://"));
        assert!(https.primary.starts_with("https://"));
        assert!(ws.websocket.as_ref().unwrap().starts_with("wss://"));
    }

    #[test]
    fn test_primal_endpoints_ipv4() {
        let endpoints = PrimalEndpoints {
            primary: "http://192.168.1.1:8080".to_string(),
            health: "http://192.168.1.1:8080/health".to_string(),
            metrics: None,
            admin: None,
            websocket: None,
            custom: String::new(),
        };

        assert!(endpoints.primary.contains("192.168.1.1"));
    }

    #[test]
    fn test_primal_endpoints_ipv6() {
        let endpoints = PrimalEndpoints {
            primary: "http://[::1]:8080".to_string(),
            health: "http://[::1]:8080/health".to_string(),
            metrics: None,
            admin: None,
            websocket: None,
            custom: String::new(),
        };

        assert!(endpoints.primary.contains("::1"));
    }

    #[test]
    fn test_primal_endpoints_custom_ports() {
        let endpoints = PrimalEndpoints {
            primary: "http://localhost:9000".to_string(),
            health: "http://localhost:9001/health".to_string(),
            metrics: Some("http://localhost:9002/metrics".to_string()),
            admin: Some("http://localhost:9003/admin".to_string()),
            websocket: Some("ws://localhost:9004/ws".to_string()),
            custom: String::new(),
        };

        assert!(endpoints.primary.contains(":9000"));
        assert!(endpoints.health.contains(":9001"));
        assert!(endpoints.metrics.as_ref().unwrap().contains(":9002"));
    }
}
