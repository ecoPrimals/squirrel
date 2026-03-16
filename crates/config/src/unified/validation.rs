// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Unified Configuration Validation
//!
//! This module provides centralized validation logic for all configuration types
//! in the Squirrel ecosystem. It consolidates validation patterns and provides
//! reusable validation functions.
//!
//! # Design Principles
//!
//! 1. **Single Responsibility**: Each validator focuses on one aspect of configuration
//! 2. **Reusability**: Common validation patterns are extracted into helper functions
//! 3. **Clear Errors**: Validation errors provide actionable information
//! 4. **Composability**: Validators can be composed for complex validation
//!
//! # Example Usage
//!
//! ```ignore
//! use squirrel_mcp_config::unified::validation::Validator;
//!
//! let result = Validator::validate_port(8080)?;
//! let result = Validator::validate_timeout_secs(30)?;
//! let result = Validator::validate_hostname("example.com")?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::net::IpAddr;
use std::path::Path;

/// Validation result type
pub type ValidationResult<T> = Result<T, ValidationError>;

/// Configuration validation errors
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    /// A field has an invalid value.
    #[error("Invalid {field}: {reason}")]
    Invalid {
        /// Name of the invalid field.
        field: String,
        /// Explanation of why the value is invalid.
        reason: String,
    },

    /// A required field is missing.
    #[error("Missing required field: {field}")]
    Missing {
        /// Name of the missing field.
        field: String,
    },

    /// A field violates a constraint (e.g. port > 0).
    #[error("Value {field} must be {constraint}")]
    Constraint {
        /// Name of the constrained field.
        field: String,
        /// Description of the constraint.
        constraint: String,
    },

    /// Conflicting configuration values.
    #[error("Conflict: {description}")]
    Conflict {
        /// Description of the conflict.
        description: String,
    },

    /// A referenced file or directory does not exist.
    #[error("File not found: {path}")]
    FileNotFound {
        /// Path that was not found.
        path: String,
    },
}

/// Unified configuration validator
///
/// Provides reusable validation functions for common configuration patterns.
pub struct Validator;

impl Validator {
    // ==================== PORT VALIDATION ====================

    /// Validate a network port number
    ///
    /// Ensures the port is in the valid range (1-65535) and returns the port.
    ///
    /// # Errors
    /// - Port is 0
    /// - Port is > 65535
    pub fn validate_port(port: u16) -> ValidationResult<u16> {
        if port == 0 {
            return Err(ValidationError::Constraint {
                field: "port".to_string(),
                constraint: "> 0".to_string(),
            });
        }
        Ok(port)
    }

    /// Validate that two ports are different
    ///
    /// # Errors
    /// - Ports are the same
    pub fn validate_ports_differ(
        port1: u16,
        port2: u16,
        name1: &str,
        name2: &str,
    ) -> ValidationResult<()> {
        if port1 == port2 {
            return Err(ValidationError::Conflict {
                description: format!(
                    "{} and {} ports must be different (both are {})",
                    name1, name2, port1
                ),
            });
        }
        Ok(())
    }

    // ==================== TIMEOUT VALIDATION ====================

    /// Validate a timeout value in seconds
    ///
    /// Ensures timeout is > 0.
    ///
    /// # Errors
    /// - Timeout is 0
    pub fn validate_timeout_secs(timeout_secs: u64, field: &str) -> ValidationResult<u64> {
        if timeout_secs == 0 {
            return Err(ValidationError::Constraint {
                field: field.to_string(),
                constraint: "> 0".to_string(),
            });
        }
        Ok(timeout_secs)
    }

    /// Validate a timeout value with a maximum
    ///
    /// Ensures timeout is > 0 and <= max.
    ///
    /// # Errors
    /// - Timeout is 0
    /// - Timeout exceeds max
    pub fn validate_timeout_with_max(
        timeout_secs: u64,
        max_secs: u64,
        field: &str,
    ) -> ValidationResult<u64> {
        Self::validate_timeout_secs(timeout_secs, field)?;

        if timeout_secs > max_secs {
            return Err(ValidationError::Constraint {
                field: field.to_string(),
                constraint: format!("≤ {} seconds", max_secs),
            });
        }
        Ok(timeout_secs)
    }

    /// Validate that timeout A is less than timeout B
    ///
    /// # Errors
    /// - Timeout A >= Timeout B
    pub fn validate_timeout_ordering(
        timeout_a: u64,
        timeout_b: u64,
        name_a: &str,
        name_b: &str,
    ) -> ValidationResult<()> {
        if timeout_a >= timeout_b {
            return Err(ValidationError::Constraint {
                field: name_a.to_string(),
                constraint: format!("< {} ({} seconds)", name_b, timeout_b),
            });
        }
        Ok(())
    }

