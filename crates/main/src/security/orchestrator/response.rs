// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Automated security response subsystem — violation tracking, policy
//! determination, and response execution.

use super::types::{ResponseType, RiskLevel, SecurityResponse, ViolationCounter};
use std::collections::HashMap;
use std::net::IpAddr;
use tracing::{error, info, warn};

use super::SecurityOrchestrator;

impl SecurityOrchestrator {
    /// Update violation counter for IP
    pub(super) async fn update_violation_counter(&self, client_ip: IpAddr, risk_level: RiskLevel) {
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
    pub(super) async fn determine_automated_response(
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
    pub(super) async fn execute_automated_response(&self, response: SecurityResponse) {
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
            }
            ResponseType::Escalate => {
                error!(
                    target = %response.target,
                    operation = "automated_security_response",
                    "ESCALATION - {}", response.details
                );
            }
        }
    }
}
