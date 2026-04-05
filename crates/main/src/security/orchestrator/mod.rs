// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! # Security Orchestration System
//!
//! This module provides unified security orchestration that coordinates:
//! - Authentication and authorization
//! - Rate limiting and `DoS` protection
//! - Input validation and sanitization
//! - Security monitoring and threat detection
//! - Automated security responses

mod types;

pub use types::{
    OrchestratorSecurityPolicy, ResponseThresholds, ResponseType, RiskLevel, SecurityCheckRequest,
    SecurityCheckResult, SecurityOrchestrationConfig, SecurityPolicy, SecurityResponse,
    SecurityStatistics, ViolationCounter,
};

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::error::PrimalError;
use crate::shutdown::{ShutdownHandler, ShutdownPhase};

use super::input_validator::ProductionInputValidator;
use super::monitoring::{
    EventSeverity, SecurityEvent, SecurityEventType, SecurityMonitoringSystem,
};
use super::rate_limiter::ProductionRateLimiter;

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

impl SecurityOrchestrator {
    /// Create a new security orchestrator
    pub async fn new(config: SecurityOrchestrationConfig) -> Result<Self, PrimalError> {
        let rate_limiter = Arc::new(ProductionRateLimiter::new(config.rate_limiting.clone()));

        let input_validator = Arc::new(ProductionInputValidator::new(
            config.input_validation.clone(),
        )?);

        let security_monitor = Arc::new(SecurityMonitoringSystem::new(
            config.security_monitoring.clone(),
        ));

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
    #[expect(
        clippy::too_many_lines,
        reason = "Orchestration logic; refactor planned"
    )]
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
        let rate_limit_result = self
            .rate_limiter
            .check_request(
                request.client_ip,
                request.endpoint_type,
                request.user_agent.as_deref(),
            )
            .await;

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
                timestamp: std::time::SystemTime::now(),
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
                    *input_type,
                    Some(request.correlation_id.clone()),
                );

                if !validation_result.is_valid {
                    allowed = false;
                    if denial_reason.is_none() {
                        denial_reason = Some(format!("Invalid input in field: {field_name}"));
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
                            violation_type: validation_result.violations.first().map_or_else(
                                || "unknown".to_string(),
                                |v| format!("{:?}", v.violation_type),
                            ),
                            risk_level: format!("{:?}", validation_result.risk_level),
                        },
                        timestamp: std::time::SystemTime::now(),
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
            self.update_violation_counter(request.client_ip, overall_risk)
                .await;
        }

        // 5. Execute automated responses if enabled
        if self.config.enable_automated_response
            && !allowed
            && let Some(response) = self
                .determine_automated_response(request.client_ip, &overall_risk)
                .await
        {
            self.execute_automated_response(response).await;
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

        if result.allowed {
            debug!(
                correlation_id = %request.correlation_id,
                client_ip = %request.client_ip,
                operation = "security_check_allowed",
                "Security check allowed request"
            );
        } else {
            warn!(
                correlation_id = %request.correlation_id,
                client_ip = %request.client_ip,
                denial_reason = result.denial_reason.as_deref().unwrap_or("unknown"),
                risk_level = ?result.risk_level,
                operation = "security_check_denied",
                "Security check denied request"
            );
        }

        result
    }

    /// Update violation counter for IP
    async fn update_violation_counter(&self, client_ip: IpAddr, risk_level: RiskLevel) {
        let mut counters = self.violation_counters.write().await;
        let now = std::time::SystemTime::now();

        let counter = counters
            .entry(client_ip)
            .or_insert_with(|| ViolationCounter {
                total_violations: 0,
                recent_violations: 0,
                first_violation: now,
                last_violation: now,
                violation_types: HashMap::new(),
            });

        counter.total_violations += 1;
        counter.recent_violations += 1;
        counter.last_violation = now;

        let violation_type = format!("{risk_level:?}");
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
            let response_type = if counter.total_violations
                >= self.config.response_thresholds.permanent_block_threshold
            {
                ResponseType::PermanentBlock
            } else if counter.recent_violations
                >= self.config.response_thresholds.temp_block_threshold
            {
                ResponseType::TemporaryBlock
            } else if counter.total_violations
                >= self.config.response_thresholds.admin_alert_threshold
            {
                ResponseType::AdminAlert
            } else if *risk_level >= RiskLevel::High {
                ResponseType::Warning
            } else {
                ResponseType::Log
            };

            Some(SecurityResponse {
                response_type,
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
                timestamp: std::time::SystemTime::now(),
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
                responses
                    .entry(response.target.clone())
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
                responses
                    .entry(response.target.clone())
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
            active_security_responses: active_responses.values().map(std::vec::Vec::len).sum(),
            high_risk_ips: violation_counters
                .iter()
                .filter(|(_, counter)| counter.recent_violations >= 5)
                .count(),
        }
    }
}

