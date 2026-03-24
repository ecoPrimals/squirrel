// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Security orchestration type definitions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::time::{Duration, SystemTime};

use crate::observability::CorrelationId;

use super::super::input_validator::{InputType, InputValidationConfig, ValidationResult};
use super::super::monitoring::{SecurityEvent, SecurityMonitoringConfig, SecurityMonitoringStats};
use super::super::rate_limiter::{
    EndpointType, RateLimitConfig, RateLimitResult, RateLimitStatistics,
};

/// Security orchestration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityOrchestrationConfig {
    /// Rate limiting configuration
    pub rate_limiting: RateLimitConfig,

    /// Input validation configuration
    pub input_validation: InputValidationConfig,

    /// Security monitoring configuration
    pub security_monitoring: SecurityMonitoringConfig,

    /// Enable automated threat response
    pub enable_automated_response: bool,

    /// Response escalation thresholds
    pub response_thresholds: ResponseThresholds,

    /// Security policies
    pub security_policies: HashMap<String, SecurityPolicy>,
}

impl Default for SecurityOrchestrationConfig {
    fn default() -> Self {
        let mut security_policies = HashMap::new();
        security_policies.insert("default".to_string(), SecurityPolicy::default());

        Self {
            rate_limiting: RateLimitConfig::default(),
            input_validation: InputValidationConfig::default(),
            security_monitoring: SecurityMonitoringConfig::default(),
            enable_automated_response: true,
            response_thresholds: ResponseThresholds::default(),
            security_policies,
        }
    }
}

/// Response escalation thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseThresholds {
    /// Number of violations before temporary IP block
    pub temp_block_threshold: u32,

    /// Duration of temporary IP block
    pub temp_block_duration: Duration,

    /// Number of violations before permanent IP block
    pub permanent_block_threshold: u32,

    /// Number of violations before alerting administrators
    pub admin_alert_threshold: u32,
}

impl Default for ResponseThresholds {
    fn default() -> Self {
        Self {
            temp_block_threshold: 10,
            temp_block_duration: Duration::from_secs(300), // 5 minutes
            permanent_block_threshold: 50,
            admin_alert_threshold: 5,
        }
    }
}

/// Security policy definition for orchestration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorSecurityPolicy {
    /// Policy name
    pub name: String,

    /// Require authentication
    pub require_authentication: bool,

    /// Minimum authorization level required
    pub min_authorization_level: String,

    /// Custom rate limits for this policy
    pub custom_rate_limits: Option<RateLimitConfig>,

    /// Custom input validation rules
    pub custom_validation_rules: Option<InputValidationConfig>,

    /// Allowed IP ranges (CIDR notation)
    pub allowed_ip_ranges: Vec<String>,

    /// Blocked IP ranges (CIDR notation)
    pub blocked_ip_ranges: Vec<String>,
}

impl Default for OrchestratorSecurityPolicy {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            require_authentication: false,
            min_authorization_level: "none".to_string(),
            custom_rate_limits: None,
            custom_validation_rules: None,
            allowed_ip_ranges: vec![],
            blocked_ip_ranges: vec![],
        }
    }
}

/// Type alias for backward compatibility with existing API.
pub type SecurityPolicy = OrchestratorSecurityPolicy;

/// Comprehensive security check request
#[derive(Debug, Clone)]
pub struct SecurityCheckRequest {
    /// Client IP address
    pub client_ip: IpAddr,

    /// User agent string
    pub user_agent: Option<String>,

    /// Endpoint being accessed
    pub endpoint: String,

    /// Endpoint type for rate limiting
    pub endpoint_type: EndpointType,

    /// Input data to validate
    pub input_data: Option<Vec<(String, String, InputType)>>,

    /// User ID if authenticated
    pub user_id: Option<String>,

    /// Session ID if available
    pub session_id: Option<String>,

    /// Security policy to apply
    pub policy_name: Option<String>,

    /// Correlation ID for tracking
    pub correlation_id: CorrelationId,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Security check result
#[derive(Debug, Clone)]
pub struct SecurityCheckResult {
    /// Whether the request should be allowed
    pub allowed: bool,

    /// Reason for denial (if not allowed)
    pub denial_reason: Option<String>,

    /// Rate limiting result
    pub rate_limit_result: RateLimitResult,

    /// Input validation results
    pub validation_results: Vec<(String, ValidationResult)>,

    /// Security events generated
    pub security_events: Vec<SecurityEvent>,

    /// Recommended actions
    pub recommended_actions: Vec<String>,

    /// Overall risk assessment
    pub risk_level: RiskLevel,
}

/// Risk level assessment for security events
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum RiskLevel {
    /// Low risk - informational
    Low,
    /// Medium risk - attention recommended
    Medium,
    /// High risk - immediate action needed
    High,
    /// Critical risk - urgent response required
    Critical,
}

/// Automated security response
#[derive(Debug, Clone)]
pub struct SecurityResponse {
    /// Response type
    pub response_type: ResponseType,

    /// Target of the response (IP, user, etc.)
    pub target: String,

    /// Duration of the response
    pub duration: Option<Duration>,

    /// Response details
    pub details: String,

    /// Timestamp of response
    pub timestamp: SystemTime,
}

/// Types of automated security responses
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseType {
    /// Log the incident
    Log,

    /// Issue a warning
    Warning,

    /// Temporarily block IP address
    TemporaryBlock,

    /// Permanently block IP address
    PermanentBlock,

    /// Alert administrators
    AdminAlert,

    /// Escalate to external security system
    Escalate,
}

/// Comprehensive security statistics
#[derive(Debug, Clone, Serialize)]
pub struct SecurityStatistics {
    /// Security monitoring statistics.
    pub monitoring_stats: SecurityMonitoringStats,
    /// Rate limiting statistics.
    pub rate_limit_stats: RateLimitStatistics,
    /// Total number of tracked IP addresses.
    pub total_tracked_ips: usize,
    /// Number of active security responses in progress.
    pub active_security_responses: usize,
    /// Number of IPs flagged as high risk.
    pub high_risk_ips: usize,
}

/// Violation counter for IP tracking (internal)
#[derive(Debug, Clone)]
pub struct ViolationCounter {
    /// Total number of violations from this IP
    pub total_violations: u32,
    /// Recent violations within the tracking window
    pub recent_violations: u32,
    /// Timestamp of first violation
    pub first_violation: SystemTime,
    /// Timestamp of most recent violation
    pub last_violation: SystemTime,
    /// Count per violation type
    pub violation_types: HashMap<String, u32>,
}
