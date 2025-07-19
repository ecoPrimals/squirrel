//! Configuration defaults for Squirrel MCP
//!
//! This module defines default configuration structures and values
//! for various service types and configuration categories.

use serde::{Deserialize, Serialize};

/// Configuration defaults that can be overridden by environment variables
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConfigDefaults {
    /// Network configuration defaults
    pub network: NetworkDefaults,
    /// Database configuration defaults
    pub database: DatabaseDefaults,
    /// External service defaults
    pub external_services: ExternalServiceDefaults,
    /// AI service defaults
    pub ai_services: AIServiceDefaults,
    /// Observability defaults
    pub observability: ObservabilityDefaults,
}

/// Network configuration defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDefaults {
    /// Default server host
    pub host: String,
    /// Default server port
    pub port: u16,
    /// Default CORS origins
    pub cors_origins: Vec<String>,
    /// Default WebSocket port
    pub websocket_port: u16,
    /// Default bind address for production
    pub bind_address: String,
}

/// Database configuration defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseDefaults {
    /// Default database URL (should be overridden in production)
    pub url: String,
    /// Default maximum connections
    pub max_connections: u32,
    /// Default connection timeout
    pub timeout_seconds: u64,
    /// Default test database URL
    pub test_url: String,
}

/// External service configuration defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalServiceDefaults {
    /// Default Songbird service URL
    pub songbird_url: String,
    /// Default Toadstool service URL
    pub toadstool_url: String,
    /// Default NestGate service URL
    pub nestgate_url: String,
    /// Default BearDog service URL
    pub beardog_url: String,
    /// Default BiomeOS service URL
    pub biomeos_url: String,
    /// Default BiomeOS AI API URL
    pub biomeos_ai_api: String,
    /// Default BiomeOS MCP API URL
    pub biomeos_mcp_api: String,
    /// Default BiomeOS Context API URL
    pub biomeos_context_api: String,
    /// Default BiomeOS Health API URL
    pub biomeos_health_api: String,
    /// Default BiomeOS Metrics API URL
    pub biomeos_metrics_api: String,
    /// Default BiomeOS WebSocket URL
    pub biomeos_websocket_url: String,
}

/// AI service configuration defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIServiceDefaults {
    /// Default OpenAI API URL
    pub openai_api_url: String,
    /// Default Anthropic API URL
    pub anthropic_api_url: String,
    /// Default Ollama API URL
    pub ollama_api_url: String,
    /// Default Llama.cpp API URL
    pub llamacpp_api_url: String,
    /// Default temperature
    pub temperature: f32,
    /// Default timeout
    pub timeout_seconds: u64,
}

/// Observability configuration defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityDefaults {
    /// Default dashboard URL
    pub dashboard_url: String,
    /// Default OTLP endpoint
    pub otlp_endpoint: String,
    /// Default Jaeger endpoint
    pub jaeger_endpoint: String,
    /// Default Zipkin endpoint
    pub zipkin_endpoint: String,
    /// Default metrics port
    pub metrics_port: u16,
    /// Default health check port
    pub health_port: u16,
}