impl ShutdownHandler for SecurityOrchestrator {
    fn component_name(&self) -> &'static str {
        "security_orchestrator"
    }

    async fn shutdown(&self, phase: ShutdownPhase) -> Result<(), PrimalError> {
        match phase {
            ShutdownPhase::StopAccepting => {
                info!("Security orchestrator stopped accepting new requests");
                Ok(())
            }
            ShutdownPhase::DrainRequests => {
                // Evolution: Properly wait for active security checks to complete
                // instead of arbitrary sleep
                let timeout_duration = Duration::from_secs(30);
                let start = std::time::Instant::now();

                loop {
                    let active_count = self.active_responses.read().await.len();

                    if active_count == 0 {
                        info!("All active security responses completed");
                        break;
                    }

                    if start.elapsed() > timeout_duration {
                        warn!(
                            "Shutdown timeout reached with {} active responses remaining",
                            active_count
                        );
                        break;
                    }

                    debug!(
                        "Waiting for {} active security responses to complete",
                        active_count
                    );

                    // Yield to allow in-flight tasks to make progress
                    tokio::task::yield_now().await;
                }

                Ok(())
            }
            ShutdownPhase::CloseConnections => Ok(()),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observability::CorrelationId;
    use crate::security::input_validator::InputType;
    use crate::security::rate_limiter::EndpointType;
    use std::net::IpAddr;

    fn create_test_config() -> SecurityOrchestrationConfig {
        let mut config = SecurityOrchestrationConfig::default();
        // Disable real-time monitoring to avoid background tasks in tests
        config.security_monitoring.enable_real_time_monitoring = false;
        config
    }

    #[tokio::test]
    async fn test_orchestrator_creation() {
        let config = create_test_config();
        let orchestrator = SecurityOrchestrator::new(config).await;
        assert!(orchestrator.is_ok());
        let orchestrator = orchestrator.expect("should succeed");
        let stats = orchestrator.get_security_statistics().await;
        assert_eq!(stats.total_tracked_ips, 0);
        assert_eq!(stats.active_security_responses, 0);
    }

    #[tokio::test]
    async fn test_security_check_allowed_request() {
        let config = create_test_config();
        let orchestrator = SecurityOrchestrator::new(config)
            .await
            .expect("should succeed");

        let request = SecurityCheckRequest {
            client_ip: "127.0.0.1".parse().expect("should succeed"),
            user_agent: Some("test-agent".to_string()),
            endpoint: "/api/test".to_string(),
            endpoint_type: EndpointType::Api,
            input_data: None,
            user_id: None,
            session_id: None,
            policy_name: None,
            correlation_id: CorrelationId::new(),
            metadata: HashMap::new(),
        };

        let result = orchestrator.check_security(request).await;
        assert!(result.allowed);
        assert!(result.denial_reason.is_none());
        assert_eq!(result.risk_level, RiskLevel::Low);
        assert!(result.security_events.is_empty());
    }

    #[tokio::test]
    async fn test_security_check_with_valid_input() {
        let config = create_test_config();
        let orchestrator = SecurityOrchestrator::new(config)
            .await
            .expect("should succeed");

        let request = SecurityCheckRequest {
            client_ip: "127.0.0.1".parse().expect("should succeed"),
            user_agent: None,
            endpoint: "/api/test".to_string(),
            endpoint_type: EndpointType::Api,
            input_data: Some(vec![(
                "username".to_string(),
                "valid_user".to_string(),
                InputType::Text,
            )]),
            user_id: None,
            session_id: None,
            policy_name: None,
            correlation_id: CorrelationId::new(),
            metadata: HashMap::new(),
        };

        let result = orchestrator.check_security(request).await;
        assert!(result.allowed);
        assert_eq!(result.validation_results.len(), 1);
    }

    #[tokio::test]
    async fn test_security_check_with_invalid_input() {
        let config = create_test_config();
        let orchestrator = SecurityOrchestrator::new(config)
            .await
            .expect("should succeed");

        let request = SecurityCheckRequest {
            client_ip: "127.0.0.1".parse().expect("should succeed"),
            user_agent: None,
            endpoint: "/api/test".to_string(),
            endpoint_type: EndpointType::Api,
            input_data: Some(vec![(
                "query".to_string(),
                "'; DROP TABLE users;--".to_string(),
                InputType::Text,
            )]),
            user_id: None,
            session_id: None,
            policy_name: None,
            correlation_id: CorrelationId::new(),
            metadata: HashMap::new(),
        };

        let result = orchestrator.check_security(request).await;
        assert!(!result.allowed);
        assert!(result.denial_reason.is_some());
        assert!(!result.security_events.is_empty());
        assert!(result.risk_level >= RiskLevel::Low);
    }

    #[tokio::test]
    async fn test_violation_counter_updates() {
        let mut config = create_test_config();
        config.security_monitoring.enable_real_time_monitoring = false;
        config.response_thresholds.temp_block_threshold = 3;
        config.response_thresholds.admin_alert_threshold = 2;
        let orchestrator = SecurityOrchestrator::new(config)
            .await
            .expect("should succeed");

        let client_ip: IpAddr = "192.168.1.100".parse().expect("should succeed");

        for _ in 0..3 {
            let request = SecurityCheckRequest {
                client_ip,
                user_agent: None,
                endpoint: "/api/test".to_string(),
                endpoint_type: EndpointType::Api,
                input_data: Some(vec![(
                    "field".to_string(),
                    "'; DROP TABLE--".to_string(),
                    InputType::Text,
                )]),
                user_id: None,
                session_id: None,
                policy_name: None,
                correlation_id: CorrelationId::new(),
                metadata: HashMap::new(),
            };
            orchestrator.check_security(request).await;
        }

        let stats = orchestrator.get_security_statistics().await;
        assert!(stats.total_tracked_ips >= 1);
    }

    #[tokio::test]
    async fn test_automated_response_escalation() {
        let mut config = create_test_config();
        config.security_monitoring.enable_real_time_monitoring = false;
        config.response_thresholds.temp_block_threshold = 2;
        config.response_thresholds.admin_alert_threshold = 2;
        config.enable_automated_response = true;
        let orchestrator = SecurityOrchestrator::new(config)
            .await
            .expect("should succeed");

        let client_ip: IpAddr = "192.168.1.101".parse().expect("should succeed");

        for _ in 0..3 {
            let request = SecurityCheckRequest {
                client_ip,
                user_agent: None,
                endpoint: "/api/test".to_string(),
                endpoint_type: EndpointType::Api,
                input_data: Some(vec![(
                    "field".to_string(),
                    "invalid<script>".to_string(),
                    InputType::Text,
                )]),
                user_id: None,
                session_id: None,
                policy_name: None,
                correlation_id: CorrelationId::new(),
                metadata: HashMap::new(),
            };
            orchestrator.check_security(request).await;
        }

        let stats = orchestrator.get_security_statistics().await;
        assert!(stats.total_tracked_ips >= 1);
    }

    #[tokio::test]
    async fn test_empty_input_data() {
        let config = create_test_config();
        let orchestrator = SecurityOrchestrator::new(config)
            .await
            .expect("should succeed");

        let request = SecurityCheckRequest {
            client_ip: "127.0.0.1".parse().expect("should succeed"),
            user_agent: None,
            endpoint: "/api/test".to_string(),
            endpoint_type: EndpointType::Api,
            input_data: Some(vec![]),
            user_id: None,
            session_id: None,
            policy_name: None,
            correlation_id: CorrelationId::new(),
            metadata: HashMap::new(),
        };

        let result = orchestrator.check_security(request).await;
        assert!(result.allowed);
        assert!(result.validation_results.is_empty());
    }

    #[tokio::test]
    async fn test_risk_level_ordering() {
        assert!(RiskLevel::Low < RiskLevel::Medium);
        assert!(RiskLevel::Medium < RiskLevel::High);
        assert!(RiskLevel::High < RiskLevel::Critical);
        assert_eq!(RiskLevel::Low.max(RiskLevel::Critical), RiskLevel::Critical);
    }

    #[tokio::test]
    async fn test_response_thresholds_default() {
        let thresholds = ResponseThresholds::default();
        assert_eq!(thresholds.temp_block_threshold, 10);
        assert_eq!(thresholds.permanent_block_threshold, 50);
        assert_eq!(thresholds.admin_alert_threshold, 5);
    }

    #[tokio::test]
    async fn test_security_policy_default() {
        let policy = SecurityPolicy::default();
        assert_eq!(policy.name, "default");
        assert!(!policy.require_authentication);
        assert_eq!(policy.min_authorization_level, "none");
    }

    #[tokio::test]
    async fn test_orchestrator_shutdown_component_name() {
        let config = create_test_config();
        let orchestrator = SecurityOrchestrator::new(config)
            .await
            .expect("should succeed");
        assert_eq!(orchestrator.component_name(), "security_orchestrator");
    }

    #[tokio::test]
    async fn test_orchestrator_shutdown_phases() {
        let config = create_test_config();
        let orchestrator = SecurityOrchestrator::new(config)
            .await
            .expect("should succeed");

        assert!(
            orchestrator
                .shutdown(ShutdownPhase::StopAccepting)
                .await
                .is_ok()
        );
        assert!(
            orchestrator
                .shutdown(ShutdownPhase::DrainRequests)
                .await
                .is_ok()
        );
        assert!(
            orchestrator
                .shutdown(ShutdownPhase::CloseConnections)
                .await
                .is_ok()
        );
        assert!(
            orchestrator
                .shutdown(ShutdownPhase::CleanupResources)
                .await
                .is_ok()
        );
        assert!(
            orchestrator
                .shutdown(ShutdownPhase::ShutdownTasks)
                .await
                .is_ok()
        );
        assert!(
            orchestrator
                .shutdown(ShutdownPhase::FinalCleanup)
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn test_security_check_with_metadata() {
        let config = create_test_config();
        let orchestrator = SecurityOrchestrator::new(config)
            .await
            .expect("should succeed");

        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());

        let request = SecurityCheckRequest {
            client_ip: "127.0.0.1".parse().expect("should succeed"),
            user_agent: Some("test-agent".to_string()),
            endpoint: "/api/test".to_string(),
            endpoint_type: EndpointType::Authentication,
            input_data: None,
            user_id: Some("user123".to_string()),
            session_id: Some("session456".to_string()),
            policy_name: Some("default".to_string()),
            correlation_id: CorrelationId::new(),
            metadata,
        };

        let result = orchestrator.check_security(request).await;
        assert!(result.allowed);
    }
}
