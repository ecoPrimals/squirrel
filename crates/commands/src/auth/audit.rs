//! Audit logging for authentication and authorization events
//!
//! This module provides functionality for logging and tracking security-related events
//! in the authentication and authorization system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;
use tokio::sync::RwLock as TokioRwLock;

use crate::auth::{AuthCredentials, PermissionLevel, User};
use crate::registry::Command;

/// Type of audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    /// Authentication attempt
    AuthenticationAttempt {
        /// Username used in the attempt
        username: String,
    },
    /// Authentication success
    AuthenticationSuccess {
        /// User that was authenticated
        user: User,
    },
    /// Authentication failure
    AuthenticationFailure {
        /// Credentials used in the attempt
        credentials: AuthCredentials,
        /// Error message
        error: String,
    },
    /// Authorization attempt
    AuthorizationAttempt {
        /// User attempting to execute the command
        user: User,
        /// Command being executed
        command: String,
    },
    /// Authorization success
    AuthorizationSuccess {
        /// User authorized to execute the command
        user: User,
        /// Command being executed
        command: String,
    },
    /// Authorization failure
    AuthorizationFailure {
        /// User denied execution of the command
        user: User,
        /// Command being executed
        command: String,
        /// Error message
        error: String,
    },
    /// User creation
    UserCreation {
        /// User that was created
        user: User,
    },
    /// User modification
    UserModification {
        /// User that was modified
        user: User,
        /// Description of changes
        changes: String,
    },
    /// User deletion
    UserDeletion {
        /// Username of the deleted user
        username: String,
    },
    /// Permission level change
    PermissionChange {
        /// User whose permission level was changed
        user: User,
        /// Old permission level
        old_level: PermissionLevel,
        /// New permission level
        new_level: PermissionLevel,
    },
}

impl fmt::Display for AuditEventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AuthenticationAttempt { username: _ } => write!(f, "Authentication Attempt"),
            Self::AuthenticationSuccess { user: _ } => write!(f, "Authentication Success"),
            Self::AuthenticationFailure { credentials: _, error: _ } => write!(f, "Authentication Failure"),
            Self::AuthorizationAttempt { user: _, command: _ } => write!(f, "Authorization Attempt"),
            Self::AuthorizationSuccess { user: _, command: _ } => write!(f, "Authorization Success"),
            Self::AuthorizationFailure { user: _, command: _, error: _ } => write!(f, "Authorization Failure"),
            Self::UserCreation { user: _ } => write!(f, "User Creation"),
            Self::UserModification { user: _, changes: _ } => write!(f, "User Modification"),
            Self::UserDeletion { username: _ } => write!(f, "User Deletion"),
            Self::PermissionChange { user: _, old_level: _, new_level: _ } => write!(f, "Permission Change"),
        }
    }
}

/// Audit event containing details about a security-related action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Type of event
    pub event_type: AuditEventType,
    /// Timestamp when the event occurred
    pub timestamp: DateTime<Utc>,
}

/// Audit logger for tracking security-related events
#[derive(Debug, Clone)]
pub struct AuditLogger {
    /// Collection of audit events
    events: Arc<TokioRwLock<Vec<AuditEvent>>>,
}

impl AuditLogger {
    /// Creates a new audit logger
    pub fn new() -> Self {
        Self {
            events: Arc::new(TokioRwLock::new(Vec::new())),
        }
    }

    /// Logs an authentication attempt
    pub async fn log_authentication_attempt(&self, credentials: &AuthCredentials) {
        let username = match credentials {
            AuthCredentials::None => "anonymous".to_string(),
            AuthCredentials::Basic { username, .. } => username.clone(),
            AuthCredentials::Token(_) => "token".to_string(),
            AuthCredentials::ApiKey(_) => "api_key".to_string(),
        };

        let event = AuditEvent {
            event_type: AuditEventType::AuthenticationAttempt {
                username,
            },
            timestamp: Utc::now(),
        };
        let mut events = self.events.write().await;
        events.push(event);
    }

    /// Logs a successful authentication
    pub async fn log_authentication_success(&self, user: &User) {
        let event = AuditEvent {
            event_type: AuditEventType::AuthenticationSuccess {
                user: user.clone(),
            },
            timestamp: Utc::now(),
        };
        let mut events = self.events.write().await;
        events.push(event);
    }

    /// Logs a failed authentication
    pub async fn log_authentication_failure(&self, credentials: &AuthCredentials, error: &str) {
        let event = AuditEvent {
            event_type: AuditEventType::AuthenticationFailure {
                credentials: credentials.clone(),
                error: error.to_string(),
            },
            timestamp: Utc::now(),
        };
        let mut events = self.events.write().await;
        events.push(event);
    }

    /// Logs an authorization attempt
    pub async fn log_authorization_attempt(&self, user: &User, command: &dyn Command) {
        let event = AuditEvent {
            event_type: AuditEventType::AuthorizationAttempt {
                user: user.clone(),
                command: command.name().to_string(),
            },
            timestamp: Utc::now(),
        };
        let mut events = self.events.write().await;
        events.push(event);
    }

    /// Logs a successful authorization
    pub async fn log_authorization_success(&self, user: &User, command: &dyn Command) {
        let event = AuditEvent {
            event_type: AuditEventType::AuthorizationSuccess {
                user: user.clone(),
                command: command.name().to_string(),
            },
            timestamp: Utc::now(),
        };
        let mut events = self.events.write().await;
        events.push(event);
    }

