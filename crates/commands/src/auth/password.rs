//! Password management module for secure password hashing and verification
//!
//! This module provides functionality for securely hashing and verifying passwords
//! using the Argon2 password hashing algorithm.

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use thiserror::Error;

/// Error type for password operations
#[derive(Debug, Error)]
pub enum PasswordError {
    /// Error during password hashing
    #[error("Failed to hash password: {0}")]
    HashingError(String),

    /// Error during password verification
    #[error("Failed to verify password: {0}")]
    VerificationError(String),

    /// Invalid password format
    #[error("Invalid password format: {0}")]
    InvalidFormat(String),
}

/// Result type for password operations
pub type PasswordResult<T> = Result<T, PasswordError>;

/// Password manager for secure password operations
#[derive(Debug, Clone)]
pub struct PasswordManager {
    /// Argon2 instance with default parameters
    argon2: Argon2<'static>,
}

impl PasswordManager {
    /// Creates a new password manager with default settings
    pub fn new() -> Self {
        Self {
            argon2: Argon2::default(),
        }
    }

    /// Hashes a password using Argon2
    ///
    /// # Arguments
    /// * `password` - The password to hash
    ///
    /// # Returns
    /// The hashed password as a string
    pub fn hash_password(&self, password: impl AsRef<[u8]>) -> PasswordResult<String> {
        let salt = SaltString::generate(&mut OsRng);
        self.argon2
            .hash_password(password.as_ref(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| PasswordError::HashingError(e.to_string()))
    }

    /// Verifies a password against a hash
    ///
    /// # Arguments
    /// * `password` - The password to verify
    /// * `hash` - The hash to verify against
    ///
    /// # Returns
    /// true if the password matches, false otherwise
    pub fn verify_password(&self, password: impl AsRef<[u8]>, hash: &str) -> PasswordResult<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| PasswordError::InvalidFormat(e.to_string()))?;

        Ok(self
            .argon2
            .verify_password(password.as_ref(), &parsed_hash)
            .is_ok())
    }
}

impl Default for PasswordManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing_and_verification() {
        let manager = PasswordManager::new();
        let password = "secure_password123";

        // Hash the password
        let hash = manager.hash_password(password).expect("Failed to hash password");

        // Verify correct password
        assert!(
            manager
                .verify_password(password, &hash)
                .expect("Failed to verify password"),
            "Password verification should succeed"
        );

        // Verify incorrect password
        assert!(
            !manager
                .verify_password("wrong_password", &hash)
                .expect("Failed to verify password"),
            "Password verification should fail for wrong password"
        );
    }

    #[test]
    fn test_invalid_hash_format() {
        let manager = PasswordManager::new();
        let result = manager.verify_password("password", "invalid_hash");
        assert!(matches!(result, Err(PasswordError::InvalidFormat(_))));
    }
} 