//! Dashboard configuration
//!
//! This module defines the configuration structures for the dashboard
//! including server settings, UI configuration, and security settings.

use std::collections::HashSet;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};

use super::security::{TlsConfig, AuthConfig, RateLimitConfig, SecurityLoggingConfig, MaskingRule, AuditConfig, AuditStorage};

/// Dashboard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    /// Server configuration
    #[serde(default)]
    pub server: ServerConfig,
    
    /// WebSocket configuration
    #[serde(default)]
    pub websocket: WebSocketConfig,
    
    /// Security configuration
    #[serde(default)]
    pub security: SecurityConfig,
    
    /// Component configuration
    #[serde(default)]
    pub components: ComponentConfig,
    
    /// Update interval in seconds
    #[serde(default = "default_update_interval")]
    pub update_interval: u64,
    
    /// Which metric categories to display
    #[serde(default)]
    pub displayed_categories: HashSet<MetricCategory>,
    
    /// Custom dashboard panels
    #[serde(default)]
    pub custom_panels: Vec<PanelConfig>,
    
    /// Alert display settings
    #[serde(default)]
    pub alert_settings: AlertDisplaySettings,
    
    /// Data retention period in seconds
    #[serde(default = "default_retention_period")]
    pub retention_period: u64,
}

fn default_update_interval() -> u64 {
    10 // 10 seconds
}

fn default_retention_period() -> u64 {
    60 * 60 * 24 * 7 // 7 days in seconds
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            websocket: WebSocketConfig::default(),
            security: SecurityConfig::default(),
            components: ComponentConfig::default(),
            update_interval: default_update_interval(),
            displayed_categories: HashSet::new(),
            custom_panels: Vec::new(),
            alert_settings: AlertDisplaySettings::default(),
            retention_period: default_retention_period(),
        }
    }
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server host
    #[serde(default = "default_host")]
    pub host: String,
    
    /// Server port
    #[serde(default = "default_port")]
    pub port: u16,
    
    /// Server path
    #[serde(default = "default_path")]
    pub path: String,
    
    /// Static file directory
    #[serde(default)]
    pub static_dir: Option<PathBuf>,
    
    /// Enable HTTP API
    #[serde(default = "default_true")]
    pub enable_http_api: bool,
}

fn default_true() -> bool {
    true
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    8765
}

fn default_path() -> String {
    "/ws".to_string()
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            path: default_path(),
            static_dir: None,
            enable_http_api: default_true(),
        }
    }
}

/// WebSocket configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketConfig {
    /// Maximum number of connections
    #[serde(default = "default_max_connections")]
    pub max_connections: usize,
    
    /// Ping interval
    #[serde(default = "default_ping_interval")]
    pub ping_interval: u64,
    
    /// Connection timeout
    #[serde(default = "default_connection_timeout")]
    pub connection_timeout: u64,
    
    /// Maximum message size
    #[serde(default = "default_max_message_size")]
    pub max_message_size: usize,
    
    /// Compression threshold
    #[serde(default = "default_compression_threshold")]
    pub compression_threshold: usize,
    
    /// Enable message compression
    #[serde(default = "default_true")]
    pub enable_compression: bool,
}

fn default_max_connections() -> usize {
    100
}

fn default_ping_interval() -> u64 {
    30 // 30 seconds
}

fn default_connection_timeout() -> u64 {
    60 // 60 seconds
}

fn default_max_message_size() -> usize {
    1024 * 1024 // 1MB
}

fn default_compression_threshold() -> usize {
    8192 // 8KB
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            max_connections: default_max_connections(),
            ping_interval: default_ping_interval(),
            connection_timeout: default_connection_timeout(),
            max_message_size: default_max_message_size(),
            compression_threshold: default_compression_threshold(),
            enable_compression: default_true(),
        }
    }
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// TLS configuration
    #[serde(default)]
    pub tls: Option<TlsConfig>,
    
    /// Authentication configuration
    #[serde(default)]
    pub auth: AuthConfig,
    
    /// Rate limiting configuration
    #[serde(default)]
    pub rate_limit: RateLimitConfig,
    
    /// Allowed origins for CORS
    #[serde(default)]
    pub allowed_origins: Vec<String>,
    
    /// Data masking rules for sensitive information
    #[serde(default)]
    pub masking_rules: Vec<MaskingRule>,
    
    /// Security logging configuration
    #[serde(default)]
    pub logging: SecurityLoggingConfig,
    
    /// Audit configuration
    #[serde(default)]
    pub audit: Option<AuditConfig>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        // Create default auth config with no authentication
        let auth = AuthConfig {
            auth_type: super::security::AuthType::None,
            token_expiration: 8 * 60 * 60, // 8 hours
            require_reauth: true,
            users: std::collections::HashMap::new(),
        };
        
        // Create default audit config
        let audit = AuditConfig {
            enabled: false,
            storage: AuditStorage::File(PathBuf::from("logs/audit.log")),
            include_user_context: true,
            tamper_proof: true,
        };
        
        // Create default security logging config
        let logging = SecurityLoggingConfig {
            log_authentication_attempts: true,
            log_authorization_failures: true,
            log_configuration_changes: true,
            log_data_access: super::security::AccessLoggingLevel::Sensitive,
        };
        
        Self {
            tls: None,
            auth,
            rate_limit: RateLimitConfig::default(),
            allowed_origins: Vec::new(),
            masking_rules: Vec::new(),
            logging,
            audit: Some(audit),
        }
    }
}

