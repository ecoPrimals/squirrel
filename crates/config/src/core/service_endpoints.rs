//! Centralized Service Endpoints Configuration
//!
//! This module provides a centralized way to configure all service endpoints,
//! eliminating hardcoded localhost values and making the system deployment-ready.

use std::env;
use url::Url;
use serde::{Serialize, Deserialize};

/// Default service ports
pub mod ports {
    pub const UI_PORT: u16 = 3000;
    pub const BEARDOG_SECURITY_PORT: u16 = 8443;
    pub const NESTGATE_STORAGE_PORT: u16 = 8444;
    pub const TOADSTOOL_COMPUTE_PORT: u16 = 8445;
    pub const MCP_DEFAULT_PORT: u16 = 8080;
    pub const METRICS_PORT: u16 = 9090;
    pub const SONGBIRD_MONITORING_PORT: u16 = 8900;
}

/// Universal service endpoints configuration - capability-based, not name-based
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalServiceEndpoints {
    pub ui_endpoint: String,
    pub security_service_endpoint: String,    // Any primal with security capabilities
    pub storage_service_endpoint: String,     // Any primal with storage capabilities
    pub compute_service_endpoint: String,     // Any primal with compute capabilities
    pub mcp_endpoint: String,                 // This primal's MCP endpoint
    pub metrics_endpoint: String,             // Metrics collection endpoint
    pub service_mesh_endpoint: String,        // Service mesh/orchestration endpoint
}

impl Default for GlobalServiceEndpoints {
    fn default() -> Self {
        // Use capability-based defaults - these will be discovered dynamically
        Self {
            ui_endpoint: "http://localhost:3000".to_string(),
            security_service_endpoint: "http://localhost:8443".to_string(),    // Discovered by security capability
            storage_service_endpoint: "http://localhost:8444".to_string(),     // Discovered by storage capability  
            compute_service_endpoint: "http://localhost:8445".to_string(),     // Discovered by compute capability
            mcp_endpoint: "http://localhost:8446".to_string(),                 // This primal's own endpoint
            metrics_endpoint: "http://localhost:8447".to_string(),             // Metrics capability endpoint
            service_mesh_endpoint: "http://localhost:8448".to_string(),        // Service mesh capability endpoint
        }
    }
}

impl GlobalServiceEndpoints {
    /// Create service endpoints from environment variables with error handling
    pub fn try_from_env() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let is_production = env::var("SQUIRREL_ENV").unwrap_or_default() == "production";
        let is_docker = env::var("SQUIRREL_DOCKER").unwrap_or_default() == "true";
        
        // Determine base hostname based on environment
        let base_host = if is_production {
            // In production, use service names for container networking
            env::var("CLUSTER_HOST").unwrap_or_else(|_| "squirrel-cluster".to_string())
        } else if is_docker {
            // In Docker Compose, use service names
            "localhost".to_string()
        } else {
            // Development mode
            env::var("DEV_HOST").unwrap_or_else(|_| "localhost".to_string())
        };

