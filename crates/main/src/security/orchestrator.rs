//! # Security Orchestration System
//! 
//! This module provides unified security orchestration that coordinates:
//! - Authentication and authorization
//! - Rate limiting and DoS protection
//! - Input validation and sanitization
//! - Security monitoring and threat detection
//! - Automated security responses

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::error::PrimalError;
use crate::observability::CorrelationId;
use crate::shutdown::{ShutdownHandler, ShutdownPhase};

use super::rate_limiter::{ProductionRateLimiter, RateLimitConfig, RateLimitResult, EndpointType};
use super::input_validator::{ProductionInputValidator, InputValidationConfig, ValidationResult, InputType};
use super::monitoring::{SecurityMonitoringSystem, SecurityMonitoringConfig, SecurityEvent, SecurityEventType, EventSeverity};

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

/// Security policy definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
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

impl Default for SecurityPolicy {
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
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
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
#[derive(Debug, Clone, PartialEq)]
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

/// Production security orchestration system
pub struct SecurityOrchestrator {
    /// Configuration
    config: SecurityOrchestrationConfig,
    
    /// Rate limiter
    rate_limiter: Arc<ProductionRateLimiter>,
    
    /// Input validator
    input_validator: Arc<ProductionInputValidator>,
    
    /// Security monitoring system
    security_monitor: Arc<SecurityMonitoringSystem>,
    
    /// Active security responses
    active_responses: Arc<RwLock<HashMap<String, Vec<SecurityResponse>>>>,
    
