// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Universal Security Client Types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ============================================================================
// CONFIGURATION TYPES
// ============================================================================

/// Configuration for universal security client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityClientConfig {
    /// Timeout for security operations
    pub operation_timeout: std::time::Duration,

    /// Maximum retries for failed operations
    pub max_retries: u32,

    /// Preferred security capabilities
    pub preferred_capabilities: Vec<SecurityCapabilityPreference>,

    /// Security policy requirements
    pub policy_requirements: SecurityPolicyRequirements,

    /// Compliance requirements
    pub compliance_requirements: ComplianceRequirements,
}

/// Security capability preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityCapabilityPreference {
    /// Capability type
    pub capability: SecurityCapabilityType,

    /// Priority weight (0.0 - 1.0)
    pub weight: f64,

    /// Required vs optional
    pub required: bool,
}

/// Types of security capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityCapabilityType {
    /// Authentication services
    Authentication {
        /// Supported auth methods
        methods: Vec<String>,
        /// Whether MFA is supported
        multi_factor: bool,
        /// Whether biometric auth is supported
        biometric_support: bool,
    },

    /// Authorization and access control
    Authorization {
        /// RBAC support
        rbac_support: bool,
        /// ABAC support
        abac_support: bool,
        /// Policy engine identifiers
        policy_engines: Vec<String>,
    },

    /// Encryption and cryptography
    Encryption {
        /// Supported algorithms
        algorithms: Vec<String>,
        /// Key management support
        key_management: bool,
        /// HSM support
        hardware_security_modules: bool,
    },

    /// Threat detection and response
    ThreatDetection {
        /// Behavioral analysis support
        behavioral_analysis: bool,
        /// Anomaly detection support
        anomaly_detection: bool,
        /// Real-time monitoring support
        real_time_monitoring: bool,
    },

    /// Compliance and audit
    Compliance {
        /// Compliance framework identifiers
        frameworks: Vec<String>,
        /// Audit logging support
        audit_logging: bool,
        /// Reporting support
        reporting: bool,
    },
}

/// Trust levels for security providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrustLevel {
    /// Minimal trust - basic verification
    Minimal,

    /// Standard trust - normal operations
    Standard,

    /// High trust - sensitive operations
    High,

    /// Maximum trust - critical operations
    Maximum,
}

/// Security policy requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicyRequirements {
    /// Data classification handling
    pub data_classification: Vec<String>,

    /// Access control policies
    pub access_control: AccessControlPolicy,

    /// Encryption requirements
    pub encryption_policy: EncryptionPolicy,

    /// Audit requirements
    pub audit_policy: AuditPolicy,
}

/// Access control policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlPolicy {
    /// Default access level
    pub default_access: AccessLevel,

    /// Role-based access control
    pub rbac_enabled: bool,

    /// Attribute-based access control
    pub abac_enabled: bool,

    /// Zero-trust architecture
    pub zero_trust: bool,
}

/// Access levels for security decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessLevel {
    /// No access
    None,

    /// Read-only access
    ReadOnly,

    /// Read-write access
    ReadWrite,

    /// Administrative access
    Admin,

    /// Full access
    Full,
}

/// Encryption policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionPolicy {
    /// Data at rest encryption
    pub data_at_rest: bool,

    /// Data in transit encryption
    pub data_in_transit: bool,

    /// Data in use encryption
    pub data_in_use: bool,

    /// Key rotation frequency
    pub key_rotation_days: u32,

    /// Minimum key strength
    pub min_key_strength: u32,
}

/// Audit policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditPolicy {
    /// Log all security events
    pub log_all_events: bool,

    /// Log retention period
    pub retention_days: u32,

    /// Real-time alerting
    pub real_time_alerts: bool,

    /// Compliance reporting
    pub compliance_reporting: bool,
}

/// Compliance requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRequirements {
    /// Required compliance frameworks
    pub frameworks: Vec<ComplianceFramework>,

    /// Data residency requirements
    pub data_residency: Option<String>,

    /// Regulatory requirements
    pub regulatory: Vec<String>,
}

/// Compliance frameworks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceFramework {
    /// SOC 2 Type II
    Soc2,

    /// ISO 27001
    Iso27001,

    /// GDPR (General Data Protection Regulation)
    Gdpr,

    /// HIPAA (Health Insurance Portability and Accountability Act)
    Hipaa,

    /// PCI DSS (Payment Card Industry Data Security Standard)
    PciDss,

    /// NIST Cybersecurity Framework
    NistCsf,

    /// Custom framework
    Custom(String),
}

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

