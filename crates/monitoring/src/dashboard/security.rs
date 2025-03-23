//! Security module for dashboard
//!
//! This module provides security features for the dashboard WebSocket server, including:
//! - TLS encryption
//! - Authentication and authorization
//! - Rate limiting
//! - Data masking
//! - Audit logging

use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::Instant;
use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;
use tracing::{error, debug};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde_json::Value;
use regex::{Regex, RegexBuilder};

/// TLS configuration for the WebSocket server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Path to the certificate file
    pub cert_path: PathBuf,
    /// Path to the private key file
    pub key_path: PathBuf,
    /// Minimum TLS version to support
    #[serde(default)]
    pub min_tls_version: TlsVersion,
    /// Cipher suite preferences
    #[serde(default)]
    pub cipher_preferences: CipherPreferences,
}

/// TLS version
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TlsVersion {
    /// TLS 1.2
    Tls12,
    /// TLS 1.3 (recommended)
    Tls13,
}

impl Default for TlsVersion {
    fn default() -> Self {
        Self::Tls13
    }
}

/// Cipher suite preferences
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CipherPreferences {
    /// Modern cipher suites (recommended)
    Modern,
    /// Intermediate cipher suites (wider compatibility)
    Intermediate,
    /// Legacy cipher suites (not recommended)
    Legacy,
}

impl Default for CipherPreferences {
    fn default() -> Self {
        Self::Modern
    }
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Authentication type
    pub auth_type: AuthType,
    /// Token expiration duration in seconds
    #[serde(default = "default_token_expiration")]
    pub token_expiration: u64,
    /// Whether to require re-authentication after expiration
    #[serde(default = "default_true")]
    pub require_reauth: bool,
    /// Allowed users with their roles
    #[serde(default)]
    pub users: HashMap<String, MonitoringRole>,
}

fn default_token_expiration() -> u64 {
    // 8 hours in seconds
    8 * 60 * 60
}

fn default_true() -> bool {
    true
}

/// Authentication type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthType {
    /// No authentication
    None,
    /// Basic authentication
    Basic,
    /// Bearer token authentication (JWT)
    Bearer,
    /// Custom authentication
    Custom,
}

/// User role for monitoring system
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MonitoringRole {
    /// Viewer role - can only view dashboards and metrics
    Viewer,
    /// Operator role - can acknowledge alerts and run predefined queries
    Operator,
    /// Administrator role - full access including configuration changes
    Administrator,
    /// Custom role with specific permissions
    Custom(Vec<Permission>),
}

/// Permission for custom roles
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Permission {
    /// View dashboards
    ViewDashboards,
    /// Modify dashboards
    ModifyDashboards,
    /// View metrics
    ViewMetrics,
    /// View alerts
    ViewAlerts,
    /// Acknowledge alerts
    AcknowledgeAlerts,
    /// Configure alerts
    ConfigureAlerts,
    /// Configure system
    ConfigureSystem,
    /// Access admin functions
    Admin,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum connections per IP address
    #[serde(default = "default_max_connections_per_ip")]
    pub max_connections_per_ip: usize,
    /// Maximum messages per minute per client
    #[serde(default = "default_max_messages_per_minute")]
    pub max_messages_per_minute: usize,
    /// Maximum subscription requests per minute per client
    #[serde(default = "default_max_subscription_requests")]
    pub max_subscription_requests_per_minute: usize,
}

fn default_max_connections_per_ip() -> usize {
    20
}

fn default_max_messages_per_minute() -> usize {
    300
}

fn default_max_subscription_requests() -> usize {
    50
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_connections_per_ip: default_max_connections_per_ip(),
            max_messages_per_minute: default_max_messages_per_minute(),
            max_subscription_requests_per_minute: default_max_subscription_requests(),
        }
    }
}

/// Data masking rule for sensitive information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskingRule {
    /// Regular expression pattern to match
    pub pattern: String,
    /// Replacement text
    pub replacement: String,
    /// Whether the pattern is case sensitive
    #[serde(default)]
    pub case_sensitive: bool,
    /// Compiled regex pattern
    #[serde(skip)]
    compiled: Option<Regex>,
}

