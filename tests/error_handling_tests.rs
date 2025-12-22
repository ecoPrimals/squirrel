//! Error handling comprehensive tests

use squirrel::error::*;
use std::io;

#[test]
fn test_error_creation() {
    let error = PrimalError::InvalidInput("test error".to_string());
    assert!(error.to_string().contains("test error"));
}

#[test]
fn test_error_chain() {
    let source = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let error = PrimalError::IoError(source);
    
    assert!(error.to_string().contains("I/O error"));
}

#[test]
fn test_error_from_string() {
    let error: PrimalError = "custom error".into();
    assert!(matches!(error, PrimalError::Custom(_)));
}

#[test]
fn test_error_debug_formatting() {
    let error = PrimalError::NotFound("resource".to_string());
    let debug_str = format!("{:?}", error);
    assert!(!debug_str.is_empty());
}

#[test]
fn test_error_display_formatting() {
    let error = PrimalError::ConfigurationError("invalid config".to_string());
    let display_str = format!("{}", error);
    assert!(display_str.contains("Configuration"));
}

#[test]
fn test_result_handling() {
    fn may_fail() -> Result<i32, PrimalError> {
        Err(PrimalError::InvalidInput("test".to_string()))
    }
    
    let result = may_fail();
    assert!(result.is_err());
}

#[test]
fn test_error_conversion_io() {
    let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
    let primal_err: PrimalError = io_err.into();
    
    assert!(matches!(primal_err, PrimalError::IoError(_)));
}

#[test]
fn test_error_context_preservation() {
    let error = PrimalError::NotFound("user:123".to_string());
    assert!(error.to_string().contains("123"));
}

#[test]
fn test_error_equality() {
    let err1 = PrimalError::InvalidInput("test".to_string());
    let err2 = PrimalError::InvalidInput("test".to_string());
    
    assert_eq!(format!("{:?}", err1), format!("{:?}", err2));
}

#[test]
fn test_error_propagation() {
    fn inner() -> Result<(), PrimalError> {
        Err(PrimalError::Timeout)
    }
    
    fn outer() -> Result<(), PrimalError> {
        inner()?;
        Ok(())
    }
    
    assert!(outer().is_err());
}

