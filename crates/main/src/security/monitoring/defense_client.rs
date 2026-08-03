// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! IPC client for delegating threat detection and anomaly classification to the
//! defense capability provider (skunkBat).
//!
//! Squirrel observes and logs security events locally; detection and classification
//! are delegated via capability-based discovery — no hardcoded primal names.

use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::{debug, warn};

use super::alerts::{AlertType, SecurityAlert};
use super::types::{BehavioralPattern, EventSeverity, SecurityEvent};
use crate::capabilities::discovery::{CapabilityProvider, discover_capability};
use crate::capabilities::lifecycle::send_jsonrpc_public;
use crate::error::PrimalError;

/// Capabilities tried in order when locating a defense provider.
const DEFENSE_CAPABILITIES: &[&str] = &["defense.anomaly", "security.anomaly", "defense.threat"];

/// Delegates anomaly detection and threat classification to skunkBat via IPC.
#[derive(Debug, Clone, Default)]
pub struct DefenseClient {
    cached_provider: Arc<RwLock<Option<CapabilityProvider>>>,
    /// When true, skip live discovery and return error immediately.
    skip_discovery: bool,
}

impl DefenseClient {
    /// Create a new defense IPC client.
    #[must_use]
    pub fn new() -> Self {
        Self {
            cached_provider: Arc::new(RwLock::new(None)),
            skip_discovery: false,
        }
    }

    /// Create a client that never attempts live discovery (for tests).
    #[cfg(test)]
    #[must_use]
    pub(crate) fn without_discovery() -> Self {
        Self {
            cached_provider: Arc::new(RwLock::new(None)),
            skip_discovery: true,
        }
    }

    /// Send an arbitrary defense RPC request (for higher-level delegation wrappers).
    pub async fn call_method(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, PrimalError> {
        self.call_defense_rpc(method, params).await
    }

    /// Classify a security event as a potential threat via the defense provider.
    pub async fn classify_threat(
        &self,
        event: &SecurityEvent,
    ) -> Result<Option<SecurityAlert>, PrimalError> {
        let event_value = serde_json::to_value(event).map_err(|e| {
            PrimalError::SerializationError(format!("Failed to serialize security event: {e}"))
        })?;

        let response = self
            .call_defense_rpc(
                "defense.classify_threat",
                serde_json::json!({ "event": event_value }),
            )
            .await?;

        Ok(parse_defense_alert(&response))
    }

    /// Detect behavioral anomalies via the defense provider.
    pub async fn detect_anomaly(
        &self,
        pattern: &BehavioralPattern,
    ) -> Result<Option<SecurityAlert>, PrimalError> {
        let pattern_value = serde_json::json!({
            "client_ip": pattern.client_ip,
            "user_id": pattern.user_id,
            "total_requests": pattern.total_requests,
            "failed_requests": pattern.failed_requests,
            "violation_count": pattern.violation_count,
            "failure_rate": pattern.failure_rate(),
        });

        let response = self
            .call_defense_rpc(
                "defense.detect_anomaly",
                serde_json::json!({ "pattern": pattern_value }),
            )
            .await?;

        Ok(parse_defense_alert(&response))
    }

    async fn call_defense_rpc(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, PrimalError> {
        let provider = self.resolve_provider().await?;
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1,
        });

        let response = send_jsonrpc_public(&provider.socket, &request)
            .await
            .map_err(|e| {
                PrimalError::OperationFailed(format!(
                    "Defense IPC '{method}' at {}: {e}",
                    provider.socket.display()
                ))
            })?;

        if let Some(error) = response.get("error") {
            let message = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("unknown defense provider error");
            return Err(PrimalError::OperationFailed(format!(
                "Defense provider error on {method}: {message}"
            )));
        }

        Ok(response
            .get("result")
            .cloned()
            .unwrap_or(serde_json::Value::Null))
    }

    async fn resolve_provider(&self) -> Result<CapabilityProvider, PrimalError> {
        let cached = self.cached_provider.read().await.clone();
        if let Some(provider) = cached {
            return Ok(provider);
        }

        if self.skip_discovery {
            return Err(PrimalError::ResourceNotFound(
                "Defense capability provider not available (discovery disabled)".into(),
            ));
        }

        for capability in DEFENSE_CAPABILITIES {
            match discover_capability(capability).await {
                Ok(provider) => {
                    debug!(
                        capability = capability,
                        socket = %provider.socket.display(),
                        "Resolved defense capability provider"
                    );
                    *self.cached_provider.write().await = Some(provider.clone());
                    return Ok(provider);
                }
                Err(e) => {
                    debug!(
                        capability = capability,
                        error = %e,
                        "Defense capability not available"
                    );
                }
            }
        }

        Err(PrimalError::ResourceNotFound(
            "Defense capability provider not available — anomaly detection and threat classification require skunkBat (defense.anomaly) via IPC".into(),
        ))
    }