impl Default for NetworkDefaults {
    fn default() -> Self {
        // Environment-aware defaults
        let is_production = std::env::var("ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string())
            .eq_ignore_ascii_case("production");

        let default_host = if is_production {
            "0.0.0.0".to_string() // Bind to all interfaces in production
        } else {
            std::env::var("DEFAULT_HOST").unwrap_or_else(|_| "127.0.0.1".to_string())
        };

        let default_cors = if is_production {
            vec![] // No default CORS origins in production - must be explicitly configured
        } else {
            vec![std::env::var("DEFAULT_CORS_ORIGIN")
                .unwrap_or_else(|_| "http://localhost:3000".to_string())]
        };

        Self {
            host: default_host,
            port: std::env::var("DEFAULT_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
            cors_origins: default_cors,
            websocket_port: std::env::var("DEFAULT_WS_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8081),
            bind_address: "0.0.0.0".to_string(),
        }
    }
}

impl Default for DatabaseDefaults {
    fn default() -> Self {
        Self {
            url: "sqlite::memory:".to_string(),
            max_connections: 10,
            timeout_seconds: 30,
            test_url: "sqlite::memory:".to_string(),
        }
    }
}

impl Default for ExternalServiceDefaults {
    fn default() -> Self {
        // Environment-aware service discovery
        let is_production = std::env::var("ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string())
            .eq_ignore_ascii_case("production");

        let service_host = if is_production {
            // In production, use service names for container/k8s environments
            "songbird"
        } else {
            "localhost"
        };

        let base_port = 8000; // Base port for service allocation

        Self {
            songbird_url: std::env::var("SONGBIRD_URL")
                .unwrap_or_else(|_| format!("http://{}:8080", service_host)),
            toadstool_url: std::env::var("TOADSTOOL_URL")
                .unwrap_or_else(|_| format!("http://toadstool:{}", base_port + 445)),
            nestgate_url: std::env::var("NESTGATE_URL")
                .unwrap_or_else(|_| format!("http://nestgate:{}", base_port + 444)),
            beardog_url: std::env::var("BEARDOG_URL")
                .unwrap_or_else(|_| format!("http://beardog:{}", base_port + 443)),
            biomeos_url: std::env::var("BIOMEOS_URL")
                .unwrap_or_else(|_| format!("http://biomeos:5000")),
            biomeos_ai_api: std::env::var("BIOMEOS_AI_API")
                .unwrap_or_else(|_| format!("http://biomeos:5000/ai")),
            biomeos_mcp_api: std::env::var("BIOMEOS_MCP_API")
                .unwrap_or_else(|_| format!("http://biomeos:5000/mcp")),
            biomeos_context_api: std::env::var("BIOMEOS_CONTEXT_API")
                .unwrap_or_else(|_| format!("http://biomeos:5000/context")),
            biomeos_health_api: std::env::var("BIOMEOS_HEALTH_API")
                .unwrap_or_else(|_| format!("http://biomeos:5000/health")),
            biomeos_metrics_api: std::env::var("BIOMEOS_METRICS_API")
                .unwrap_or_else(|_| format!("http://biomeos:5000/metrics")),
            biomeos_websocket_url: std::env::var("BIOMEOS_WS_URL")
                .unwrap_or_else(|_| format!("ws://biomeos:5000/ws")),
        }
    }
}

impl Default for AIServiceDefaults {
    fn default() -> Self {
        let is_production = std::env::var("ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string())
            .eq_ignore_ascii_case("production");

        let local_ai_host = if is_production {
            "ollama" // Use service name in production
        } else {
            "localhost"
        };

        Self {
            openai_api_url: std::env::var("OPENAI_API_URL")
                .unwrap_or_else(|_| "https://api.openai.com/v1".to_string()),
            anthropic_api_url: std::env::var("ANTHROPIC_API_URL")
                .unwrap_or_else(|_| "https://api.anthropic.com/v1".to_string()),
            ollama_api_url: std::env::var("OLLAMA_API_URL")
                .unwrap_or_else(|_| format!("http://{}:11434", local_ai_host)),
            llamacpp_api_url: std::env::var("LLAMACPP_API_URL")
                .unwrap_or_else(|_| format!("http://{}:8080", local_ai_host)),
            temperature: std::env::var("DEFAULT_AI_TEMPERATURE")
                .ok()
                .and_then(|t| t.parse().ok())
                .unwrap_or(0.7),
            timeout_seconds: std::env::var("AI_TIMEOUT_SECONDS")
                .ok()
                .and_then(|t| t.parse().ok())
                .unwrap_or(30),
        }
    }
}

impl Default for ObservabilityDefaults {
    fn default() -> Self {
        let is_production = std::env::var("ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string())
            .eq_ignore_ascii_case("production");

        let observability_host = if is_production {
            "jaeger" // Use service names in production
        } else {
            "localhost"
        };

        Self {
            dashboard_url: std::env::var("DASHBOARD_URL").unwrap_or_else(|_| {
                format!(
                    "http://{}:3000",
                    if is_production {
                        "dashboard"
                    } else {
                        "localhost"
                    }
                )
            }),
            otlp_endpoint: std::env::var("OTLP_ENDPOINT")
                .unwrap_or_else(|_| format!("http://{}:4317", observability_host)),
            jaeger_endpoint: std::env::var("JAEGER_ENDPOINT")
                .unwrap_or_else(|_| format!("http://{}:14268/api/traces", observability_host)),
            zipkin_endpoint: std::env::var("ZIPKIN_ENDPOINT").unwrap_or_else(|_| {
                format!(
                    "http://{}:9411",
                    if is_production { "zipkin" } else { "localhost" }
                )
            }),
            metrics_port: std::env::var("METRICS_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(4318),
            health_port: std::env::var("HEALTH_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(4319),
        }
    }
}