        Ok(Self {
            ui_endpoint: Self::get_ui_endpoint(&base_host, is_production, is_docker),
            security_service_endpoint: Self::get_security_service_endpoint(&base_host, is_production, is_docker),
            storage_service_endpoint: Self::get_storage_service_endpoint(&base_host, is_production, is_docker),
            compute_service_endpoint: Self::get_compute_service_endpoint(&base_host, is_production, is_docker),
            mcp_endpoint: Self::get_mcp_endpoint(&base_host, is_production, is_docker),
            metrics_endpoint: Self::get_metrics_endpoint(&base_host, is_production, is_docker),
            service_mesh_endpoint: Self::get_service_mesh_endpoint(&base_host, is_production, is_docker),
        })
    }

    /// Create service endpoints from environment variables with smart defaults
    pub fn from_env() -> Self {
        match Self::try_from_env() {
            Ok(endpoints) => endpoints,
            Err(e) => {
                use tracing::warn;
                warn!("Failed to load service endpoints from environment: {}, using defaults", e);
                Self::default()
            }
        }
    }

    fn get_ui_endpoint(base_host: &str, is_production: bool, is_docker: bool) -> String {
        env::var("SQUIRREL_UI_ENDPOINT").unwrap_or_else(|_| {
            if is_production {
                format!("https://ui.{}", base_host)
            } else if is_docker {
                "http://biomeos-ui:3000".to_string()
            } else {
                format!("http://{}:{}", base_host, ports::UI_PORT)
            }
        })
    }

    fn get_security_service_endpoint(base_host: &str, is_production: bool, is_docker: bool) -> String {
        env::var("SECURITY_SERVICE_ENDPOINT").unwrap_or_else(|_| {
            if is_production {
                format!("https://security.{}", base_host)
            } else if is_docker {
                "http://security-service:8443".to_string()
            } else {
                format!("http://{}:{}", base_host, ports::BEARDOG_SECURITY_PORT)
            }
        })
    }

    fn get_storage_service_endpoint(base_host: &str, is_production: bool, is_docker: bool) -> String {
        env::var("STORAGE_SERVICE_ENDPOINT").unwrap_or_else(|_| {
            if is_production {
                format!("https://storage.{}", base_host)
            } else if is_docker {
                "http://storage-service:8444".to_string()
            } else {
                format!("http://{}:{}", base_host, ports::NESTGATE_STORAGE_PORT)
            }
        })
    }

    fn get_compute_service_endpoint(base_host: &str, is_production: bool, is_docker: bool) -> String {
        env::var("COMPUTE_SERVICE_ENDPOINT").unwrap_or_else(|_| {
            if is_production {
                format!("https://compute.{}", base_host)
            } else if is_docker {
                "http://compute-service:8445".to_string()
            } else {
                format!("http://{}:{}", base_host, ports::TOADSTOOL_COMPUTE_PORT)
            }
        })
    }

    fn get_mcp_endpoint(base_host: &str, is_production: bool, is_docker: bool) -> String {
        env::var("MCP_ENDPOINT").unwrap_or_else(|_| {
            if is_production {
                format!("https://mcp.{}", base_host)
            } else if is_docker {
                "http://squirrel-mcp:8080".to_string()
            } else {
                format!("http://{}:{}", base_host, ports::MCP_DEFAULT_PORT)
            }
        })
    }

    fn get_metrics_endpoint(base_host: &str, is_production: bool, is_docker: bool) -> String {
        env::var("METRICS_ENDPOINT").unwrap_or_else(|_| {
            if is_production {
                format!("https://metrics.{}", base_host)
            } else if is_docker {
                "http://prometheus:9090".to_string()
            } else {
                format!("http://{}:{}", base_host, ports::METRICS_PORT)
            }
        })
    }

    fn get_service_mesh_endpoint(base_host: &str, is_production: bool, is_docker: bool) -> String {
        env::var("SONGBIRD_ENDPOINT").unwrap_or_else(|_| {
            if is_production {
                format!("https://songbird.{}", base_host)
            } else if is_docker {
                "http://songbird:8900".to_string()
            } else {
                format!("http://{}:{}", base_host, ports::SONGBIRD_MONITORING_PORT)
            }
        })
    }

    /// Get beardog security service URL
    pub fn beardog_url(&self) -> Result<Url, url::ParseError> {
        Url::parse(&self.security_service_endpoint)
    }

    /// Get UI service URL
    pub fn ui_url(&self) -> Result<Url, url::ParseError> {
        Url::parse(&self.ui_endpoint)
    }

    /// Get nestgate storage service URL
    pub fn nestgate_url(&self) -> Result<Url, url::ParseError> {
        Url::parse(&self.storage_service_endpoint)
    }

    /// Get toadstool compute service URL
    pub fn toadstool_url(&self) -> Result<Url, url::ParseError> {
        Url::parse(&self.compute_service_endpoint)
    }

    /// Get MCP service URL
    pub fn mcp_url(&self) -> Result<Url, url::ParseError> {
        Url::parse(&self.mcp_endpoint)
    }

    /// Get metrics service URL
    pub fn metrics_url(&self) -> Result<Url, url::ParseError> {
        Url::parse(&self.metrics_endpoint)
    }

    /// Get songbird monitoring service URL
    pub fn songbird_url(&self) -> Result<Url, url::ParseError> {
        Url::parse(&self.service_mesh_endpoint)
    }

    /// Get CORS origins based on environment
    pub fn cors_origins(&self) -> Vec<String> {
        let env_origins = env::var("SQUIRREL_CORS_ORIGINS")
            .unwrap_or_default()
            .split(',')
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.trim().to_string())
            .collect::<Vec<_>>();

        if !env_origins.is_empty() {
            env_origins
        } else {
            // Default CORS origins based on environment
            vec![
                self.ui_endpoint.clone(),
                // Add common development origins
                "http://localhost:3000".to_string(),
                "http://127.0.0.1:3000".to_string(),
            ]
        }
    }
}