/// Component configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentConfig {
    /// Default refresh interval for components
    #[serde(default = "default_component_refresh")]
    pub default_refresh_interval: u64,
    
    /// Default data retention policy
    #[serde(default = "default_data_retention")]
    pub default_data_retention: u64,
    
    /// Show timestamps by default
    #[serde(default = "default_true")]
    pub show_timestamps: bool,
}

fn default_component_refresh() -> u64 {
    5 // 5 seconds
}

fn default_data_retention() -> u64 {
    3600 // 1 hour in seconds
}

impl Default for ComponentConfig {
    fn default() -> Self {
        Self {
            default_refresh_interval: default_component_refresh(),
            default_data_retention: default_data_retention(),
            show_timestamps: default_true(),
        }
    }
}

/// Metric category
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetricCategory {
    /// System metrics
    System,
    /// Protocol metrics
    Protocol,
    /// Tool metrics
    Tool,
    /// Network metrics
    Network,
    /// Custom metrics
    Custom(String),
}

/// Panel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelConfig {
    /// Panel ID
    pub id: String,
    /// Panel title
    pub title: String,
    /// Panel type
    pub panel_type: PanelType,
    /// Metrics to display
    pub metrics: Vec<String>,
    /// Panel position
    pub position: PanelPosition,
    /// Panel size
    pub size: PanelSize,
    /// Refresh rate in seconds
    pub refresh_rate: u64,
}

/// Panel type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PanelType {
    /// Line chart
    LineChart,
    /// Bar chart
    BarChart,
    /// Gauge
    Gauge,
    /// Table
    Table,
    /// Status panel
    StatusPanel,
    /// Custom panel
    Custom(String),
}

/// Panel position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelPosition {
    /// X position
    pub x: u32,
    /// Y position
    pub y: u32,
}

/// Panel size
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelSize {
    /// Width
    pub width: u32,
    /// Height
    pub height: u32,
}

/// Alert display settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertDisplaySettings {
    /// Show acknowledged alerts
    #[serde(default)]
    pub show_acknowledged: bool,
    /// Maximum number of alerts to display
    #[serde(default = "default_max_alerts")]
    pub max_alerts: usize,
    /// Group alerts by source
    #[serde(default)]
    pub group_by_source: bool,
    /// Auto refresh alerts
    #[serde(default = "default_true")]
    pub auto_refresh: bool,
}

fn default_max_alerts() -> usize {
    50
}

impl Default for AlertDisplaySettings {
    fn default() -> Self {
        Self {
            show_acknowledged: false,
            max_alerts: default_max_alerts(),
            group_by_source: true,
            auto_refresh: default_true(),
        }
    }
}

impl DashboardConfig {
    /// Creates a new dashboard configuration with default values
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Sets the server host
    #[must_use]
    pub fn with_host(mut self, host: impl Into<String>) -> Self {
        self.server.host = host.into();
        self
    }
    
    /// Sets the server port
    #[must_use]
    pub fn with_port(mut self, port: u16) -> Self {
        self.server.port = port;
        self
    }
    
    /// Sets the maximum number of connections
    #[must_use]
    pub fn with_max_connections(mut self, max_connections: usize) -> Self {
        self.websocket.max_connections = max_connections;
        self
    }
    
    /// Sets the compression threshold
    #[must_use]
    pub fn with_compression_threshold(mut self, threshold: usize) -> Self {
        self.websocket.compression_threshold = threshold;
        self
    }
    
    /// Enables TLS
    #[must_use]
    pub fn with_tls(mut self, cert_path: impl Into<PathBuf>, key_path: impl Into<PathBuf>) -> Self {
        self.security.tls = Some(TlsConfig {
            cert_path: cert_path.into(),
            key_path: key_path.into(),
            min_tls_version: super::security::TlsVersion::default(),
            cipher_preferences: super::security::CipherPreferences::default(),
        });
        self
    }
    
    /// Sets the authentication config
    #[must_use]
    pub fn with_auth(mut self, auth: AuthConfig) -> Self {
        self.security.auth = auth;
        self
    }
    
    /// Sets the allowed origins
    #[must_use]
    pub fn with_allowed_origins(mut self, origins: Vec<String>) -> Self {
        self.security.allowed_origins = origins;
        self
    }
    
    /// Adds a masking rule
    #[must_use]
    pub fn with_masking_rule(mut self, pattern: &str, replacement: &str) -> Self {
        self.security.masking_rules.push(
            MaskingRule::new(pattern, replacement)
        );
        self
    }
    
    /// Enables audit logging
    #[must_use]
    pub fn with_audit_logging(mut self, path: impl Into<PathBuf>) -> Self {
        self.security.audit = Some(AuditConfig {
            enabled: true,
            storage: AuditStorage::File(path.into()),
            include_user_context: true,
            tamper_proof: true,
        });
        self
    }
    
    /// Adds a custom panel
    #[must_use]
    pub fn with_panel(mut self, panel: PanelConfig) -> Self {
        self.custom_panels.push(panel);
        self
    }
    
    /// Sets the update interval
    #[must_use]
    pub fn with_update_interval(mut self, interval: u64) -> Self {
        self.update_interval = interval;
        self
    }
    
    /// Sets the retention period
    #[must_use]
    pub fn with_retention_period(mut self, period: u64) -> Self {
        self.retention_period = period;
        self
    }
} 