    /// Violation counters by IP
    violation_counters: Arc<RwLock<HashMap<IpAddr, ViolationCounter>>>,
}

#[derive(Debug, Clone)]
struct ViolationCounter {
    total_violations: u32,
    recent_violations: u32,
    first_violation: SystemTime,
    last_violation: SystemTime,
    violation_types: HashMap<String, u32>,
}

impl SecurityOrchestrator {
    /// Create a new security orchestrator
    pub async fn new(config: SecurityOrchestrationConfig) -> Result<Self, PrimalError> {
        let rate_limiter = Arc::new(ProductionRateLimiter::new(config.rate_limiting.clone()));
        
        let input_validator = Arc::new(
            ProductionInputValidator::new(config.input_validation.clone())?
        );
        
        let security_monitor = Arc::new(
            SecurityMonitoringSystem::new(config.security_monitoring.clone())
        );
        
        // Start security monitoring
        security_monitor.start().await?;
        
        Ok(Self {
            config,
            rate_limiter,
            input_validator,
            security_monitor,
            active_responses: Arc::new(RwLock::new(HashMap::new())),
            violation_counters: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// Perform comprehensive security check
    pub async fn check_security(&self, request: SecurityCheckRequest) -> SecurityCheckResult {
        let mut security_events = Vec::new();
        let mut recommended_actions = Vec::new();
        let mut overall_risk = RiskLevel::Low;
        let mut allowed = true;
        let mut denial_reason = None;
        
        debug!(
            correlation_id = %request.correlation_id,
            client_ip = %request.client_ip,
            endpoint = %request.endpoint,
            operation = "security_check_start",
            "Starting comprehensive security check"
        );
        
        // 1. Check rate limits
        let rate_limit_result = self.rate_limiter.check_request(
            request.client_ip,
            request.endpoint_type.clone(),
            request.user_agent.clone(),
        ).await;
        
        if !rate_limit_result.allowed {
            allowed = false;
            denial_reason = Some("Rate limit exceeded".to_string());
            overall_risk = overall_risk.max(RiskLevel::Medium);
            
            // Record rate limit violation event
            security_events.push(SecurityEvent {
                event_id: Uuid::new_v4(),
                event_type: SecurityEventType::RateLimitViolation {
                    client_ip: request.client_ip.to_string(),
                    endpoint: request.endpoint.clone(),
                    violation_count: 1,
                },
                timestamp: SystemTime::now(),
                source_ip: request.client_ip.to_string(),
                user_agent: request.user_agent.clone(),
                correlation_id: request.correlation_id.clone(),
                severity: EventSeverity::Warning,
                metadata: HashMap::new(),
                source_component: "rate_limiter".to_string(),
            });
            
            recommended_actions.push("Consider blocking IP if violations continue".to_string());
        }
        
        // 2. Validate input data
        let mut validation_results = Vec::new();
        if let Some(input_data) = &request.input_data {
            for (field_name, field_value, input_type) in input_data {
                let validation_result = self.input_validator.validate_input(
                    field_value,
                    input_type.clone(),
                    Some(request.correlation_id.clone()),
                );
                
                if !validation_result.is_valid {
                    allowed = false;
                    if denial_reason.is_none() {
                        denial_reason = Some(format!("Invalid input in field: {}", field_name));
                    }
                    
                    overall_risk = overall_risk.max(match validation_result.risk_level {
                        super::input_validator::RiskLevel::Low => RiskLevel::Low,
                        super::input_validator::RiskLevel::Medium => RiskLevel::Medium,
                        super::input_validator::RiskLevel::High => RiskLevel::High,
                        super::input_validator::RiskLevel::Critical => RiskLevel::Critical,
                    });
                    
                    // Record input validation violation event
                    security_events.push(SecurityEvent {
                        event_id: Uuid::new_v4(),
                        event_type: SecurityEventType::InputValidationViolation {
                            client_ip: request.client_ip.to_string(),
                            violation_type: validation_result.violations.first()
                                .map(|v| format!("{:?}", v.violation_type))
                                .unwrap_or_else(|| "unknown".to_string()),
                            risk_level: format!("{:?}", validation_result.risk_level),
                        },
                        timestamp: SystemTime::now(),
                        source_ip: request.client_ip.to_string(),
                        user_agent: request.user_agent.clone(),
                        correlation_id: request.correlation_id.clone(),
                        severity: match validation_result.risk_level {
                            super::input_validator::RiskLevel::Low => EventSeverity::Info,
                            super::input_validator::RiskLevel::Medium => EventSeverity::Warning,
                            super::input_validator::RiskLevel::High => EventSeverity::High,
                            super::input_validator::RiskLevel::Critical => EventSeverity::Critical,
                        },
                        metadata: HashMap::new(),
                        source_component: "input_validator".to_string(),
                    });
                    
                    recommended_actions.push("Review input validation rules".to_string());
                }
                
                validation_results.push((field_name.clone(), validation_result));
            }
        }
        
        // 3. Record security events
        for event in &security_events {
            self.security_monitor.record_event(event.clone()).await;
        }
        
        // 4. Update violation counters
        if !allowed {
            self.update_violation_counter(request.client_ip, overall_risk.clone()).await;
        }
        
        // 5. Execute automated responses if enabled
        if self.config.enable_automated_response && !allowed {
            if let Some(response) = self.determine_automated_response(
                request.client_ip,
                &overall_risk,
            ).await {
                self.execute_automated_response(response).await;
            }
        }
        
        let result = SecurityCheckResult {
            allowed,
            denial_reason,
            rate_limit_result,
            validation_results,
            security_events,
            recommended_actions,
            risk_level: overall_risk,
        };
        
        if !result.allowed {
            warn!(
                correlation_id = %request.correlation_id,
                client_ip = %request.client_ip,
                denial_reason = result.denial_reason.as_deref().unwrap_or("unknown"),
                risk_level = ?result.risk_level,
                operation = "security_check_denied",
                "Security check denied request"
            );
        } else {
            debug!(
                correlation_id = %request.correlation_id,
                client_ip = %request.client_ip,
                operation = "security_check_allowed",
                "Security check allowed request"
            );
        }
        
        result
    }
    
    /// Update violation counter for IP
    async fn update_violation_counter(&self, client_ip: IpAddr, risk_level: RiskLevel) {
        let mut counters = self.violation_counters.write().await;
        let now = SystemTime::now();
        
        let counter = counters.entry(client_ip).or_insert_with(|| ViolationCounter {
            total_violations: 0,
            recent_violations: 0,
            first_violation: now,
            last_violation: now,
            violation_types: HashMap::new(),
        });
        
        counter.total_violations += 1;
        counter.recent_violations += 1;
        counter.last_violation = now;
        
        let violation_type = format!("{:?}", risk_level);
        *counter.violation_types.entry(violation_type).or_insert(0) += 1;
    }
    
    /// Determine appropriate automated response
    async fn determine_automated_response(
        &self,
        client_ip: IpAddr,
        risk_level: &RiskLevel,
    ) -> Option<SecurityResponse> {
        let counters = self.violation_counters.read().await;
        
        if let Some(counter) = counters.get(&client_ip) {
            let response_type = if counter.total_violations >= self.config.response_thresholds.permanent_block_threshold {
                ResponseType::PermanentBlock
            } else if counter.recent_violations >= self.config.response_thresholds.temp_block_threshold {
                ResponseType::TemporaryBlock
            } else if counter.total_violations >= self.config.response_thresholds.admin_alert_threshold {
                ResponseType::AdminAlert
            } else if *risk_level >= RiskLevel::High {
                ResponseType::Warning
            } else {
                ResponseType::Log
            };
            
            Some(SecurityResponse {
                response_type: response_type.clone(),
                target: client_ip.to_string(),
                duration: if response_type == ResponseType::TemporaryBlock {
                    Some(self.config.response_thresholds.temp_block_duration)
                } else {
                    None
                },
                details: format!(
                    "Automated response for {} violations from {}",
                    counter.total_violations, client_ip
                ),
                timestamp: SystemTime::now(),
            })
        } else {
            None
        }
    }
    
    /// Execute automated security response
    async fn execute_automated_response(&self, response: SecurityResponse) {
        match response.response_type {
            ResponseType::Log => {
                info!(
                    target = %response.target,
                    operation = "automated_security_response",
                    "Security incident logged: {}", response.details
                );
            }
            ResponseType::Warning => {
                warn!(
                    target = %response.target,
                    operation = "automated_security_response",
                    "Security warning issued: {}", response.details
                );
            }
            ResponseType::TemporaryBlock => {
                warn!(
                    target = %response.target,
                    duration_seconds = response.duration.map(|d| d.as_secs()),
                    operation = "automated_security_response",
                    "Temporary block initiated: {}", response.details
                );
                
                // Store the response
                let mut responses = self.active_responses.write().await;
                responses.entry(response.target.clone())
                    .or_insert_with(Vec::new)
                    .push(response);
            }
            ResponseType::PermanentBlock => {
                error!(
                    target = %response.target,
                    operation = "automated_security_response",
                    "Permanent block initiated: {}", response.details
                );
                
                // Store the response
                let mut responses = self.active_responses.write().await;
                responses.entry(response.target.clone())
                    .or_insert_with(Vec::new)
                    .push(response);
            }
            ResponseType::AdminAlert => {
                error!(
                    target = %response.target,
                    operation = "automated_security_response",
                    "ADMIN ALERT - {}", response.details
                );
                
                // In production, this would trigger external alerting systems
            }
            ResponseType::Escalate => {
                error!(
                    target = %response.target,
                    operation = "automated_security_response",
                    "ESCALATION - {}", response.details
                );
                
                // In production, this would escalate to external security systems
            }
        }
    }
    
    /// Get security statistics
    pub async fn get_security_statistics(&self) -> SecurityStatistics {
        let monitoring_stats = self.security_monitor.get_statistics().await;
        let rate_limit_stats = self.rate_limiter.get_statistics().await;
        let violation_counters = self.violation_counters.read().await;
        let active_responses = self.active_responses.read().await;
        
        SecurityStatistics {
            monitoring_stats,
            rate_limit_stats,
            total_tracked_ips: violation_counters.len(),
            active_security_responses: active_responses.values().map(|v| v.len()).sum(),
            high_risk_ips: violation_counters.iter()
                .filter(|(_, counter)| counter.recent_violations >= 5)
                .count(),
        }
    }
}

/// Comprehensive security statistics
#[derive(Debug, Clone, Serialize)]
pub struct SecurityStatistics {
    pub monitoring_stats: super::monitoring::SecurityMonitoringStats,
    pub rate_limit_stats: super::rate_limiter::RateLimitStatistics,
    pub total_tracked_ips: usize,
    pub active_security_responses: usize,
    pub high_risk_ips: usize,
}

#[async_trait::async_trait]
impl ShutdownHandler for SecurityOrchestrator {
    fn component_name(&self) -> &str {
        "security_orchestrator"
    }
    
    async fn shutdown(&self, phase: ShutdownPhase) -> Result<(), PrimalError> {
        match phase {
            ShutdownPhase::StopAccepting => {
                info!("Security orchestrator stopped accepting new requests");
                Ok(())
            }
            ShutdownPhase::DrainRequests => {
                // Allow current security checks to complete
                tokio::time::sleep(Duration::from_secs(2)).await;
                Ok(())
            }
            ShutdownPhase::CloseConnections => {
                Ok(())
            }
            ShutdownPhase::CleanupResources => {
                // Cleanup rate limiter data
                self.rate_limiter.cleanup_expired_data().await;
                Ok(())
            }
            ShutdownPhase::ShutdownTasks => {
                // Shutdown security monitoring
                self.security_monitor.shutdown(phase).await?;
                Ok(())
            }
            ShutdownPhase::FinalCleanup => {
                // Clear all data
                {
                    let mut responses = self.active_responses.write().await;
                    responses.clear();
                }
                {
                    let mut counters = self.violation_counters.write().await;
                    counters.clear();
                }
                info!("Security orchestrator final cleanup completed");
                Ok(())
            }
        }
    }
    
    async fn is_shutdown_complete(&self) -> bool {
        self.security_monitor.is_shutdown_complete().await
    }
    
    fn estimated_shutdown_time(&self) -> Duration {
        Duration::from_secs(15)
    }
} 