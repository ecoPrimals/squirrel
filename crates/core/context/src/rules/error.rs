//! Error types for the rules module
use std::fmt;
use std::io;
use std::path::PathBuf;

/// Result type for rule operations
pub type Result<T> = std::result::Result<T, RuleError>;

/// Error type for rule operations
#[derive(Debug)]
pub enum RuleError {
    /// IO error
    IoError(io::Error),
    /// Serialization/Deserialization error
    SerializationError(String),
    /// Invalid rule format
    InvalidFormat(String),
    /// Rule not found
    NotFound(String),
    /// Rule already exists
    AlreadyExists(String),
    /// Directory operation error
    DirectoryError(String),
    /// Validation error
    ValidationError(String),
    /// Plugin error
    PluginError(String),
    /// Plugin not found
    PluginNotFound(String),
    /// Evaluation error
    EvaluationError(String),
    /// Action execution error
    ActionExecutionError(String),
    /// Invalid path error
    InvalidPath(String),
    /// Invalid type error
    InvalidType(String),
    /// Rule parse error
    ParseError(String),
    /// Rule dependency error
    DependencyError(String),
    /// Rule circular dependency error
    CircularDependencyError(String),
    /// Rule validation error
    RuleValidationError {
        /// Rule ID
        rule_id: String,
        /// List of validation errors
        errors: Vec<String>,
    },
    /// Path not found in file system
    PathNotFound(PathBuf),
}

impl fmt::Display for RuleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuleError::IoError(e) => write!(f, "IO error: {e}"),
            RuleError::SerializationError(e) => write!(f, "Serialization error: {e}"),
            RuleError::InvalidFormat(e) => write!(f, "Invalid rule format: {e}"),
            RuleError::NotFound(id) => write!(f, "Rule not found: {id}"),
            RuleError::AlreadyExists(id) => write!(f, "Rule already exists: {id}"),
            RuleError::DirectoryError(e) => write!(f, "Directory operation error: {e}"),
            RuleError::ValidationError(e) => write!(f, "Validation error: {e}"),
            RuleError::PluginError(e) => write!(f, "Plugin error: {e}"),
            RuleError::PluginNotFound(e) => write!(f, "Plugin not found: {e}"),
            RuleError::EvaluationError(e) => write!(f, "Evaluation error: {e}"),
            RuleError::ActionExecutionError(e) => write!(f, "Action execution error: {e}"),
            RuleError::InvalidPath(e) => write!(f, "Invalid path: {e}"),
            RuleError::InvalidType(e) => write!(f, "Invalid type: {e}"),
            RuleError::ParseError(e) => write!(f, "Parse error: {e}"),
            RuleError::DependencyError(e) => write!(f, "Dependency error: {e}"),
            RuleError::CircularDependencyError(e) => write!(f, "Circular dependency error: {e}"),
            RuleError::RuleValidationError { rule_id, errors } => {
                write!(f, "Rule validation error for '{rule_id}': ")?;
                for (i, error) in errors.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{error}")?;
                }
                Ok(())
            }
            RuleError::PathNotFound(path) => write!(f, "Path not found: {}", path.display()),
        }
    }
}

impl std::error::Error for RuleError {}

impl From<io::Error> for RuleError {
    fn from(error: io::Error) -> Self {
        RuleError::IoError(error)
    }
}

impl From<serde_json::Error> for RuleError {
    fn from(error: serde_json::Error) -> Self {
        RuleError::SerializationError(error.to_string())
    }
}

impl From<serde_yaml::Error> for RuleError {
    fn from(error: serde_yaml::Error) -> Self {
        RuleError::SerializationError(error.to_string())
    }
}
