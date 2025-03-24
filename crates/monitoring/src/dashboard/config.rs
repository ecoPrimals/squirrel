//! Dashboard configuration
//!
//! This module defines the configuration structures for the dashboard
//! including server settings, UI configuration, and security settings.

use std::collections::HashSet;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};

use super::security::{TlsConfig, AuthConfig, RateLimitConfig, MaskingRule, AuditConfig, AuditStorage};

fn default_true() -> bool {
    true
}

/// Security settings for the dashboard
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecuritySettings {
    /// Allowed CORS origins
    pub allowed_origins: Vec<String>,
    
    /// Authentication settings
    pub auth: Option<AuthConfig>,
    
    /// Rate limiting settings
    pub rate_limit: Option<RateLimitConfig>,
    
    /// Data masking settings
    pub data_masking: Vec<MaskingRule>,
    
    /// Audit logging settings
    pub audit: Option<AuditConfig>,
    
    /// TLS configuration
    pub tls: Option<TlsConfig>,
}

/// Performance settings for the dashboard
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerformanceSettings {
    /// Whether to enable message compression
    pub compression_enabled: Option<bool>,
    
    /// Minimum size for compression in bytes
    pub min_compression_size: Option<usize>,
    
    /// Compression level (0-9)
    pub compression_level: Option<u32>,
    
    /// Whether to enable message batching
    pub batching_enabled: Option<bool>,
    
    /// Maximum number of messages per batch
    pub max_batch_size: Option<usize>,
    
    /// Maximum time to wait before sending a batch (in milliseconds)
    pub max_batch_interval: Option<u64>,
    
    /// Connection buffer size
    pub connection_buffer_size: Option<usize>,
    
    /// Maximum message size in bytes
    pub max_message_size: Option<usize>,
    
    /// Whether to enable WebSocket heartbeats
    pub enable_heartbeats: Option<bool>,
    
    /// Heartbeat interval in seconds
    pub heartbeat_interval: Option<u64>,
    
    /// Maximum number of concurrent connections
    pub max_connections: Option<usize>,
}

/// Dashboard configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DashboardConfig {
    /// Server settings
    pub server: Option<ServerSettings>,
    
    /// WebSocket settings
    pub websocket: Option<WebSocketSettings>,
    
    /// Security settings
    pub security: SecuritySettings,
    
    /// Performance settings
    pub performance: Option<PerformanceSettings>,
    
    /// Component settings
    pub components: Option<ComponentSettings>,
    
    /// Layout settings
    pub layout: Option<LayoutSettings>,
    
    /// Theme settings
    pub theme: Option<ThemeSettings>,
    
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

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerSettings {
    /// Host address to bind to
    pub host: String,
    /// Port number to use
    pub port: u16,
    /// URL path prefix
    pub path_prefix: Option<String>,
}

/// WebSocket configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WebSocketSettings {
    /// WebSocket endpoint
    pub endpoint: String,
    /// Ping interval in seconds
    pub ping_interval: u32,
    /// Connection timeout in seconds
    pub timeout: u32,
    /// Maximum connections
    pub max_connections: u32,
    /// Compression threshold
    pub compression_threshold: Option<usize>,
}

/// Component settings for the dashboard
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComponentSettings {
    /// Whether to show the metrics panel
    pub show_metrics: Option<bool>,
    
    /// Whether to show the alerts panel
    pub show_alerts: Option<bool>,
    
    /// Whether to show the health panel
    pub show_health: Option<bool>,
    
    /// Whether to show the network panel
    pub show_network: Option<bool>,
    
    /// Whether to show the analytics panel
    pub show_analytics: Option<bool>,
}

/// Layout settings for the dashboard
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LayoutSettings {
    /// Dashboard layout type
    pub layout_type: String,
    /// Default panel arrangement
    pub default_panels: Vec<String>,
    /// User customizable
    pub customizable: bool,
}