/// Global service endpoints instance
static mut GLOBAL_ENDPOINTS: Option<GlobalServiceEndpoints> = None;
static INIT: std::sync::Once = std::sync::Once::new();

/// Get global service endpoints (singleton)
/// 
/// Returns the global service endpoints, initializing them from environment if needed.
/// Uses safe fallback to default values if initialization fails.
pub fn get_service_endpoints() -> &'static GlobalServiceEndpoints {
    use tracing::{error, warn};
    
    unsafe {
        INIT.call_once(|| {
            match GlobalServiceEndpoints::try_from_env() {
                Ok(endpoints) => {
                    GLOBAL_ENDPOINTS = Some(endpoints);
                }
                Err(e) => {
                    error!("Failed to initialize service endpoints from environment: {}", e);
                    warn!("Using default service endpoints as fallback");
                    GLOBAL_ENDPOINTS = Some(GlobalServiceEndpoints::default());
                }
            }
        });
        
        // This is now safe because INIT.call_once guarantees initialization
        GLOBAL_ENDPOINTS.as_ref().expect("Service endpoints must be initialized by call_once")
    }
}

/// Initialize service endpoints with custom configuration
pub fn init_service_endpoints(endpoints: GlobalServiceEndpoints) {
    unsafe {
        GLOBAL_ENDPOINTS = Some(endpoints);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_endpoints() {
        let endpoints = GlobalServiceEndpoints::default();
        assert!(endpoints.ui_endpoint.contains("3000"));
        assert!(endpoints.security_service_endpoint.contains("8443"));
        assert!(endpoints.storage_service_endpoint.contains("8444"));
        assert!(endpoints.compute_service_endpoint.contains("8445"));
        assert!(endpoints.service_mesh_endpoint.contains("8900"));
    }

    #[test]
    fn test_production_endpoints() {
        env::set_var("SQUIRREL_ENV", "production");
        env::set_var("CLUSTER_HOST", "example.com");
        
        let endpoints = GlobalServiceEndpoints::from_env();
        assert!(endpoints.security_service_endpoint.contains("https://security.example.com"));
        assert!(endpoints.ui_endpoint.contains("https://ui.example.com"));
        
        // Cleanup
        env::remove_var("SQUIRREL_ENV");
        env::remove_var("CLUSTER_HOST");
    }

    #[test]
    fn test_docker_endpoints() {
        env::set_var("SQUIRREL_DOCKER", "true");
        
        let endpoints = GlobalServiceEndpoints::from_env();
        assert!(endpoints.ui_endpoint.contains("biomeos-ui"));
        assert!(endpoints.security_service_endpoint.contains("beardog:"));
        
        // Cleanup
        env::remove_var("SQUIRREL_DOCKER");
    }

    #[test]
    fn test_url_parsing() {
        let endpoints = GlobalServiceEndpoints::default();
        assert!(endpoints.beardog_url().is_ok());
        assert!(endpoints.ui_url().is_ok());
        assert!(endpoints.nestgate_url().is_ok());
        assert!(endpoints.toadstool_url().is_ok());
        assert!(endpoints.songbird_url().is_ok());
    }

    #[test]
    fn test_cors_origins() {
        let endpoints = GlobalServiceEndpoints::default();
        let origins = endpoints.cors_origins();
        assert!(!origins.is_empty());
        assert!(origins.iter().any(|o| o.contains("3000")));
    }
} 