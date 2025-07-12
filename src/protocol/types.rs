use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Authentication credentials for MCP operations
#[derive(Debug, Clone)]
pub struct AuthCredentials {
    pub username: String,
    pub password: String,
    pub token: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Security metadata for MCP operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityMetadata {
    /// Protocol version
    pub version: String,
    /// Security token
    pub token: Option<String>,
    /// Encryption enabled
    pub encrypted: bool,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

impl Default for SecurityMetadata {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            token: None,
            encrypted: false,
            timestamp: Utc::now(),
        }
    }
}

/// Message metadata for MCP operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageMetadata {
    /// Message ID
    pub message_id: String,
    /// Message type
    pub message_type: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Sender ID
    pub sender_id: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl Default for MessageMetadata {
    fn default() -> Self {
        Self {
            message_id: Uuid::new_v4().to_string(),
            message_type: "generic".to_string(),
            timestamp: Utc::now(),
            sender_id: None,
            metadata: HashMap::new(),
        }
    }
}

impl Default for AuthCredentials {
    fn default() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            token: None,
            metadata: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_auth_credentials_default() {
        let creds = AuthCredentials::default();
        assert!(creds.username.is_empty());
        assert!(creds.password.is_empty());
        assert!(creds.token.is_none());
        assert!(creds.metadata.is_empty());
    }
} 