    // ==================== NETWORK VALIDATION ====================

    /// Validate an IP address string
    ///
    /// # Errors
    /// - String is not a valid IP address
    pub fn validate_ip_address(ip: &str) -> ValidationResult<IpAddr> {
        ip.parse::<IpAddr>().map_err(|_| ValidationError::Invalid {
            field: "ip_address".to_string(),
            reason: format!("Invalid IP address: {}", ip),
        })
    }

    /// Validate a hostname
    ///
    /// Checks that hostname follows RFC 1123 conventions:
    /// - Total length <= 253 characters
    /// - Labels <= 63 characters
    /// - Only alphanumeric and hyphens
    /// - Labels don't start or end with hyphen
    ///
    /// # Errors
    /// - Empty hostname
    /// - Invalid format
    pub fn validate_hostname(hostname: &str) -> ValidationResult<()> {
        if hostname.is_empty() {
            return Err(ValidationError::Missing {
                field: "hostname".to_string(),
            });
        }

        if hostname.len() > 253 {
            return Err(ValidationError::Invalid {
                field: "hostname".to_string(),
                reason: "Hostname length exceeds 253 characters".to_string(),
            });
        }

        let valid = hostname.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && label.chars().all(|c| c.is_alphanumeric() || c == '-')
                && !label.starts_with('-')
                && !label.ends_with('-')
        });

        if !valid {
            return Err(ValidationError::Invalid {
                field: "hostname".to_string(),
                reason: format!("Invalid hostname format: {}", hostname),
            });
        }

        Ok(())
    }

    /// Validate a URL scheme
    ///
    /// # Errors
    /// - Scheme is not in allowed list
    pub fn validate_url_scheme(url: &str, allowed_schemes: &[&str]) -> ValidationResult<()> {
        if let Some(pos) = url.find("://") {
            let scheme = &url[..pos];
            if !allowed_schemes.contains(&scheme) {
                return Err(ValidationError::Invalid {
                    field: "url_scheme".to_string(),
                    reason: format!(
                        "URL scheme '{}' not allowed. Must be one of: {:?}",
                        scheme, allowed_schemes
                    ),
                });
            }
        } else {
            return Err(ValidationError::Invalid {
                field: "url".to_string(),
                reason: "URL missing scheme (e.g., http://, https://)".to_string(),
            });
        }
        Ok(())
    }

    // ==================== FILE VALIDATION ====================

    /// Validate that a file exists
    ///
    /// # Errors
    /// - File does not exist
    pub fn validate_file_exists(path: &Path, field: &str) -> ValidationResult<()> {
        if !path.exists() {
            return Err(ValidationError::FileNotFound {
                path: format!("{}: {}", field, path.display()),
            });
        }
        if !path.is_file() {
            return Err(ValidationError::Invalid {
                field: field.to_string(),
                reason: format!("Path is not a file: {}", path.display()),
            });
        }
        Ok(())
    }

    /// Validate that a directory exists
    ///
    /// # Errors
    /// - Directory does not exist
    pub fn validate_dir_exists(path: &Path, field: &str) -> ValidationResult<()> {
        if !path.exists() {
            return Err(ValidationError::FileNotFound {
                path: format!("{}: {}", field, path.display()),
            });
        }
        if !path.is_dir() {
            return Err(ValidationError::Invalid {
                field: field.to_string(),
                reason: format!("Path is not a directory: {}", path.display()),
            });
        }
        Ok(())
    }

    /// Validate that a path's parent directory exists
    ///
    /// Useful for validating paths where the file doesn't exist yet but should be creatable.
    ///
    /// # Errors
    /// - Parent directory does not exist
    pub fn validate_parent_dir_exists(path: &Path, field: &str) -> ValidationResult<()> {
        if let Some(parent) = path.parent()
            && !parent.exists()
        {
            return Err(ValidationError::FileNotFound {
                path: format!("{} parent directory: {}", field, parent.display()),
            });
        }
        Ok(())
    }

    // ==================== STRING VALIDATION ====================

    /// Validate that a string is not empty
    ///
    /// # Errors
    /// - String is empty
    pub fn validate_not_empty(value: &str, field: &str) -> ValidationResult<()> {
        if value.is_empty() {
            return Err(ValidationError::Missing {
                field: field.to_string(),
            });
        }
        Ok(())
    }

    /// Validate that a string contains only alphanumeric characters and allowed symbols
    ///
    /// # Errors
    /// - String contains invalid characters
    pub fn validate_alphanumeric_with(
        value: &str,
        field: &str,
        allowed: &[char],
    ) -> ValidationResult<()> {
        let valid = value
            .chars()
            .all(|c| c.is_alphanumeric() || allowed.contains(&c));

        if !valid {
            return Err(ValidationError::Invalid {
                field: field.to_string(),
                reason: format!(
                    "Contains invalid characters. Only alphanumeric and {:?} allowed",
                    allowed
                ),
            });
        }
        Ok(())
    }

    /// Validate semantic version string (major.minor.patch)
    ///
    /// # Errors
    /// - Invalid semver format
    pub fn validate_semver(version: &str) -> ValidationResult<()> {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() < 2 || parts.len() > 3 {
            return Err(ValidationError::Invalid {
                field: "version".to_string(),
                reason: format!(
                    "Invalid semver format: {}. Expected major.minor or major.minor.patch",
                    version
                ),
            });
        }

        for part in parts {
            if part.parse::<u32>().is_err() {
                return Err(ValidationError::Invalid {
                    field: "version".to_string(),
                    reason: format!("Invalid semver part: {}. Must be numeric", part),
                });
            }
        }

        Ok(())
    }

    // ==================== NUMERIC VALIDATION ====================

    /// Validate that a value is greater than a minimum
    ///
    /// # Errors
    /// - Value <= min
    pub fn validate_greater_than<T: PartialOrd + std::fmt::Display>(
        value: T,
        min: T,
        field: &str,
    ) -> ValidationResult<T> {
        if value <= min {
            return Err(ValidationError::Constraint {
                field: field.to_string(),
                constraint: format!("> {}", min),
            });
        }
        Ok(value)
    }

    /// Validate that a value is in a range
    ///
    /// # Errors
    /// - Value < min or value > max
    pub fn validate_range<T: PartialOrd + std::fmt::Display>(
        value: T,
        min: T,
        max: T,
        field: &str,
    ) -> ValidationResult<T> {
        if value < min || value > max {
            return Err(ValidationError::Constraint {
                field: field.to_string(),
                constraint: format!("between {} and {}", min, max),
            });
        }
        Ok(value)
    }

    // ==================== SECURITY VALIDATION ====================

    /// Validate API key length (basic check)
    ///
    /// # Errors
    /// - API key too short
    pub fn validate_api_key(key: &str, min_length: usize, field: &str) -> ValidationResult<()> {
        if key.len() < min_length {
            return Err(ValidationError::Invalid {
                field: field.to_string(),
                reason: format!("API key too short (< {} characters)", min_length),
            });
        }
        Ok(())
    }

    /// Validate JWT secret minimum length
    ///
    /// # Errors
    /// - Secret too short (< 32 bytes recommended)
    pub fn validate_jwt_secret(secret: &str) -> ValidationResult<()> {
        if secret.len() < 32 {
            return Err(ValidationError::Invalid {
                field: "jwt_secret".to_string(),
                reason: "JWT secret should be at least 32 characters for security".to_string(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_port() {
        assert!(Validator::validate_port(8080).is_ok());
        assert!(Validator::validate_port(1).is_ok());
        assert!(Validator::validate_port(65535).is_ok());
        assert!(Validator::validate_port(0).is_err());
    }

    #[test]
    fn test_validate_ports_differ() {
        assert!(Validator::validate_ports_differ(8080, 8081, "http", "ws").is_ok());
        assert!(Validator::validate_ports_differ(8080, 8080, "http", "ws").is_err());
    }

    #[test]
    fn test_validate_timeout_secs() {
        assert!(Validator::validate_timeout_secs(30, "timeout").is_ok());
        assert!(Validator::validate_timeout_secs(0, "timeout").is_err());
    }

    #[test]
    fn test_validate_timeout_with_max() {
        assert!(Validator::validate_timeout_with_max(30, 60, "timeout").is_ok());
        assert!(Validator::validate_timeout_with_max(90, 60, "timeout").is_err());
        assert!(Validator::validate_timeout_with_max(0, 60, "timeout").is_err());
    }

    #[test]
    fn test_validate_timeout_ordering() {
        assert!(Validator::validate_timeout_ordering(10, 30, "connect", "request").is_ok());
        assert!(Validator::validate_timeout_ordering(30, 10, "connect", "request").is_err());
        assert!(Validator::validate_timeout_ordering(30, 30, "connect", "request").is_err());
    }

    #[test]
    fn test_validate_ip_address() {
        assert!(Validator::validate_ip_address("127.0.0.1").is_ok());
        assert!(Validator::validate_ip_address("0.0.0.0").is_ok());
        assert!(Validator::validate_ip_address("::1").is_ok());
        assert!(Validator::validate_ip_address("invalid").is_err());
    }

    #[test]
    fn test_validate_hostname() {
        assert!(Validator::validate_hostname("example.com").is_ok());
        assert!(Validator::validate_hostname("test-host.example.com").is_ok());
        assert!(Validator::validate_hostname("host123.example.com").is_ok());
        assert!(Validator::validate_hostname("").is_err());
        assert!(Validator::validate_hostname("-example.com").is_err());
        assert!(Validator::validate_hostname("example-.com").is_err());
    }

    #[test]
    fn test_validate_url_scheme() {
        assert!(Validator::validate_url_scheme("https://example.com", &["http", "https"]).is_ok());
        assert!(Validator::validate_url_scheme("http://example.com", &["http", "https"]).is_ok());
        assert!(Validator::validate_url_scheme("ftp://example.com", &["http", "https"]).is_err());
        assert!(Validator::validate_url_scheme("example.com", &["http", "https"]).is_err());
    }

    #[test]
    fn test_validate_not_empty() {
        assert!(Validator::validate_not_empty("test", "field").is_ok());
        assert!(Validator::validate_not_empty("", "field").is_err());
    }

    #[test]
    fn test_validate_alphanumeric_with() {
        assert!(Validator::validate_alphanumeric_with("test-name", "name", &['-']).is_ok());
        assert!(Validator::validate_alphanumeric_with("test_name", "name", &['-', '_']).is_ok());
        assert!(Validator::validate_alphanumeric_with("test@name", "name", &['-']).is_err());
    }

    #[test]
    fn test_validate_semver() {
        assert!(Validator::validate_semver("1.0.0").is_ok());
        assert!(Validator::validate_semver("1.0").is_ok());
        assert!(Validator::validate_semver("1.2.3").is_ok());
        assert!(Validator::validate_semver("1").is_err());
        assert!(Validator::validate_semver("1.0.0.0").is_err());
        assert!(Validator::validate_semver("invalid").is_err());
    }

    #[test]
    fn test_validate_greater_than() {
        assert!(Validator::validate_greater_than(10, 0, "value").is_ok());
        assert!(Validator::validate_greater_than(0, 0, "value").is_err());
        assert!(Validator::validate_greater_than(-5, 0, "value").is_err());
    }

    #[test]
    fn test_validate_range() {
        assert!(Validator::validate_range(50, 0, 100, "value").is_ok());
        assert!(Validator::validate_range(0, 0, 100, "value").is_ok());
        assert!(Validator::validate_range(100, 0, 100, "value").is_ok());
        assert!(Validator::validate_range(-1, 0, 100, "value").is_err());
        assert!(Validator::validate_range(101, 0, 100, "value").is_err());
    }

    #[test]
    fn test_validate_api_key() {
        assert!(Validator::validate_api_key("sk-12345678901234567890", 10, "api_key").is_ok());
        assert!(Validator::validate_api_key("short", 10, "api_key").is_err());
    }

    #[test]
    fn test_validate_jwt_secret() {
        assert!(Validator::validate_jwt_secret("this_is_a_very_long_secret_key_for_jwt").is_ok());
        assert!(Validator::validate_jwt_secret("short").is_err());
    }

    #[test]
    fn test_validation_error_display() {
        let err = ValidationError::Invalid {
            field: "port".to_string(),
            reason: "must be > 0".to_string(),
        };
        assert!(err.to_string().contains("port"));
        assert!(err.to_string().contains("must be > 0"));

        let err = ValidationError::Missing {
            field: "hostname".to_string(),
        };
        assert!(err.to_string().contains("hostname"));

        let err = ValidationError::Constraint {
            field: "timeout".to_string(),
            constraint: "> 0".to_string(),
        };
        assert!(err.to_string().contains("timeout"));

        let err = ValidationError::Conflict {
            description: "ports must differ".to_string(),
        };
        assert!(err.to_string().contains("Conflict"));

        let err = ValidationError::FileNotFound {
            path: "/nonexistent".to_string(),
        };
        assert!(err.to_string().contains("File not found"));
    }

    #[test]
    fn test_validate_file_exists_nonexistent() {
        let result =
            Validator::validate_file_exists(std::path::Path::new("/nonexistent/path"), "config");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_dir_exists_nonexistent() {
        let result =
            Validator::validate_dir_exists(std::path::Path::new("/nonexistent/dir"), "data_dir");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_hostname_too_long() {
        let long = "a".repeat(254);
        assert!(Validator::validate_hostname(&long).is_err());
    }
}
