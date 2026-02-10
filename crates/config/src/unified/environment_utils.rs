// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Environment Utilities
//!
//! This module provides utility functions for working with environment variables
//! and environment-aware configuration.
//!
//! # Example Usage
//!
//! ```ignore
//! use squirrel_mcp_config::unified::environment_utils::*;
//!
//! // Get environment with validation
//! let env = get_environment();
//!
//! // Get typed environment variable
//! let port: u16 = get_env_var("SQUIRREL_HTTP_PORT", "8080")?;
//!
//! // Get optional environment variable
//! if let Some(key) = get_env_var_optional("OPENAI_API_KEY") {
//!     println!("API key configured");
//! }
//! ```

use crate::environment::Environment;
use std::env;
use std::str::FromStr;

/// Get the current environment
///
/// Reads from `MCP_ENV` environment variable and returns the appropriate environment type.
/// Defaults to `Development` if not set or invalid.
///
/// # Example
///
/// ```ignore
/// use squirrel_mcp_config::unified::environment_utils::get_environment;
///
/// let env = get_environment();
/// if env.is_production() {
///     println!("Running in production mode");
/// }
/// ```
pub fn get_environment() -> Environment {
    Environment::from_env()
}

/// Get an environment variable with a default value
///
/// Reads an environment variable and parses it to the target type.
/// If the variable is not set, uses the provided default value.
///
/// # Type Parameters
///
/// - `T`: The target type (must implement `FromStr`)
///
/// # Arguments
///
/// - `key`: Environment variable name
/// - `default`: Default value as a string
///
/// # Errors
///
/// Returns an error if the environment variable exists but cannot be parsed.
///
/// # Example
///
/// ```ignore
/// use squirrel_mcp_config::unified::environment_utils::get_env_var;
///
/// // Get port with default
/// let port: u16 = get_env_var("SQUIRREL_HTTP_PORT", "8080").unwrap();
///
/// // Get boolean with default
/// let enabled: bool = get_env_var("SQUIRREL_TLS_ENABLED", "false").unwrap();
/// ```
pub fn get_env_var<T>(key: &str, default: &str) -> Result<T, String>
where
    T: FromStr,
    T::Err: std::fmt::Display,
{
    let value = env::var(key).unwrap_or_else(|_| default.to_string());
    value
        .parse::<T>()
        .map_err(|e| format!("Failed to parse {}: {}", key, e))
}

/// Get an optional environment variable
///
/// Reads an environment variable and returns `Some(value)` if set, `None` otherwise.
///
/// # Arguments
///
/// - `key`: Environment variable name
///
/// # Example
///
/// ```ignore
/// use squirrel_mcp_config::unified::environment_utils::get_env_var_optional;
///
/// if let Some(api_key) = get_env_var_optional("OPENAI_API_KEY") {
///     println!("API key is configured");
/// } else {
///     println!("API key not set");
/// }
/// ```
pub fn get_env_var_optional(key: &str) -> Option<String> {
    env::var(key).ok()
}

/// Get a required environment variable
///
/// Reads an environment variable and returns an error if not set.
///
/// # Arguments
///
/// - `key`: Environment variable name
///
/// # Errors
///
/// Returns an error if the environment variable is not set.
///
/// # Example
///
/// ```ignore
/// use squirrel_mcp_config::unified::environment_utils::get_env_var_required;
///
/// let database_url = get_env_var_required("DATABASE_URL")?;
/// # Ok::<(), String>(())
/// ```
pub fn get_env_var_required(key: &str) -> Result<String, String> {
    env::var(key).map_err(|_| format!("Required environment variable not set: {}", key))
}

