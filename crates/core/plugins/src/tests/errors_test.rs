// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

#[cfg(test)]
mod tests {
    use crate::errors::{PluginError, Result};
    use std::io;
    use uuid::Uuid;
    
    #[test]
    fn test_plugin_error_display() {
        let errors = [
            (PluginError::PluginNotFound("plugin-id".to_string()), "Plugin not found: plugin-id"),
            (PluginError::InitializationError("Failed to initialize plugin".to_string()), "Plugin initialization error: Failed to initialize plugin"),
            (PluginError::CommandNotFound("test-command".to_string()), "Command not found: test-command"),
            (PluginError::DependencyError("Missing dependency".to_string()), "Plugin dependency error: Missing dependency"),
            (PluginError::ValidationError("Invalid plugin".to_string()), "Plugin validation error: Invalid plugin")
        ];
        
        for (error, expected) in errors.iter() {
            assert_eq!(error.to_string(), *expected);
        }
    }
    
    #[test]
    fn test_plugin_error_from_io_error() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let plugin_error: PluginError = io_error.into();
        
        match plugin_error {
            PluginError::IoError(err) => {
                assert_eq!(err.kind(), io::ErrorKind::NotFound);
                assert_eq!(err.to_string(), "File not found");
            }
            _ => panic!("Expected IoError variant"),
        }
    }
    
    #[test]
    fn test_plugin_error_chaining() {
        // Test error propagation and conversion
        fn function_that_fails() -> Result<()> {
            Err(PluginError::InitializationError("Initial error".to_string()))
        }
        
        fn function_that_propagates() -> Result<()> {
            function_that_fails()?;
            Ok(())
        }
        
        let result = function_that_propagates();
        assert!(result.is_err());
        
        match result {
            Err(PluginError::InitializationError(msg)) => {
                assert_eq!(msg, "Initial error");
            }
            _ => panic!("Expected InitializationError variant"),
        }
    }

    #[test]
    fn test_uuid_errors() {
        let id = Uuid::new_v4();
        let error = PluginError::NotFound(id);
        assert!(error.to_string().contains(&id.to_string()));

        let error = PluginError::AlreadyRegistered(id);
        assert!(error.to_string().contains(&id.to_string()));

        let error = PluginError::DependencyCycle(id);
        assert!(error.to_string().contains(&id.to_string()));
    }
} 