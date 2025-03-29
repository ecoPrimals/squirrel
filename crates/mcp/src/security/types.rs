// Security types module
//
// This module defines the core types used in the security subsystem,
// particularly for Role-Based Access Control (RBAC).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid;

/// Role in the RBAC system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    /// Unique identifier for the role
    pub id: String,
    /// Name of the role
    pub name: String,
    /// Optional description of the role
    pub description: Option<String>,
    /// Set of permissions granted by this role
    pub permissions: HashSet<Permission>,
    /// Set of parent role IDs
    pub parent_roles: HashSet<String>,
    /// Security level required for this role
    pub security_level: SecurityLevel,
    /// Whether this role can be delegated
    pub can_delegate: bool,
    /// Roles that this role can manage
    pub managed_roles: HashSet<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// Permission for a specific resource and action
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Permission {
    /// Unique identifier for the permission
    pub id: String,
    /// Name of the permission
    pub name: String,
    /// Resource the permission applies to
    pub resource: String,
    /// Action allowed by this permission
    pub action: Action,
    /// Optional resource identifier for fine-grained control
    pub resource_id: Option<String>,
    /// Optional scope limitation
    pub scope: PermissionScope,
    /// Conditions under which this permission applies
    pub conditions: Vec<PermissionCondition>,
}

impl Permission {
    /// Create a new permission with required fields
    #[must_use] pub fn new(resource: &str, action: Action) -> Self {
        Self {
            id: format!("{}-{}", resource, action.to_string().to_lowercase()),
            name: format!("{action} {resource}"),
            resource: resource.to_string(),
            action,
            resource_id: None,
            scope: PermissionScope::All,
            conditions: Vec::new(),
        }
    }

    /// Create a new permission with all fields
    #[must_use] pub const fn with_details(
        id: String, 
        name: String,
        resource: String, 
        action: Action, 
        resource_id: Option<String>,
        scope: PermissionScope,
        conditions: Vec<PermissionCondition>,
    ) -> Self {
        Self {
            id,
            name,
            resource,
            action,
            resource_id,
            scope,
            conditions,
        }
    }
}

/// Action types for permissions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// Create new resources
    Create,
    /// Read existing resources
    Read,
    /// Write to resources
    Write,
    /// Update existing resources
    Update,
    /// Delete existing resources
    Delete,
    /// Execute operations on resources
    Execute,
    /// Full administrative access
    Admin,
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Create => write!(f, "create"),
            Self::Read => write!(f, "read"),
            Self::Write => write!(f, "write"),
            Self::Update => write!(f, "update"),
            Self::Delete => write!(f, "delete"),
            Self::Execute => write!(f, "execute"),
            Self::Admin => write!(f, "admin"),
        }
    }
}

/// Scope of a permission
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(Default)]
pub enum PermissionScope {
    /// Permission applies only to owned resources
    #[default]
    Own,
    /// Permission applies to resources in the same group
    Group,
    /// Permission applies to all resources
    All,
    /// Permission applies to resources matching a pattern
    Pattern(String),
}


/// Condition for permission application
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PermissionCondition {
    /// Time-based condition (e.g., business hours)
    TimeRange {
        /// Start time in format HH:MM (24-hour)
        start_time: String,
        /// End time in format HH:MM (24-hour)
        end_time: String,
        /// Days of week when condition applies (e.g., "Monday", "Tuesday")
        days: Vec<String>,
    },
    /// Network-based condition (e.g., specific IP range)
    NetworkRange { 
        /// CIDR notation for network range (e.g., "192.168.1.0/24")
        cidr: String 
    },
    /// Security level requirement
    MinimumSecurityLevel(SecurityLevel),
    /// Custom attribute-based condition
    AttributeEquals { 
        /// Name of the attribute to check
        attribute: String, 
        /// Expected value of the attribute
        value: String 
    },
}

/// Context for evaluating permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionContext {
    /// Current user ID
    pub user_id: String,
    /// Current time
    pub current_time: Option<DateTime<Utc>>,
    /// Current network address
    pub network_address: Option<String>,
    /// Current security level
    pub security_level: SecurityLevel,
    /// Custom attributes
    pub attributes: HashMap<String, String>,
    /// Resource owner ID
    pub resource_owner_id: Option<String>,
    /// Resource group ID
    pub resource_group_id: Option<String>,
}

impl PermissionContext {
    /// Create a new permission context with the given user ID
    #[must_use] pub fn new(user_id: &str) -> Self {
        Self {
            user_id: user_id.to_string(),
            current_time: Some(Utc::now()),
            network_address: None,
            security_level: SecurityLevel::Standard,
            attributes: HashMap::new(),
            resource_owner_id: None,
            resource_group_id: None,
        }
    }
}

impl Default for PermissionContext {
    fn default() -> Self {
        Self {
            user_id: String::from("system"),
            current_time: Some(Utc::now()),
            network_address: None,
            security_level: SecurityLevel::Standard,
            attributes: HashMap::new(),
            resource_owner_id: None,
            resource_group_id: None,
        }
    }
}