    /// Logs a failed authorization
    pub async fn log_authorization_failure(&self, user: &User, command: &dyn Command, error: &str) {
        let event = AuditEvent {
            event_type: AuditEventType::AuthorizationFailure {
                user: user.clone(),
                command: command.name().to_string(),
                error: error.to_string(),
            },
            timestamp: Utc::now(),
        };
        let mut events = self.events.write().await;
        events.push(event);
    }

    /// Logs a user creation
    pub async fn log_user_creation(&self, user: &User) {
        let event = AuditEvent {
            event_type: AuditEventType::UserCreation {
                user: user.clone(),
            },
            timestamp: Utc::now(),
        };
        let mut events = self.events.write().await;
        events.push(event);
    }

    /// Logs a user modification
    pub async fn log_user_modification(&self, user: &User, changes: &str) {
        let event = AuditEvent {
            event_type: AuditEventType::UserModification {
                user: user.clone(),
                changes: changes.to_string(),
            },
            timestamp: Utc::now(),
        };
        let mut events = self.events.write().await;
        events.push(event);
    }

    /// Logs a user deletion
    pub async fn log_user_deletion(&self, username: &str) {
        let event = AuditEvent {
            event_type: AuditEventType::UserDeletion {
                username: username.to_string(),
            },
            timestamp: Utc::now(),
        };
        let mut events = self.events.write().await;
        events.push(event);
    }

    /// Logs a permission level change
    pub async fn log_permission_change(&self, user: &User, old_level: PermissionLevel, new_level: PermissionLevel) {
        let event = AuditEvent {
            event_type: AuditEventType::PermissionChange {
                user: user.clone(),
                old_level,
                new_level,
            },
            timestamp: Utc::now(),
        };
        let mut events = self.events.write().await;
        events.push(event);
    }

    /// Gets all audit events
    pub async fn get_events(&self) -> Vec<AuditEvent> {
        let events = self.events.read().await;
        events.clone()
    }

    /// Gets audit events of a specific type
    pub async fn get_events_by_type(&self, event_type: AuditEventType) -> Vec<AuditEvent> {
        let events = self.events.read().await;
        events
            .iter()
            .filter(|event| {
                match (&event.event_type, &event_type) {
                    (AuditEventType::AuthenticationAttempt { .. }, AuditEventType::AuthenticationAttempt { .. }) => true,
                    (AuditEventType::AuthenticationSuccess { .. }, AuditEventType::AuthenticationSuccess { .. }) => true,
                    (AuditEventType::AuthenticationFailure { .. }, AuditEventType::AuthenticationFailure { .. }) => true,
                    (AuditEventType::AuthorizationAttempt { .. }, AuditEventType::AuthorizationAttempt { .. }) => true,
                    (AuditEventType::AuthorizationSuccess { .. }, AuditEventType::AuthorizationSuccess { .. }) => true,
                    (AuditEventType::AuthorizationFailure { .. }, AuditEventType::AuthorizationFailure { .. }) => true,
                    (AuditEventType::UserCreation { .. }, AuditEventType::UserCreation { .. }) => true,
                    (AuditEventType::UserModification { .. }, AuditEventType::UserModification { .. }) => true,
                    (AuditEventType::UserDeletion { .. }, AuditEventType::UserDeletion { .. }) => true,
                    (AuditEventType::PermissionChange { .. }, AuditEventType::PermissionChange { .. }) => true,
                    _ => false,
                }
            })
            .cloned()
            .collect()
    }

    /// Gets audit events for a specific user
    pub async fn get_events_by_user(&self, username: &str) -> Vec<AuditEvent> {
        let events = self.events.read().await;
        events
            .iter()
            .filter(|event| match &event.event_type {
                AuditEventType::AuthenticationAttempt { username: u } => u == username,
                AuditEventType::AuthenticationSuccess { user } => user.name == username,
                AuditEventType::AuthenticationFailure { credentials, .. } => match credentials {
                    AuthCredentials::Basic { username: u, .. } => u == username,
                    _ => false,
                },
                AuditEventType::AuthorizationAttempt { user, .. } => user.name == username,
                AuditEventType::AuthorizationSuccess { user, .. } => user.name == username,
                AuditEventType::AuthorizationFailure { user, .. } => user.name == username,
                AuditEventType::UserCreation { user } => user.name == username,
                AuditEventType::UserModification { user, .. } => user.name == username,
                AuditEventType::UserDeletion { username: u } => u == username,
                AuditEventType::PermissionChange { user, .. } => user.name == username,
            })
            .cloned()
            .collect()
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    

    #[tokio::test]
    async fn test_audit_logging() {
        let logger = AuditLogger::new();
        let username = "testuser";

        // Log an authentication attempt
        logger.log_authentication_attempt(&AuthCredentials::Basic {
            username: username.to_string(),
            password: "password123".to_string(),
        }).await;

        // Verify the event was logged
        let events = logger.get_events().await;
        assert_eq!(events.len(), 1);
        
        let event = &events[0];
        match &event.event_type {
            AuditEventType::AuthenticationAttempt { username: logged_username } => {
                assert_eq!(logged_username, username);
            }
            _ => panic!("Expected AuthenticationAttempt event"),
        }
    }
} 