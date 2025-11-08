//! Service Endpoints - Legacy compatibility layer
//!
//! This module provides backward compatibility for code that relies on
//! `get_service_endpoints()`. New code should use the unified config system directly.

use once_cell::sync::Lazy;
use std::sync::Arc;
use url::Url;

/// Global service endpoints structure
#[derive(Debug, Clone)]
pub struct GlobalServiceEndpoints {
    pub mcp_endpoint: String,
    pub service_mesh_endpoint: String,
    pub security_service_endpoint: String,
    pub compute_service_endpoint: String,
    pub ui_endpoint: String,
    pub songbird_endpoint: String,
}

impl GlobalServiceEndpoints {
    /// Get MCP URL
    pub fn mcp_url(&self) -> Result<Url, url::ParseError> {
        Url::parse(&self.mcp_endpoint)
    }

    /// Get Beardog (security) URL
    pub fn beardog_url(&self) -> Result<Url, url::ParseError> {
        Url::parse(&self.security_service_endpoint)
    }

    /// Get Toadstool (compute) URL
    pub fn toadstool_url(&self) -> Result<Url, url::ParseError> {
        Url::parse(&self.compute_service_endpoint)
    }

    /// Get NestGate (database) URL
    pub fn nestgate_url(&self) -> Result<Url, url::ParseError> {
        // Default to localhost:5432 for NestGate
        std::env::var("NESTGATE_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:5432".to_string())
            .parse()
    }

    /// Get UI/Dashboard URL
    pub fn ui_url(&self) -> Result<Url, url::ParseError> {
        Url::parse(&self.ui_endpoint)
    }

    /// Get CORS origins
    pub fn cors_origins(&self) -> Vec<String> {
        // Default CORS origins based on environment
        let env = std::env::var("ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string());
        
        if env.eq_ignore_ascii_case("production") {
            vec!["https://localhost".to_string()]
        } else {
            vec![
                "http://localhost:3000".to_string(),
                "http://127.0.0.1:3000".to_string(),
                "http://localhost:8080".to_string(),
            ]
        }
    }
}

impl Default for GlobalServiceEndpoints {
    fn default() -> Self {
        let is_prod = std::env::var("ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string())
            .eq_ignore_ascii_case("production");
        
        let host = if is_prod { "0.0.0.0" } else { "127.0.0.1" };
        
        Self {
            mcp_endpoint: std::env::var("MCP_ENDPOINT")
                .unwrap_or_else(|_| format!("http://{}:8444", host)),
            service_mesh_endpoint: std::env::var("SERVICE_MESH_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:8500".to_string()),
            security_service_endpoint: std::env::var("SECURITY_SERVICE_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:8443".to_string()),
            compute_service_endpoint: std::env::var("COMPUTE_SERVICE_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:8445".to_string()),
            ui_endpoint: std::env::var("UI_ENDPOINT")
                .unwrap_or_else(|_| format!("http://{}:3000", host)),
            songbird_endpoint: std::env::var("SONGBIRD_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:8500".to_string()),
        }
    }
}

/// Global service endpoints singleton
static GLOBAL_ENDPOINTS: Lazy<Arc<GlobalServiceEndpoints>> = Lazy::new(|| {
    Arc::new(GlobalServiceEndpoints::default())
});

/// Get global service endpoints
///
/// This function provides backward compatibility for code that relies on
/// centralized service endpoint configuration. New code should use the
/// unified config system instead.
pub fn get_service_endpoints() -> Arc<GlobalServiceEndpoints> {
    Arc::clone(&GLOBAL_ENDPOINTS)
}

