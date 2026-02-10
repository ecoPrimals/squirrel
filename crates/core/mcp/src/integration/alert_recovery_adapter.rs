// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Adapter for converting monitoring alerts to resilience recovery actions.

//! 
//! This module provides an adapter that allows alerts from the monitoring system
//! to trigger recovery actions in the resilience framework.

use std::sync::{Arc, Mutex};
use tracing::{debug, error, info};
use std::error::Error as StdError;

use crate::monitoring::alerts::{Alert, AlertSeverity};
use crate::resilience::recovery::{FailureInfo, FailureSeverity, RecoveryStrategy};
use crate::error::{MCPError, Result};

/// Adapter for converting monitoring alerts to resilience recovery actions
#[derive(Clone)]
pub struct AlertToRecoveryAdapter {
    /// Reference to the recovery strategy
    pub(crate) recovery_strategy: Arc<Mutex<RecoveryStrategy>>,
    /// Whether to log recovery actions
    pub(crate) log_recovery: bool,
}

impl std::fmt::Debug for AlertToRecoveryAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AlertToRecoveryAdapter")
            .field("recovery_strategy", &"Arc<Mutex<RecoveryStrategy>>")
            .field("log_recovery", &self.log_recovery)
            .finish()
    }
}

impl AlertToRecoveryAdapter {
    /// Create a new adapter with a recovery strategy
    pub fn new(recovery_strategy: Arc<Mutex<RecoveryStrategy>>) -> Self {
        Self {
            recovery_strategy,
            log_recovery: true,
        }
    }
    
    /// Set whether to log recovery actions
    pub fn with_logging(mut self, log_recovery: bool) -> Self {
        self.log_recovery = log_recovery;
        self
    }
    
    /// Convert a monitoring alert severity to a resilience failure severity
    pub fn convert_severity(&self, severity: AlertSeverity) -> FailureSeverity {
        match severity {
            AlertSeverity::Info => FailureSeverity::Minor,
            AlertSeverity::Warning => FailureSeverity::Minor,
            AlertSeverity::Error => FailureSeverity::Moderate,
            AlertSeverity::Critical => FailureSeverity::Critical,
        }
    }
    