/// Universal security request - AI-first design
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalSecurityRequest {
    /// Unique request identifier
    pub request_id: Uuid,

    /// Operation type
    pub operation: SecurityOperation,

    /// Security context
    pub security_context: SecurityContext,

    /// Request payload
    pub payload: SecurityPayload,

    /// Required trust level
    pub required_trust_level: TrustLevel,

    /// AI context for intelligent routing
    pub ai_context: AISecurityContext,

    /// Request metadata
    pub metadata: HashMap<String, String>,
}

/// Types of security operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityOperation {
    /// Authenticate user
    Authenticate {
        /// Identity to authenticate
        identity: String,
        /// Credentials for authentication
        credentials: HashMap<String, String>,
    },

    /// Authorize access
    Authorize {
        /// Subject requesting access
        subject: String,
        /// Resource being accessed
        resource: String,
        /// Action being performed
        action: String,
    },

    /// Encrypt data
    Encrypt {
        /// Encryption algorithm
        algorithm: String,
        /// Optional key ID
        key_id: Option<String>,
    },

    /// Decrypt data
    Decrypt {
        /// Decryption algorithm
        algorithm: String,
        /// Key ID for decryption
        key_id: String,
    },

    /// Analyze threat
    AnalyzeThreat {
        /// Event data to analyze
        event_data: Vec<u8>,
        /// Analysis context
        context: HashMap<String, String>,
    },

    /// Audit log
    AuditLog {
        /// Event type
        event_type: String,
        /// Severity level
        severity: String,
    },
}

/// Security context for operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    /// User identifier
    pub user_id: String,

    /// Session identifier
    pub session_id: String,

    /// IP address
    pub ip_address: String,

    /// User agent
    pub user_agent: String,

    /// Security clearance level
    pub clearance_level: String,

    /// Additional context
    pub additional_context: HashMap<String, String>,
}

/// Security payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPayload {
    /// Data to process
    pub data: Option<Vec<u8>>,

    /// Security parameters
    pub parameters: HashMap<String, serde_json::Value>,

    /// Policy overrides
    pub policy_overrides: Option<HashMap<String, String>>,

    /// Compliance requirements
    pub compliance_tags: Vec<String>,
}

/// AI context for intelligent security routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AISecurityContext {
    /// Risk assessment
    pub risk_assessment: RiskLevel,

    /// Threat intelligence
    pub threat_intelligence: Vec<ThreatIndicator>,

    /// Behavioral analysis
    pub behavioral_analysis: BehavioralProfile,

    /// Context awareness
    pub context_awareness: ContextAwareness,
}

/// Risk levels for security operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Very low risk
    VeryLow,

    /// Low risk
    Low,

    /// Medium risk
    Medium,

    /// High risk
    High,

    /// Critical risk
    Critical,
}

/// Threat indicators for security analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatIndicator {
    /// Indicator type
    pub indicator_type: String,

    /// Indicator value
    pub value: String,

    /// Confidence level
    pub confidence: f64,

    /// Source
    pub source: String,
}

/// Behavioral profile for user analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralProfile {
    /// Normal access patterns
    pub normal_patterns: Vec<String>,

    /// Anomaly score (0.0 - 1.0)
    pub anomaly_score: f64,

    /// Historical behavior
    pub historical_behavior: HashMap<String, f64>,
}

/// Context awareness for security decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextAwareness {
    /// Time-based context
    pub temporal_context: TemporalContext,

    /// Location-based context
    pub location_context: LocationContext,

    /// Device-based context
    pub device_context: DeviceContext,
}

/// Temporal context for security
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalContext {
    /// Normal access hours
    pub normal_hours: Vec<u8>,

    /// Current time anomaly score
    pub time_anomaly_score: f64,

    /// Frequency analysis
    pub frequency_analysis: HashMap<String, f64>,
}

/// Location context for security
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationContext {
    /// Allowed locations
    pub allowed_locations: Vec<String>,

    /// Current location risk
    pub location_risk_score: f64,

    /// Travel patterns
    pub travel_patterns: Vec<String>,
}

