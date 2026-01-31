//! External Tracing Configuration
//!
//! This module provides configuration structures for external tracing
//! system integrations.

/// Configuration for external tracing systems
#[derive(Debug, Clone)]
pub struct ExternalTracingConfig {
    /// Endpoint URL for the tracing system
    pub endpoint_url: String,
    
    /// Authentication token, if needed
    pub auth_token: Option<String>,
    
    /// How often to flush spans to the external system (in seconds)
    pub flush_interval_seconds: u64,
    
    /// Maximum number of spans to buffer before flushing
    pub max_buffer_size: usize,
    
    /// Whether to add certain standard attributes to all spans
    pub add_standard_attributes: bool,
    
    /// Name of the service for tracing
    pub service_name: String,
    
    /// Environment name (dev, staging, prod)
    pub environment: String,
}

impl Default for ExternalTracingConfig {
    fn default() -> Self {
        // Multi-tier Jaeger tracing endpoint resolution
        let endpoint_url = std::env::var("JAEGER_ENDPOINT")
            .or_else(|_| std::env::var("TRACING_ENDPOINT"))
            .unwrap_or_else(|_| {
                let port = std::env::var("JAEGER_PORT")
                    .ok()
                    .and_then(|p| p.parse::<u16>().ok())
                    .unwrap_or(14268);  // Default Jaeger collector port
                format!("http://localhost:{}/api/traces", port)
            });

        Self {
            endpoint_url,
            auth_token: None,
            flush_interval_seconds: 30,
            max_buffer_size: 1000,
            add_standard_attributes: true,
            service_name: "squirrel-mcp".to_string(),
            environment: "development".to_string(),
        }
    }
} 