/// Condition that restricts access based on time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeWindow {
    /// Start time in format HH:MM (24-hour)
    pub start_time: String,
    /// End time in format HH:MM (24-hour)
    pub end_time: String,
    /// Days of week when condition applies (e.g., "Monday", "Tuesday")
    pub days: Vec<String>,
}

/// Network range condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkCondition {
    /// Restrict access to specific CIDR network range
    NetworkRange { 
        /// CIDR notation for network range (e.g., "192.168.1.0/24")
        cidr: String 
    },
}

/// Attribute-based condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttributeCondition {
    /// Condition that checks if an attribute equals a specific value
    AttributeEquals { 
        /// Name of the attribute to check
        attribute: String, 
        /// Expected value of the attribute
        value: String 
    },
}

// Security-related types for MCP

/// Security level for MCP operations.
///
/// This enumeration defines the various security levels supported by the MCP system,
/// from low to critical. These levels are used to specify the required security
/// for different operations and resources, enabling fine-grained security control.
///
/// # Ordering
///
/// Security levels form a total ordering where:
/// Low < Standard < High < Critical
///
/// This allows for easy comparison of security requirements.
///
/// # Usage
///
/// ```
/// use crate::security::types::SecurityLevel;
///
/// // Function that requires a minimum security level
/// fn secure_operation(level: SecurityLevel) -> bool {
///     if level >= SecurityLevel::High {
///         // Perform high-security operation
///         true
///     } else {
///         // Reject with insufficient security
///         false
///     }
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash, Default)]
pub enum SecurityLevel {
    /// Low security level: Minimal security requirements for non-sensitive operations
    Low = 0,
    /// Standard security level: Default for most operations
    #[default]
    Standard = 5,
    /// High security level: For sensitive operations requiring stronger security
    High = 10,
    /// Critical security level: Maximum security for the most sensitive operations
    Critical = 15,
}

/// Encryption format for secure communications.
///
/// This enumeration defines the supported encryption algorithms for
/// securing communications within the MCP system. The appropriate
/// format should be selected based on security requirements and
/// performance considerations.
///
/// # Security Considerations
///
/// - `None`: Provides no encryption and should only be used for non-sensitive data
/// - `Aes256Gcm`: Provides strong security with reasonable performance
/// - `ChaCha20Poly1305`: Alternative that may be faster on systems without AES hardware acceleration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, Hash)]
pub enum EncryptionFormat {
    /// No encryption: Data is transmitted in plaintext
    #[default]
    None,
    /// AES-256-GCM encryption: Industry standard authenticated encryption
    Aes256Gcm,
    /// ChaCha20-Poly1305 encryption: Modern stream cipher with authentication
    ChaCha20Poly1305,
}

/// Encryption information for secure communications
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EncryptionInfo {
    /// Encryption format used
    pub format: EncryptionFormat,
    /// Optional encryption key ID
    pub key_id: Option<String>,
    /// Optional initialization vector
    pub iv: Option<Vec<u8>>,
    /// Optional additional authenticated data
    pub aad: Option<Vec<u8>>,
}

/// Security metadata for messages
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SecurityMetadata {
    /// Security level for the message
    pub security_level: SecurityLevel,
    /// Optional encryption information
    pub encryption_info: Option<EncryptionInfo>,
    /// Optional digital signature
    pub signature: Option<String>,
    /// Optional authentication token
    pub auth_token: Option<String>,
    /// Optional permissions
    pub permissions: Option<Vec<String>>,
    /// Optional roles
    pub roles: Option<Vec<String>>,
}

// --- Identity Types ---

/// Wrapper type for User IDs.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub String);

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

impl UserId {
    /// Create a new user ID
    #[must_use]
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for UserId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for UserId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Wrapper type for Role IDs.
pub type RoleId = String;

/// User role within the MCP system.
///
/// ... (doc comment) ...
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UserRole {
    /// Administrator role: Full system access
    Admin,
    /// User role: Standard access for normal operations
    User,
    /// Guest role: Limited access, typically read-only
    Guest,
    /// Custom role: Role with specific permissions
    Custom(String), // Using String for custom roles
}

impl Default for UserRole {
    fn default() -> Self {
        Self::Guest // Changed default to Guest as per the original types.rs code that was removed
    }
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Admin => write!(f, "Admin"),
            Self::User => write!(f, "User"),
            Self::Guest => write!(f, "Guest"),
            Self::Custom(role) => write!(f, "Custom({role})"),
        }
    }
}

// --- End Identity Types/UserRole ---

// --- Type Aliases ---

/// Type alias for a Permission identifier (typically a String).
pub type PermissionId = String;

// --- End Type Aliases ---

// --- Token Types ---

/// Session token type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionToken(pub String);

impl Default for SessionToken {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionToken {
    /// Create a new session token
    #[must_use]
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl std::fmt::Display for SessionToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for SessionToken {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Authentication token type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AuthToken(pub String);

impl Default for AuthToken {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthToken {
    /// Create a new authentication token
    #[must_use]
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl std::fmt::Display for AuthToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for AuthToken {
    fn from(s: String) -> Self {
        Self(s)
    }
}

// --- End Token Types ---

// Other security-related types (like Role, Permission, etc.) will go here. 