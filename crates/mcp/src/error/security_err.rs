use thiserror::Error;
use crate::types::SecurityLevel; // Import SecurityLevel

/// Security-related errors
#[derive(Debug, Clone, Error)]
pub enum SecurityError {
    /// Authentication error that occurs when credentials cannot be verified
    /// or the authentication process fails for any reason
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    /// Authorization error that occurs when a user lacks permissions
    /// to perform the requested operation
    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),
    
    /// Error that occurs when provided credentials are invalid,
    /// malformed, or do not match expected format
    #[error("Invalid credentials: {0}")]
    InvalidCredentials(String),
    
    /// Error that occurs when an authentication token has expired
    /// and is no longer valid for use
    #[error("Token expired")]
    TokenExpired,
    
    /// Error that occurs when a token is invalid, corrupted,
    /// or cannot be verified
    #[error("Invalid token: {0}")]
    InvalidToken(String),
    
    /// Error that occurs when a user role does not exist or
    /// is not valid in the current context
    #[error("Invalid role: {0}")]
    InvalidRole(String),
    
    /// Error that occurs during encryption operations, such as
    /// key generation, data encryption, or signature creation
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    
    /// Error that occurs during decryption operations, such as
    /// key retrieval, data decryption, or signature verification
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    
    /// General security error that occurs within the security
    /// subsystem but doesn't fit other specific categories
    #[error("Internal security error: {0}")]
    InternalError(String),
    
    /// Error that occurs when a message is sent with an insufficient
    /// security level for the operation being performed
    #[error("Invalid security level: required {required:?}, provided {provided:?}")]
    InvalidSecurityLevel {
        /// The security level required by the operation or receiver
        required: SecurityLevel,
        /// The security level provided in the message or request
        provided: SecurityLevel,
    },
    
    /// Error that occurs within the underlying system security
    /// infrastructure or OS security mechanisms
    #[error("System error: {0}")]
    System(String),
    
    /// Error that occurs when a permission string has invalid format
    /// or cannot be parsed correctly
    #[error("Invalid permission format: {0}")]
    InvalidPermissionFormat(String),
    
    /// Error that occurs when an action specified in a permission
    /// is not recognized or not supported
    #[error("Invalid action in permission: {0}")]
    InvalidActionInPermission(String),
    
    /// Error that occurs during the creation of a new role
    /// in the security system
    #[error("Error creating role: {0}")]
    ErrorCreatingRole(String),
    
    /// Error related to the Role-Based Access Control system,
    /// such as role assignment or permission checking
    #[error("RBAC error: {0}")]
    RBACError(String),
    
    /// Error that occurs during validation of security-related
    /// data or operations
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    /// Error that occurs when attempting to create a security
    /// entity with an ID that already exists
    #[error("Duplicate ID error: {0}")]
    DuplicateIDError(String),
    
    /// Error that occurs when a security-related entity
    /// could not be found
    #[error("Not found: {0}")]
    NotFound(String),
    
    /// Error that occurs when an operation would violate
    /// a defined security policy
    #[error("Policy violation: {0}")]
    PolicyViolation(String),
} 