/// Device context for security
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceContext {
    /// Trusted devices
    pub trusted_devices: Vec<String>,

    /// Device risk score
    pub device_risk_score: f64,

    /// Device fingerprinting
    pub device_fingerprint: HashMap<String, String>,
}

/// Universal security response - AI-first design
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalSecurityResponse {
    /// Request identifier
    pub request_id: Uuid,

    /// Operation success
    pub success: bool,

    /// Security decision
    pub decision: SecurityDecision,

    /// Response data
    pub data: Option<Vec<u8>>,

    /// Provider that handled the request
    pub provider_id: String,

    /// Security metrics
    pub security_metrics: SecurityMetrics,

    /// AI insights and recommendations
    pub ai_insights: AISecurityInsights,

    /// Error information (if applicable)
    pub error: Option<String>,
}

/// Security decision result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityDecision {
    /// Decision outcome
    pub outcome: DecisionOutcome,

    /// Confidence in decision
    pub confidence: f64,

    /// Risk score
    pub risk_score: f64,

    /// Decision factors
    pub factors: Vec<DecisionFactor>,

    /// Recommended actions
    pub recommended_actions: Vec<String>,
}

/// Security decision outcomes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecisionOutcome {
    /// Allow the operation
    Allow,

    /// Deny the operation
    Deny,

    /// Allow with conditions
    AllowWithConditions {
        /// Conditions that must be met
        conditions: Vec<String>,
    },

    /// Require additional authentication
    RequireAdditionalAuth {
        /// Required auth methods
        methods: Vec<String>,
    },

    /// Monitor and allow
    MonitorAndAllow,
}

/// Factors that influenced the security decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionFactor {
    /// Factor name
    pub name: String,

    /// Factor weight in decision
    pub weight: f64,

    /// Factor value
    pub value: serde_json::Value,

    /// Impact on decision
    pub impact: String,
}

/// Security metrics for the operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityMetrics {
    /// Processing time
    pub processing_time: std::time::Duration,

    /// Policy evaluations performed
    pub policy_evaluations: u32,

    /// Security events generated
    pub events_generated: u32,

    /// Threat indicators processed
    pub threat_indicators: u32,

    /// Provider security score
    pub provider_security_score: f64,
}

/// AI insights and recommendations for security
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AISecurityInsights {
    /// Confidence in security decision
    pub confidence_score: f64,

    /// Threat analysis
    pub threat_analysis: ThreatAnalysis,

    /// Security recommendations
    pub security_recommendations: Vec<String>,

    /// Risk mitigation strategies
    pub risk_mitigation: Vec<String>,

    /// Behavioral insights
    pub behavioral_insights: Vec<String>,
}

/// Threat analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatAnalysis {
    /// Detected threats
    pub detected_threats: Vec<String>,

    /// Threat severity scores
    pub severity_scores: HashMap<String, f64>,

    /// Attack patterns
    pub attack_patterns: Vec<String>,

    /// Recommended countermeasures
    pub countermeasures: Vec<String>,
}

// ============================================================================
// DEFAULT IMPLEMENTATIONS
// ============================================================================

impl Default for SecurityClientConfig {
    fn default() -> Self {
        Self {
            operation_timeout: std::time::Duration::from_secs(30),
            max_retries: 3,
            preferred_capabilities: vec![SecurityCapabilityPreference {
                capability: SecurityCapabilityType::Authentication {
                    methods: vec!["password".to_string(), "mfa".to_string()],
                    multi_factor: true,
                    biometric_support: false,
                },
                weight: 0.9,
                required: true,
            }],
            policy_requirements: SecurityPolicyRequirements {
                data_classification: vec!["public".to_string(), "internal".to_string()],
                access_control: AccessControlPolicy {
                    default_access: AccessLevel::None,
                    rbac_enabled: true,
                    abac_enabled: false,
                    zero_trust: true,
                },
                encryption_policy: EncryptionPolicy {
                    data_at_rest: true,
                    data_in_transit: true,
                    data_in_use: false,
                    key_rotation_days: 90,
                    min_key_strength: 256,
                },
                audit_policy: AuditPolicy {
                    log_all_events: true,
                    retention_days: 365,
                    real_time_alerts: true,
                    compliance_reporting: true,
                },
            },
            compliance_requirements: ComplianceRequirements {
                frameworks: vec![ComplianceFramework::Soc2],
                data_residency: None,
                regulatory: vec![],
            },
        }
    }
}