    /// Clear cached provider (e.g. after provider restart).
    pub async fn invalidate_cache(&self) {
        *self.cached_provider.write().await = None;
    }
}

/// Parse a defense provider IPC response into a [`SecurityAlert`], if present.
fn parse_defense_alert(response: &serde_json::Value) -> Option<SecurityAlert> {
    let alert_value = response.get("alert").or_else(|| {
        if response.get("alert_type").is_some() || response.get("title").is_some() {
            Some(response)
        } else {
            None
        }
    })?;

    if alert_value.is_null() {
        return None;
    }

    let alert_type = alert_value
        .get("alert_type")
        .and_then(|v| v.as_str())
        .map_or(AlertType::SuspiciousBehavior, parse_alert_type);

    let severity = alert_value
        .get("severity")
        .and_then(|v| v.as_str())
        .map_or(EventSeverity::Warning, parse_severity);

    let title = alert_value
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Defense provider alert")
        .to_string();

    let description = alert_value
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("Threat classified by defense capability provider")
        .to_string();

    let mut alert = SecurityAlert::new(alert_type, severity, title, description);

    if let Some(actions) = alert_value
        .get("recommended_actions")
        .and_then(|v| v.as_array())
    {
        for action in actions {
            if let Some(s) = action.as_str() {
                alert = alert.with_action(s);
            }
        }
    }

    if let Some(entity) = alert_value.get("affected_entity").and_then(|v| v.as_str()) {
        alert = alert.with_affected_entity(entity);
    }

    Some(alert)
}

fn parse_alert_type(value: &str) -> AlertType {
    match value {
        "BruteForceAttempt" | "brute_force" => AlertType::BruteForceAttempt,
        "RateLimitAbuse" | "rate_limit_abuse" => AlertType::RateLimitAbuse,
        "InputValidationAbuse" | "input_validation_abuse" => AlertType::InputValidationAbuse,
        "SuspiciousBehavior" | "suspicious_behavior" => AlertType::SuspiciousBehavior,
        "PolicyViolation" | "policy_violation" => AlertType::PolicyViolation,
        "AnomalousAccess" | "anomalous_access" => AlertType::AnomalousAccess,
        "ExcessiveConcurrentSessions" | "excessive_concurrent_sessions" => {
            AlertType::ExcessiveConcurrentSessions
        }
        "HighFailureRate" | "high_failure_rate" => AlertType::HighFailureRate,
        other => {
            warn!(
                alert_type = other,
                "Unknown defense alert type — mapping to SuspiciousBehavior"
            );
            AlertType::SuspiciousBehavior
        }
    }
}

fn parse_severity(value: &str) -> EventSeverity {
    match value {
        "Info" | "info" => EventSeverity::Info,
        "High" | "high" => EventSeverity::High,
        "Critical" | "critical" => EventSeverity::Critical,
        _ => EventSeverity::Warning,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observability::CorrelationId;

    #[test]
    fn parse_defense_alert_from_nested_object() {
        let response = serde_json::json!({
            "alert": {
                "alert_type": "BruteForceAttempt",
                "severity": "High",
                "title": "Brute force detected",
                "description": "Multiple failed logins",
                "recommended_actions": ["Block IP"],
                "affected_entity": "10.0.0.1"
            }
        });

        let alert = parse_defense_alert(&response).expect("should parse");
        assert_eq!(alert.alert_type, AlertType::BruteForceAttempt);
        assert_eq!(alert.severity, EventSeverity::High);
        assert_eq!(alert.affected_entities, vec!["10.0.0.1".to_string()]);
    }

    #[test]
    fn parse_defense_alert_null_returns_none() {
        assert!(parse_defense_alert(&serde_json::json!({ "alert": null })).is_none());
    }

    #[tokio::test]
    async fn classify_threat_without_provider_returns_error() {
        let client = DefenseClient::without_discovery();
        let event = SecurityEvent::new(
            super::super::types::SecurityEventType::Authentication {
                success: false,
                user_id: None,
                method: "password".to_string(),
            },
            "10.0.0.1".to_string(),
            EventSeverity::Warning,
            "auth".to_string(),
            CorrelationId::new(),
        );

        let err = client.classify_threat(&event).await.unwrap_err();
        assert!(matches!(err, PrimalError::ResourceNotFound(_)));
    }
}