/// Get a boolean environment variable
///
/// Reads an environment variable and interprets it as a boolean.
/// Recognizes: "true", "1", "yes", "on" as true (case-insensitive).
/// Recognizes: "false", "0", "no", "off" as false (case-insensitive).
///
/// # Arguments
///
/// - `key`: Environment variable name
/// - `default`: Default value if not set
///
/// # Example
///
/// ```ignore
/// use squirrel_mcp_config::unified::environment_utils::get_env_bool;
///
/// let tls_enabled = get_env_bool("SQUIRREL_TLS_ENABLED", false);
/// let metrics_enabled = get_env_bool("SQUIRREL_METRICS_ENABLED", true);
/// ```
pub fn get_env_bool(key: &str, default: bool) -> bool {
    env::var(key)
        .map(|v| {
            let v_lower = v.to_lowercase();
            matches!(v_lower.as_str(), "true" | "1" | "yes" | "on")
        })
        .unwrap_or(default)
}

/// Get environment-aware default value
///
/// Returns different default values based on the current environment.
///
/// # Type Parameters
///
/// - `T`: The return type
///
/// # Arguments
///
/// - `dev`: Value for development environment
/// - `test`: Value for testing environment
/// - `staging`: Value for staging environment
/// - `prod`: Value for production environment
///
/// # Example
///
/// ```ignore
/// use squirrel_mcp_config::unified::environment_utils::get_env_aware_default;
///
/// // Different log levels per environment
/// let log_level = get_env_aware_default("debug", "warn", "info", "info");
///
/// // Different timeouts per environment
/// let timeout = get_env_aware_default(120, 10, 60, 30);
/// ```
pub fn get_env_aware_default<T>(dev: T, test: T, staging: T, prod: T) -> T {
    let env = get_environment();
    match env {
        Environment::Development => dev,
        Environment::Testing => test,
        Environment::Staging => staging,
        Environment::Production => prod,
    }
}

/// Check if running in a specific environment
///
/// # Arguments
///
/// - `env_type`: The environment type to check
///
/// # Example
///
/// ```ignore
/// use squirrel_mcp_config::{Environment, unified::environment_utils::is_environment};
///
/// if is_environment(Environment::Production) {
///     println!("Production mode - enabling security");
/// }
/// ```
pub fn is_environment(env_type: Environment) -> bool {
    get_environment() == env_type
}

/// Get all Squirrel-prefixed environment variables
///
/// Returns a map of all environment variables that start with "SQUIRREL_".
/// Useful for debugging and configuration inspection.
///
/// # Example
///
/// ```ignore
/// use squirrel_mcp_config::unified::environment_utils::get_squirrel_env_vars;
///
/// let vars = get_squirrel_env_vars();
/// for (key, value) in vars {
///     println!("{} = {}", key, value);
/// }
/// ```
pub fn get_squirrel_env_vars() -> Vec<(String, String)> {
    env::vars()
        .filter(|(key, _)| key.starts_with("SQUIRREL_") || key.starts_with("MCP_"))
        .collect()
}

/// Validate that required environment variables are set for the current environment
///
/// # Arguments
///
/// - `required_vars`: List of environment variable names that must be set
///
/// # Returns
///
/// Returns `Ok(())` if all variables are set, or `Err` with a list of missing variables.
///
/// # Example
///
/// ```ignore
/// use squirrel_mcp_config::unified::environment_utils::validate_required_env_vars;
///
/// validate_required_env_vars(&[
///     "DATABASE_URL",
///     "OPENAI_API_KEY",
///     "SQUIRREL_JWT_SECRET",
/// ])?;
/// # Ok::<(), Vec<String>>(())
/// ```
pub fn validate_required_env_vars(required_vars: &[&str]) -> Result<(), Vec<String>> {
    let missing: Vec<String> = required_vars
        .iter()
        .filter(|var| env::var(var).is_err())
        .map(|s| s.to_string())
        .collect();

    if missing.is_empty() {
        Ok(())
    } else {
        Err(missing)
    }
}