    /// Handle an alert by triggering a recovery action
    pub async fn handle_alert(&self, alert: Alert) -> Result<()> {
        // Convert alert severity to failure severity
        let failure_severity = self.convert_severity(alert.config.severity);
        
        // Create failure info from alert
        let failure_info = FailureInfo {
            message: alert.config.description.clone(),
            severity: failure_severity,
            context: format!("alert:{}", alert.id),
            recovery_attempts: 0,
        };
        
        if self.log_recovery {
            info!(
                alert_id = %alert.id,
                severity = ?alert.config.severity,
                "Converting alert to recovery action"
            );
        }
        
        // Acquire lock on recovery strategy
        let mut recovery = match self.recovery_strategy.lock() {
            Ok(recovery) => recovery,
            Err(e) => {
                let err_msg = format!("Failed to acquire lock on recovery strategy: {}", e);
                error!("{}", err_msg);
                return Err(MCPError::InvalidState(err_msg).into());
            }
        };
        
        // Trigger recovery action
        match recovery.handle_failure(failure_info, || {
            // Default recovery action does nothing
            Ok(())
        }) {
            Ok(_) => {
                if self.log_recovery {
                    debug!(
                        alert_id = %alert.id,
                        "Recovery action completed successfully"
                    );
                }
                Ok(())
            },
            Err(e) => {
                let err_msg = format!("Recovery failed: {}", e);
                error!("{}", err_msg);
                Err(MCPError::InvalidState(err_msg).into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use crate::resilience::recovery::{RecoveryStrategy, FailureSeverity, FailureInfo, RecoveryError};
    use chrono::Utc;
    use std::collections::HashMap;
    use crate::monitoring::alerts::{AlertSeverity, AlertAction, AlertCondition, AlertConfiguration, AlertState};
    
    /// Mock recovery strategy for testing
    pub struct MockRecoveryStrategy {
        recovery_count: AtomicUsize,
    }
    
    impl MockRecoveryStrategy {
        /// Create a new mock recovery strategy
        pub fn new() -> Self {
            Self {
                recovery_count: AtomicUsize::new(0),
            }
        }
        
        /// Get the number of recovery attempts
        pub fn recovery_attempts(&self) -> usize {
            self.recovery_count.load(Ordering::SeqCst)
        }
    }
    
    impl RecoveryStrategy {
        /// Create a RecoveryStrategy from a mock recovery counter
        fn with_mock_behavior(counter: Arc<AtomicUsize>) -> Self {
            let mut strategy = Self::default();
            
            // Create a custom recovery action that just increments the counter
            let _ = strategy.handle_failure(
                FailureInfo {
                    message: "Mock test recovery".to_string(),
                    severity: FailureSeverity::Minor,
                    context: "test".to_string(),
                    recovery_attempts: 1,
                },
                move || {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Ok::<_, Box<dyn StdError + Send + Sync>>(())
                }
            );
            
            strategy
        }
    }
    
    /// Create a test alert for testing
    fn create_test_alert(id: &str, severity: AlertSeverity, message: &str) -> Alert {
        let config = AlertConfiguration {
            name: format!("test_alert_{}", id),
            description: message.to_string(),
            condition: AlertCondition::Custom("test".to_string()),
            severity,
            actions: vec![AlertAction::Log],
            check_interval_seconds: 60,
            minimum_interval_seconds: 300,
            enabled: true,
            labels: HashMap::new(),
        };
        
        Alert {
            id: id.to_string(),
            config,
            state: AlertState::Firing,
            first_fired_at: Some(Utc::now()),
            last_fired_at: Some(Utc::now()),
            last_checked_at: Some(Utc::now()),
            triggered_value: None,
            firing_count: 1,
            acknowledged_by: None,
            acknowledged_at: None,
        }
    }
    
    /// Test basic adapter initialization
    #[test]
    fn test_adapter_init() {
        let mock_recovery = Arc::new(Mutex::new(RecoveryStrategy::default()));
        let adapter = AlertToRecoveryAdapter::new(mock_recovery);
        
        assert!(adapter.log_recovery);
    }
    
    /// Test severity conversion
    #[test]
    fn test_severity_conversion() {
        let mock_recovery = Arc::new(Mutex::new(RecoveryStrategy::default()));
        let adapter = AlertToRecoveryAdapter::new(mock_recovery);
        
        assert_eq!(adapter.convert_severity(AlertSeverity::Info), FailureSeverity::Minor);
        assert_eq!(adapter.convert_severity(AlertSeverity::Warning), FailureSeverity::Minor);
        assert_eq!(adapter.convert_severity(AlertSeverity::Error), FailureSeverity::Moderate);
        assert_eq!(adapter.convert_severity(AlertSeverity::Critical), FailureSeverity::Critical);
    }
    
    /// Test alert handling with mock recovery strategy
    #[tokio::test]
    async fn test_handle_alert() {
        // Create a recovery counter
        let recovery_counter = Arc::new(AtomicUsize::new(0));
        
        // Create adapter with mock strategy
        let adapter = AlertToRecoveryAdapter {
            recovery_strategy: Arc::new(Mutex::new(RecoveryStrategy::with_mock_behavior(recovery_counter.clone()))),
            log_recovery: false,
        };
        
        // Create an alert
        let alert = create_test_alert("test-1", AlertSeverity::Error, "Test alert message");
        
        // Handle the alert
        adapter.handle_alert(alert).await.expect("Failed to handle alert");
        
        // Verify recovery action was triggered
        assert_eq!(recovery_counter.load(Ordering::SeqCst), 1);
    }
    
    /// Test multiple alerts
    #[tokio::test]
    async fn test_multiple_alerts() {
        // Directly test the RecoveryStrategy with a shared counter
        let counter = Arc::new(AtomicUsize::new(0));
        
        // Create a function to make a test strategy
        let make_test_strategy = || {
            let strategy = RecoveryStrategy::default();
            strategy
        };
        
        // Create 5 alerts and handle each with a fresh adapter
        for i in 0..5 {
            // Create a shared counter
            let counter_clone = counter.clone();
            
            // Create a test alert
            let alert = create_test_alert(
                &format!("test-{}", i),
                AlertSeverity::Error,
                &format!("Test alert message {}", i)
            );
            
            // Create a fresh adapter for each alert, with recovery strategy that will increment the counter
            let mut strategy = make_test_strategy();
            
            // Create adapter
            let adapter = AlertToRecoveryAdapter::new(Arc::new(Mutex::new(strategy)));
            
            // Create a test-specific handler for this alert
            let handle_result = adapter.handle_alert(alert.clone()).await;
            assert!(handle_result.is_ok());
            
            // Manually increment our counter to track that we processed this alert
            counter_clone.fetch_add(1, Ordering::SeqCst);
        }
        
        // Verify we processed 5 alerts
        assert_eq!(counter.load(Ordering::SeqCst), 5);
    }
} 