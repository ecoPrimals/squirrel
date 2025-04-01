// crates/mcp/src/security/types.rs
//! Common security-related types used across the MCP crate.

use serde::{Serialize, Deserialize};
use std::fmt;
use std::any::Any;
use crate::security::traits::{ResourceTrait, ActionTrait};

// Re-export UserId from the identity module
pub use crate::security::identity::UserId;

// Re-export Token and AuthCredentials from token module
pub use crate::security::token::{Token, AuthCredentials};

/// Resource that can be accessed with permissions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resource {
    /// Resource identifier
    pub id: String,
    /// Optional resource attributes
    pub attributes: Option<serde_json::Value>,
}

impl Resource {
    /// Create a new resource
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            attributes: None,
        }
    }
    
    /// Create a new resource with attributes
    pub fn with_attributes(id: &str, attributes: serde_json::Value) -> Self {
        Self {
            id: id.to_string(),
            attributes: Some(attributes),
        }
    }
}

// Implement Display for Resource
impl fmt::Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Resource({})", self.id)
    }
}

// Implement ResourceTrait for Resource
impl ResourceTrait for Resource {
    fn id(&self) -> &str {
        &self.id
    }
    
    fn attributes(&self) -> Option<&serde_json::Value> {
        self.attributes.as_ref()
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Action that can be performed on a resource
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct Action {
    /// String representation of the action
    pub action: String,
}

impl Action {
    /// Create a new action
    pub fn new(action: &str) -> Self {
        Self {
            action: action.to_string()
        }
    }
    
    /// Execute action
    pub fn execute() -> Self {
        Self {
            action: "execute".to_string()
        }
    }
    
    /// Read action
    pub fn read() -> Self {
        Self {
            action: "read".to_string()
        }
    }
    
    /// Write action
    pub fn write() -> Self {
        Self {
            action: "write".to_string()
        }
    }
    
    /// Delete action
    pub fn delete() -> Self {
        Self {
            action: "delete".to_string()
        }
    }
    
    /// Admin action with highest privileges
    pub fn admin() -> Self {
        Self {
            action: "admin".to_string()
        }
    }
    
    /// Convert to string reference
    pub fn as_ref(&self) -> &str {
        &self.action
    }
}

// Implement Display for Action
impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Action({})", self.action)
    }
}

// Implement ActionTrait for Action
impl ActionTrait for Action {
    fn as_ref(&self) -> &str {
        &self.action
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Represents a role ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoleId(pub String);

// --- Encryption Types ---

/// Enum representing supported encryption formats.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EncryptionFormat {
    AesGcm, // AES-GCM (commonly used authenticated encryption)
    Aes256Gcm, // AES-256-GCM (same as above but more explicit about key size)
    ChaCha20Poly1305, // ChaCha20-Poly1305 (another AEAD option)
    None, // Represents no encryption
}

impl Default for EncryptionFormat {
    fn default() -> Self {
        Self::None
    }
}

/// Contains information about encryption applied to data.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct EncryptionInfo {
    pub format: EncryptionFormat,
    pub key_id: Option<String>,
    // Add other fields like IV/nonce if needed by the format
    pub iv: Option<Vec<u8>>,
    pub aad: Option<Vec<u8>>,
}

/// Security level for different operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityLevel {
    Low,
    Standard,
    High,
    Critical,
}

impl Default for SecurityLevel {
    fn default() -> Self {
        SecurityLevel::Standard
    }
}

/// Holds security-related metadata for a message or operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SecurityMetadata {
    pub security_level: SecurityLevel,
    pub encryption_info: Option<EncryptionInfo>,
    pub signature: Option<String>,
    pub auth_token: Option<String>,
    pub permissions: Option<Vec<String>>,
    pub roles: Option<Vec<String>>,
}