/// Validate environment-specific requirements
///
/// Checks that required environment variables are set based on the current environment.
/// For example, production might require JWT secrets and API keys, while development doesn't.
///
/// # Example
///
/// ```ignore
/// use squirrel_mcp_config::unified::environment_utils::validate_environment_requirements;
///
/// // This will check environment-specific requirements
/// validate_environment_requirements()?;
/// # Ok::<(), String>(())
/// ```
pub fn validate_environment_requirements() -> Result<(), String> {
    let env = get_environment();

    match env {
        Environment::Production => {
            // Production requires security configuration
            let required = vec!["SQUIRREL_JWT_SECRET", "DATABASE_URL"];

            validate_required_env_vars(&required).map_err(|missing| {
                format!(
                    "Production environment requires these variables: {}",
                    missing.join(", ")
                )
            })?;

            // Validate JWT secret length
            if let Ok(secret) = env::var("SQUIRREL_JWT_SECRET") {
                if secret.len() < 32 {
                    return Err(
                        "SQUIRREL_JWT_SECRET must be at least 32 characters in production"
                            .to_string(),
                    );
                }
            }
        }
        Environment::Staging => {
            // Staging requires database
            validate_required_env_vars(&["DATABASE_URL"])
                .map_err(|_| "Staging environment requires DATABASE_URL".to_string())?;
        }
        _ => {
            // Development and testing are more lenient
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_get_environment() {
        // Save original value
        let original = env::var("MCP_ENV").ok();

        env::set_var("MCP_ENV", "development");
        assert_eq!(get_environment(), Environment::Development);

        env::set_var("MCP_ENV", "production");
        assert_eq!(get_environment(), Environment::Production);

        // Restore original or remove
        match original {
            Some(val) => env::set_var("MCP_ENV", val),
            None => env::remove_var("MCP_ENV"),
        }
    }

    #[test]
    fn test_get_env_var() {
        env::set_var("TEST_PORT", "8080");
        let port: u16 = get_env_var("TEST_PORT", "3000").unwrap();
        assert_eq!(port, 8080);

        let port: u16 = get_env_var("NONEXISTENT_PORT", "3000").unwrap();
        assert_eq!(port, 3000);
    }

    #[test]
    fn test_get_env_var_optional() {
        env::set_var("TEST_KEY", "value");
        assert_eq!(get_env_var_optional("TEST_KEY"), Some("value".to_string()));
        assert_eq!(get_env_var_optional("NONEXISTENT_KEY"), None);
    }

    #[test]
    fn test_get_env_bool() {
        env::set_var("TEST_ENABLED", "true");
        assert!(get_env_bool("TEST_ENABLED", false));

        env::set_var("TEST_ENABLED", "1");
        assert!(get_env_bool("TEST_ENABLED", false));

        env::set_var("TEST_ENABLED", "false");
        assert!(!get_env_bool("TEST_ENABLED", true));

        assert!(get_env_bool("NONEXISTENT", true));
    }

    #[test]
    #[serial]
    fn test_get_env_aware_default() {
        // Save original value
        let original = env::var("MCP_ENV").ok();

        env::set_var("MCP_ENV", "development");
        assert_eq!(get_env_aware_default(1, 2, 3, 4), 1);

        env::set_var("MCP_ENV", "production");
        assert_eq!(get_env_aware_default(1, 2, 3, 4), 4);

        // Restore original or remove
        match original {
            Some(val) => env::set_var("MCP_ENV", val),
            None => env::remove_var("MCP_ENV"),
        }
    }

    #[test]
    #[serial]
    fn test_is_environment() {
        // Use serial_test or cleanup to avoid race conditions
        env::set_var("MCP_ENV", "production");
        assert!(is_environment(Environment::Production));
        assert!(!is_environment(Environment::Development));

        // Cleanup
        env::remove_var("MCP_ENV");
    }

    #[test]
    fn test_validate_required_env_vars() {
        // Use unique var names to avoid conflicts with other tests
        let var1 = format!("REQUIRED_VAR_1_{}", std::process::id());
        let var2 = format!("REQUIRED_VAR_2_{}", std::process::id());

        env::set_var(&var1, "value1");
        env::set_var(&var2, "value2");

        assert!(validate_required_env_vars(&[&var1, &var2]).is_ok());
        assert!(validate_required_env_vars(&[&var1, "NONEXISTENT_VAR_UNIQUE"]).is_err());

        // Cleanup
        env::remove_var(&var1);
        env::remove_var(&var2);
    }
}