/// Theme settings for the dashboard
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThemeSettings {
    /// Theme name
    pub name: String,
    /// Primary color
    pub primary_color: String,
    /// Secondary color
    pub secondary_color: String,
    /// Dark mode enabled
    pub dark_mode: bool,
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
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the server host
    pub fn with_host(mut self, host: impl Into<String>) -> Self {
        let server = self.server.get_or_insert_with(ServerSettings::default);
        server.host = host.into();
        self
    }

    /// Sets the server port
    pub fn with_port(mut self, port: u16) -> Self {
        let server = self.server.get_or_insert_with(ServerSettings::default);
        server.port = port;
        self
    }

    /// Sets the WebSocket max connections
    pub fn with_max_connections(mut self, max_connections: u32) -> Self {
        let websocket = self.websocket.get_or_insert_with(WebSocketSettings::default);
        websocket.max_connections = max_connections;
        self
    }

    /// Sets the WebSocket compression threshold
    pub fn with_compression_threshold(mut self, threshold: usize) -> Self {
        let websocket = self.websocket.get_or_insert_with(WebSocketSettings::default);
        websocket.compression_threshold = Some(threshold);
        self
    }

    /// Enables TLS with the given certificate and key files
    pub fn with_tls(mut self, cert_path: impl Into<PathBuf>, key_path: impl Into<PathBuf>) -> Self {
        self.security.tls = Some(TlsConfig {
            cert_path: cert_path.into(),
            key_path: key_path.into(),
            min_tls_version: super::security::TlsVersion::default(),
            cipher_preferences: super::security::CipherPreferences::default(),
        });
        self
    }

    /// Sets the authentication configuration
    pub fn with_auth(mut self, auth: AuthConfig) -> Self {
        self.security.auth = Some(auth);
        self
    }

    /// Sets the allowed CORS origins
    pub fn with_allowed_origins(mut self, origins: Vec<String>) -> Self {
        self.security.allowed_origins = origins;
        self
    }

    /// Adds a data masking rule
    pub fn add_masking_rule(mut self, pattern: &str, replacement: &str) -> Self {
        self.security.data_masking.push(
            MaskingRule::new(pattern, replacement)
        );
        self
    }

    /// Enables audit logging
    pub fn with_audit_logging(mut self, path: impl Into<PathBuf>) -> Self {
        self.security.audit = Some(AuditConfig {
            enabled: true,
            storage: AuditStorage::File(path.into()),
            include_user_context: true,
            tamper_proof: true,
        });
        self
    }

    /// Sets the performance settings for compression
    pub fn with_compression(mut self, enable: bool, min_size: usize, level: u32) -> Self {
        let performance = self.performance.get_or_insert_with(PerformanceSettings::default);
        performance.compression_enabled = Some(enable);
        performance.min_compression_size = Some(min_size);
        performance.compression_level = Some(level);
        self
    }

    /// Sets the performance settings for batching
    pub fn with_batching(mut self, enable: bool, max_size: usize, max_interval: u64) -> Self {
        let performance = self.performance.get_or_insert_with(PerformanceSettings::default);
        performance.batching_enabled = Some(enable);
        performance.max_batch_size = Some(max_size);
        performance.max_batch_interval = Some(max_interval);
        self
    }

    /// Sets the theme
    pub fn with_theme(mut self, name: &str, primary_color: &str, secondary_color: &str, dark_mode: bool) -> Self {
        let theme = self.theme.get_or_insert_with(ThemeSettings::default);
        theme.name = name.to_string();
        theme.primary_color = primary_color.to_string();
        theme.secondary_color = secondary_color.to_string();
        theme.dark_mode = dark_mode;
        self
    }

    /// Sets the layout
    pub fn with_layout(mut self, layout_type: &str, default_panels: Vec<String>, customizable: bool) -> Self {
        let layout = self.layout.get_or_insert_with(LayoutSettings::default);
        layout.layout_type = layout_type.to_string();
        layout.default_panels = default_panels;
        layout.customizable = customizable;
        self
    }

    /// Sets which components to show
    pub fn with_components(mut self, metrics: bool, alerts: bool, health: bool, network: bool, analytics: bool) -> Self {
        let components = self.components.get_or_insert_with(ComponentSettings::default);
        components.show_metrics = Some(metrics);
        components.show_alerts = Some(alerts);
        components.show_health = Some(health);
        components.show_network = Some(network);
        components.show_analytics = Some(analytics);
        self
    }
} 