impl MaskingRule {
    /// Creates a new masking rule
    pub fn new(pattern: &str, replacement: &str) -> Self {
        Self {
            pattern: pattern.to_string(),
            replacement: replacement.to_string(),
            case_sensitive: true,
            compiled: None,
        }
    }

    /// Compiles the regex pattern
    pub fn compile(&self) -> std::result::Result<Regex, regex::Error> {
        RegexBuilder::new(&self.pattern)
            .case_insensitive(!self.case_sensitive)
            .build()
    }

    /// Applies the masking rule to a string
    pub fn apply(&self, input: &str) -> std::result::Result<String, regex::Error> {
        let regex = self.compile()?;
        Ok(regex.replace_all(input, &self.replacement).to_string())
    }
}

/// Security logging configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecurityLoggingConfig {
    /// Whether to log authentication attempts
    #[serde(default = "default_true")]
    pub log_authentication_attempts: bool,
    /// Whether to log authorization failures
    #[serde(default = "default_true")]
    pub log_authorization_failures: bool,
    /// Whether to log configuration changes
    #[serde(default = "default_true")]
    pub log_configuration_changes: bool,
    /// Level of data access logging
    #[serde(default)]
    pub log_data_access: AccessLoggingLevel,
}

/// Access logging level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessLoggingLevel {
    /// No access logging
    None,
    /// Log access to sensitive data only
    Sensitive,
    /// Log all data access
    All,
}

impl Default for AccessLoggingLevel {
    fn default() -> Self {
        Self::Sensitive
    }
}

/// Audit configuration for security events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    /// Whether audit logging is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Where to store audit logs
    pub storage: AuditStorage,
    /// Whether to include user context in audit logs
    #[serde(default = "default_true")]
    pub include_user_context: bool,
    /// Whether to make audit logs tamper-proof
    #[serde(default = "default_true")]
    pub tamper_proof: bool,
}

/// Audit storage options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditStorage {
    /// Store audit logs in files
    File(PathBuf),
    /// Store audit logs in a database
    Database(String),
    /// Custom storage
    Custom,
}

/// JWT Claims for authentication
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Issued at timestamp
    pub iat: u64,
    /// Expiration timestamp
    pub exp: u64,
    /// Role
    pub role: String,
}

/// Rate limiter for WebSocket connections
#[derive(Debug)]
pub struct RateLimiter {
    /// Configuration
    config: RateLimitConfig,
    /// Connection counters per IP
    connections: Arc<RwLock<HashMap<String, usize>>>,
    /// Message counters per client
    messages: Arc<RwLock<HashMap<String, (usize, Instant)>>>,
    /// Subscription counters per client
    subscriptions: Arc<RwLock<HashMap<String, (usize, Instant)>>>,
}

impl RateLimiter {
    /// Creates a new rate limiter
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            connections: Arc::new(RwLock::new(HashMap::new())),
            messages: Arc::new(RwLock::new(HashMap::new())),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Checks if a connection from an IP is allowed
    pub async fn check_connection(&self, ip: &str) -> bool {
        let mut connections = self.connections.write().await;
        let count = connections.entry(ip.to_string()).or_insert(0);
        
        if *count >= self.config.max_connections_per_ip {
            false
        } else {
            *count += 1;
            true
        }
    }

    /// Releases a connection from an IP
    pub async fn release_connection(&self, ip: &str) {
        let mut connections = self.connections.write().await;
        if let Some(count) = connections.get_mut(ip) {
            if *count > 0 {
                *count -= 1;
            }
        }
    }

    /// Checks if a message from a client is allowed
    pub async fn check_message(&self, client_id: &str) -> bool {
        let mut messages = self.messages.write().await;
        let now = Instant::now();
        
        // Clean expired entries
        messages.retain(|_, (_, timestamp)| {
            now.duration_since(*timestamp).as_secs() < 60
        });
        
        let entry = messages.entry(client_id.to_string()).or_insert((0, now));
        
        // Reset counter if more than a minute has passed
        if now.duration_since(entry.1).as_secs() >= 60 {
            *entry = (1, now);
            true
        } else if entry.0 >= self.config.max_messages_per_minute {
            false
        } else {
            entry.0 += 1;
            true
        }
    }

