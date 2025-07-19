//! Security Health Monitoring
//!
//! This module contains types and functionality for monitoring the health
//! of security systems and components.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Security health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityHealthStatus {
    /// Overall status message
    pub status: String,
    /// Number of active threats detected
    pub active_threats: u32,
    /// Number of policy violations
    pub policy_violations: u32,
    /// Number of authentication failures
    pub auth_failures: u32,
    /// System uptime
    pub uptime: Duration,
    /// Last health check timestamp
    pub last_check: DateTime<Utc>,
}

impl SecurityHealthStatus {
    /// Create a new healthy security status
    pub fn healthy() -> Self {
        Self {
            status: "healthy".to_string(),
            active_threats: 0,
            policy_violations: 0,
            auth_failures: 0,
            uptime: Duration::from_secs(0),
            last_check: Utc::now(),
        }
    }

    /// Create a new unhealthy security status
    pub fn unhealthy(reason: &str) -> Self {
        Self {
            status: format!("unhealthy: {}", reason),
            active_threats: 0,
            policy_violations: 0,
            auth_failures: 0,
            uptime: Duration::from_secs(0),
            last_check: Utc::now(),
        }
    }

    /// Check if the security system is healthy
    pub fn is_healthy(&self) -> bool {
        self.status == "healthy" && self.active_threats == 0
    }

    /// Update health status
    pub fn update_status(&mut self, status: String) {
        self.status = status;
        self.last_check = Utc::now();
    }

    /// Record a threat detection
    pub fn record_threat(&mut self) {
        self.active_threats += 1;
        self.last_check = Utc::now();
    }

    /// Record a policy violation
    pub fn record_policy_violation(&mut self) {
        self.policy_violations += 1;
        self.last_check = Utc::now();
    }

    /// Record an authentication failure
    pub fn record_auth_failure(&mut self) {
        self.auth_failures += 1;
        self.last_check = Utc::now();
    }

    /// Reset threat count
    pub fn reset_threats(&mut self) {
        self.active_threats = 0;
        self.last_check = Utc::now();
    }

    /// Update uptime
    pub fn update_uptime(&mut self, uptime: Duration) {
        self.uptime = uptime;
        self.last_check = Utc::now();
    }

    /// Get health score (0-100)
    pub fn get_health_score(&self) -> u8 {
        let mut score: u8 = 100;

        // Deduct points for active threats
        score = score.saturating_sub(self.active_threats as u8 * 10);

        // Deduct points for policy violations
        score = score.saturating_sub((self.policy_violations / 10) as u8);

        // Deduct points for authentication failures
        score = score.saturating_sub((self.auth_failures / 5) as u8);

        // Ensure we don't go below 0
        score
    }
}

impl Default for SecurityHealthStatus {
    fn default() -> Self {
        Self::healthy()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_healthy_status() {
        let status = SecurityHealthStatus::healthy();
        assert!(status.is_healthy());
        assert_eq!(status.status, "healthy");
        assert_eq!(status.active_threats, 0);
        assert_eq!(status.get_health_score(), 100);
    }

    #[test]
    fn test_unhealthy_status() {
        let status = SecurityHealthStatus::unhealthy("test failure");
        assert!(!status.is_healthy());
        assert_eq!(status.status, "unhealthy: test failure");
    }

    #[test]
    fn test_threat_recording() {
        let mut status = SecurityHealthStatus::healthy();

        status.record_threat();
        assert_eq!(status.active_threats, 1);
        assert!(!status.is_healthy());

        status.record_threat();
        assert_eq!(status.active_threats, 2);

        status.reset_threats();
        assert_eq!(status.active_threats, 0);
        assert!(status.is_healthy());
    }

    #[test]
    fn test_health_score() {
        let mut status = SecurityHealthStatus::healthy();
        assert_eq!(status.get_health_score(), 100);

        status.record_threat();
        assert_eq!(status.get_health_score(), 90);

        status.record_threat();
        assert_eq!(status.get_health_score(), 80);

        // Add policy violations
        for _ in 0..10 {
            status.record_policy_violation();
        }
        assert_eq!(status.get_health_score(), 79);

        // Add auth failures
        for _ in 0..5 {
            status.record_auth_failure();
        }
        assert_eq!(status.get_health_score(), 78);
    }

    #[test]
    fn test_status_updates() {
        let mut status = SecurityHealthStatus::healthy();
        let initial_time = status.last_check;

        // Small delay to ensure timestamp changes
        std::thread::sleep(std::time::Duration::from_millis(1));

        status.update_status("initializing".to_string());
        assert_eq!(status.status, "initializing");
        assert!(status.last_check > initial_time);
    }
}
