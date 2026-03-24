// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use std::error::Error;
use std::fmt;

/// Errors that can occur in the plugin system
#[derive(Debug)]
pub enum PluginError {
    /// Plugin not found
    NotFound(String),
    /// Plugin already exists
    AlreadyExists(String),
    /// IO error
    IoError(std::io::Error),
    /// Plugin loading error
    LoadError(String),
    /// Plugin initialization error
    InitError(String),
    /// Plugin validation error
    ValidationError(String),
    /// Command registration error
    RegisterError(String),
    /// Security error
    SecurityError(String),
    /// Unknown plugin error
    Unknown(String),
}

impl PluginError {
    /// Create a new NotFound error
    pub fn plugin_not_found(name: &str) -> Self {
        PluginError::NotFound(name.to_string())
    }

    /// Create a new AlreadyExists error
    pub fn plugin_already_exists(name: &str) -> Self {
        PluginError::AlreadyExists(name.to_string())
    }
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PluginError::NotFound(name) => write!(f, "Plugin not found: {}", name),
            PluginError::AlreadyExists(name) => write!(f, "Plugin already exists: {}", name),
            PluginError::IoError(err) => write!(f, "IO error: {}", err),
            PluginError::LoadError(msg) => write!(f, "Plugin loading error: {}", msg),
            PluginError::InitError(msg) => write!(f, "Plugin initialization error: {}", msg),
            PluginError::ValidationError(msg) => write!(f, "Plugin validation error: {}", msg),
            PluginError::RegisterError(msg) => write!(f, "Command registration error: {}", msg),
            PluginError::SecurityError(msg) => write!(f, "Security error: {}", msg),
            PluginError::Unknown(msg) => write!(f, "Unknown plugin error: {}", msg),
        }
    }
}

impl Error for PluginError {}

impl From<std::io::Error> for PluginError {
    fn from(err: std::io::Error) -> Self {
        PluginError::IoError(err)
    }
}

impl From<String> for PluginError {
    fn from(err: String) -> Self {
        PluginError::Unknown(err)
    }
}

impl From<&str> for PluginError {
    fn from(err: &str) -> Self {
        PluginError::Unknown(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_error_display() {
        assert_eq!(
            PluginError::NotFound("foo".into()).to_string(),
            "Plugin not found: foo"
        );
        assert_eq!(
            PluginError::AlreadyExists("bar".into()).to_string(),
            "Plugin already exists: bar"
        );
        assert_eq!(
            PluginError::LoadError("load fail".into()).to_string(),
            "Plugin loading error: load fail"
        );
        assert_eq!(
            PluginError::InitError("init fail".into()).to_string(),
            "Plugin initialization error: init fail"
        );
        assert_eq!(
            PluginError::ValidationError("bad".into()).to_string(),
            "Plugin validation error: bad"
        );
        assert_eq!(
            PluginError::RegisterError("reg fail".into()).to_string(),
            "Command registration error: reg fail"
        );
        assert_eq!(
            PluginError::SecurityError("sec fail".into()).to_string(),
            "Security error: sec fail"
        );
        assert_eq!(
            PluginError::Unknown("unknown".into()).to_string(),
            "Unknown plugin error: unknown"
        );
    }

    #[test]
    fn test_io_error_display() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let plugin_err = PluginError::IoError(io_err);
        assert!(plugin_err.to_string().contains("IO error"));
    }

    #[test]
    fn test_plugin_not_found_helper() {
        let err = PluginError::plugin_not_found("test-plugin");
        assert!(matches!(err, PluginError::NotFound(ref s) if s == "test-plugin"));
    }

    #[test]
    fn test_plugin_already_exists_helper() {
        let err = PluginError::plugin_already_exists("test-plugin");
        assert!(matches!(err, PluginError::AlreadyExists(ref s) if s == "test-plugin"));
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let err: PluginError = io_err.into();
        assert!(matches!(err, PluginError::IoError(_)));
    }

    #[test]
    fn test_from_string() {
        let err: PluginError = "something broke".to_string().into();
        assert!(matches!(err, PluginError::Unknown(ref s) if s == "something broke"));
    }

    #[test]
    fn test_from_str() {
        let err: PluginError = "oops".into();
        assert!(matches!(err, PluginError::Unknown(ref s) if s == "oops"));
    }

    #[test]
    fn test_error_trait() {
        let err = PluginError::NotFound("test".into());
        let _: &dyn Error = &err;
    }
}