    /// Checks if a subscription request from a client is allowed
    pub async fn check_subscription(&self, client_id: &str) -> bool {
        let mut subscriptions = self.subscriptions.write().await;
        let now = Instant::now();
        
        // Clean expired entries
        subscriptions.retain(|_, (_, timestamp)| {
            now.duration_since(*timestamp).as_secs() < 60
        });
        
        let entry = subscriptions.entry(client_id.to_string()).or_insert((0, now));
        
        // Reset counter if more than a minute has passed
        if now.duration_since(entry.1).as_secs() >= 60 {
            *entry = (1, now);
            true
        } else if entry.0 >= self.config.max_subscription_requests_per_minute {
            false
        } else {
            entry.0 += 1;
            true
        }
    }
}

/// Authentication manager for WebSocket connections
#[derive(Debug)]
pub struct AuthManager {
    /// Configuration
    config: AuthConfig,
    /// JWT secret for token validation
    jwt_secret: String,
}

impl AuthManager {
    /// Creates a new authentication manager
    pub fn new(config: AuthConfig, jwt_secret: String) -> Self {
        Self {
            config,
            jwt_secret,
        }
    }

    /// Validates a JWT token
    pub fn validate_token(&self, token: &str) -> std::result::Result<Claims, String> {
        let decoding_key = DecodingKey::from_secret(self.jwt_secret.as_bytes());
        let validation = Validation::default();
        
        match decode::<Claims>(token, &decoding_key, &validation) {
            Ok(token_data) => Ok(token_data.claims),
            Err(e) => Err(format!("Token validation error: {}", e)),
        }
    }

    /// Creates a JWT token
    pub fn create_token(&self, username: &str) -> std::result::Result<String, String> {
        let now = chrono::Utc::now().timestamp() as u64;
        let expiration = now + self.config.token_expiration;
        
        let role = match self.config.users.get(username) {
            Some(role) => match role {
                MonitoringRole::Viewer => "viewer",
                MonitoringRole::Operator => "operator",
                MonitoringRole::Administrator => "admin",
                MonitoringRole::Custom(_) => "custom",
            },
            None => return Err("User not found".to_string()),
        };
        
        let claims = Claims {
            sub: username.to_string(),
            iat: now,
            exp: expiration,
            role: role.to_string(),
        };
        
        let encoding_key = EncodingKey::from_secret(self.jwt_secret.as_bytes());
        encode(&Header::default(), &claims, &encoding_key)
            .map_err(|e| format!("Token creation error: {}", e))
    }

    /// Checks if a user has a specific permission
    pub fn has_permission(&self, username: &str, permission: &Permission) -> bool {
        match self.config.users.get(username) {
            Some(role) => match role {
                MonitoringRole::Viewer => matches!(
                    permission,
                    Permission::ViewDashboards | Permission::ViewMetrics | Permission::ViewAlerts
                ),
                MonitoringRole::Operator => matches!(
                    permission,
                    Permission::ViewDashboards | Permission::ViewMetrics | 
                    Permission::ViewAlerts | Permission::AcknowledgeAlerts |
                    Permission::ModifyDashboards
                ),
                MonitoringRole::Administrator => true,
                MonitoringRole::Custom(permissions) => permissions.contains(permission),
            },
            None => false,
        }
    }
}

/// Data masking manager for sensitive information
#[derive(Debug)]
pub struct DataMaskingManager {
    /// Masking rules
    rules: Vec<MaskingRule>,
    /// Compiled regular expressions
    compiled_rules: Vec<(Regex, String)>,
}

impl DataMaskingManager {
    /// Creates a new data masking manager
    pub fn new(rules: Vec<MaskingRule>) -> std::result::Result<Self, regex::Error> {
        let mut compiled_rules = Vec::new();
        
        for rule in &rules {
            compiled_rules.push((rule.compile()?, rule.replacement.clone()));
        }
        
        Ok(Self {
            rules,
            compiled_rules,
        })
    }

    /// Applies all masking rules to a string
    pub fn mask_string(&self, input: &str) -> String {
        let mut result = input.to_string();
        
        for (regex, replacement) in &self.compiled_rules {
            result = regex.replace_all(&result, replacement).to_string();
        }
        
        result
    }

