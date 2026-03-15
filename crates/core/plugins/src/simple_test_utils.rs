// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

/// Simple test utilities that don't depend on external crates
use std::fmt;

/// A simplified version of the PluginError for testing
#[derive(Debug)]
pub enum SimpleError {
    /// General error
    General(String),
    /// Not found error
    NotFound(String),
    /// Validation error
    Validation(String),
}

impl fmt::Display for SimpleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SimpleError::General(msg) => write!(f, "Error: {}", msg),
            SimpleError::NotFound(msg) => write!(f, "Not found: {}", msg),
            SimpleError::Validation(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

/// A simplified Result type
pub type SimpleResult<T> = Result<T, SimpleError>;

/// Utility function to create a not found error
pub fn not_found<T>(msg: &str) -> SimpleResult<T> {
    Err(SimpleError::NotFound(msg.to_string()))
}

/// Utility function to create a validation error
pub fn validation_error<T>(msg: &str) -> SimpleResult<T> {
    Err(SimpleError::Validation(msg.to_string()))
}

/// Utility function to create a general error
pub fn general_error<T>(msg: &str) -> SimpleResult<T> {
    Err(SimpleError::General(msg.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_error_display() {
        let errors = [
            (SimpleError::General("General error".to_string()), "Error: General error"),
            (SimpleError::NotFound("Item not found".to_string()), "Not found: Item not found"),
            (SimpleError::Validation("Invalid input".to_string()), "Validation error: Invalid input"),
        ];
        
        for (error, expected) in errors.iter() {
            assert_eq!(error.to_string(), *expected);
        }
    }
    
    #[test]
    fn test_error_creation_functions() {
        // Test not_found
        let not_found_result: SimpleResult<()> = not_found("Item not found");
        match not_found_result {
            Err(SimpleError::NotFound(msg)) => assert_eq!(msg, "Item not found"),
            _ => panic!("Expected NotFound error"),
        }
        
        // Test validation_error
        let validation_result: SimpleResult<()> = validation_error("Invalid input");
        match validation_result {
            Err(SimpleError::Validation(msg)) => assert_eq!(msg, "Invalid input"),
            _ => panic!("Expected Validation error"),
        }
        
        // Test general_error
        let general_result: SimpleResult<()> = general_error("Something went wrong");
        match general_result {
            Err(SimpleError::General(msg)) => assert_eq!(msg, "Something went wrong"),
            _ => panic!("Expected General error"),
        }
    }
} 