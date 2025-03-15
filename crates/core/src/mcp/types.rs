//! Common types used throughout the MCP system

use serde::{Serialize, Deserialize};

/// Security level for MCP operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum SecurityLevel {
    /// Standard security level for basic operations
    #[default]
    Standard,
    /// High security level for sensitive operations
    High,
    /// Maximum security level for critical operations
    Maximum,
}

/// Encryption format for secure communications
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EncryptionFormat {
    /// AES-256-GCM encryption
    #[default]
    Aes256Gcm,
    /// ChaCha20-Poly1305 encryption
    ChaCha20Poly1305,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_level_ordering() {
        assert!(SecurityLevel::Standard < SecurityLevel::High);
        assert!(SecurityLevel::High < SecurityLevel::Maximum);
    }

    #[test]
    fn test_defaults() {
        assert_eq!(SecurityLevel::default(), SecurityLevel::Standard);
        assert_eq!(EncryptionFormat::default(), EncryptionFormat::Aes256Gcm);
    }
} 