    /// Applies all masking rules to a JSON value
    pub fn mask_json(&self, input: &Value) -> Value {
        match input {
            Value::String(s) => Value::String(self.mask_string(s)),
            Value::Array(arr) => {
                let masked_arr = arr.iter()
                    .map(|v| self.mask_json(v))
                    .collect();
                Value::Array(masked_arr)
            },
            Value::Object(obj) => {
                let mut masked_obj = serde_json::Map::new();
                for (k, v) in obj {
                    masked_obj.insert(k.clone(), self.mask_json(v));
                }
                Value::Object(masked_obj)
            },
            _ => input.clone(),
        }
    }
}

/// Audit logger for security events
#[derive(Debug)]
pub struct AuditLogger {
    /// Configuration
    config: AuditConfig,
    /// Database connection for audit storage (if using database)
    #[allow(dead_code)]
    db_connection: Option<String>,
}

impl AuditLogger {
    /// Creates a new audit logger
    pub fn new(config: AuditConfig) -> Self {
        let db_connection = match &config.storage {
            AuditStorage::Database(conn_str) => Some(conn_str.clone()),
            _ => None,
        };
        
        Self {
            config,
            db_connection,
        }
    }

    /// Logs an audit event
    pub async fn log_event(&self, event_type: &str, details: Value, username: Option<&str>) {
        if !self.config.enabled {
            return;
        }
        
        let timestamp = chrono::Utc::now();
        let mut event = serde_json::Map::new();
        
        event.insert("timestamp".to_string(), Value::String(timestamp.to_rfc3339()));
        event.insert("event_type".to_string(), Value::String(event_type.to_string()));
        
        if self.config.include_user_context {
            if let Some(user) = username {
                event.insert("username".to_string(), Value::String(user.to_string()));
            }
        }
        
        event.insert("details".to_string(), details);
        
        let event_json = Value::Object(event);
        
        match &self.config.storage {
            AuditStorage::File(path) => {
                // Ensure directory exists
                if let Some(parent) = path.parent() {
                    if !parent.exists() {
                        if let Err(e) = std::fs::create_dir_all(parent) {
                            error!("Failed to create audit log directory: {}", e);
                            return;
                        }
                    }
                }
                
                // Append to log file
                let result = tokio::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                    .await;
                
                match result {
                    Ok(mut file) => {
                        let log_entry = format!("{}\n", serde_json::to_string(&event_json).unwrap());
                        if let Err(e) = tokio::io::AsyncWriteExt::write_all(&mut file, log_entry.as_bytes()).await {
                            error!("Failed to write to audit log: {}", e);
                        }
                    },
                    Err(e) => {
                        error!("Failed to open audit log file: {}", e);
                    }
                }
            },
            AuditStorage::Database(_) => {
                // Database implementation would go here
                // For now, just log that we would store to database
                debug!("Would store audit event to database: {}", serde_json::to_string(&event_json).unwrap());
            },
            AuditStorage::Custom => {
                // Custom implementation would go here
                debug!("Would store audit event using custom storage: {}", serde_json::to_string(&event_json).unwrap());
            },
        }
    }
}

// Helper functions for security validation

/// Validates a TLS configuration
pub fn validate_tls_config(config: &TlsConfig) -> std::result::Result<(), String> {
    // Check if certificate file exists
    if !config.cert_path.exists() {
        return Err(format!("Certificate file not found: {:?}", config.cert_path));
    }
    
    // Check if key file exists
    if !config.key_path.exists() {
        return Err(format!("Key file not found: {:?}", config.key_path));
    }
    
    Ok(())
}

/// Origin verifier for controlling allowed domains
#[derive(Debug, Clone)]
pub struct OriginVerifier {
    /// List of allowed origins
    allowed_origins: HashSet<String>,
}

impl OriginVerifier {
    /// Creates a new origin verifier
    #[must_use]
    pub fn new(allowed_origins: Vec<String>) -> Self {
        let allowed_origins_set: HashSet<String> = allowed_origins.into_iter().collect();
        Self { allowed_origins: allowed_origins_set }
    }

    /// Checks if an origin is allowed
    pub fn is_allowed(&self, origin: &str) -> bool {
        // If empty, allow all origins
        if self.allowed_origins.is_empty() {
            return true;
        }
        
        self.allowed_origins.contains(origin)
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            auth_type: AuthType::None,
            token_expiration: 8 * 60 * 60, // 8 hours
            require_reauth: true,
            users: HashMap::new(),
        }
    }
} 