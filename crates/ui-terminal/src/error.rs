use std::fmt;
use std::io;

/// Custom error types for the terminal UI
#[derive(Debug)]
pub enum Error {
    /// IO errors
    Io(io::Error),
    
    /// Data provider errors
    DataProvider(String),
    
    /// Widget rendering errors
    Rendering(String),
    
    /// General application errors
    Application(String),
    
    /// External errors
    External(String),
}

impl Error {
    /// Create a new application error with a component and message
    pub fn new(component: &str, message: &str) -> Self {
        Error::Application(format!("[{}] {}", component, message))
    }
}

/// Result type alias for UI operations
pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(err) => write!(f, "IO error: {}", err),
            Error::DataProvider(msg) => write!(f, "Data provider error: {}", msg),
            Error::Rendering(msg) => write!(f, "Rendering error: {}", msg),
            Error::Application(msg) => write!(f, "Application error: {}", msg),
            Error::External(msg) => write!(f, "External error: {}", msg),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<String> for Error {
    fn from(err: String) -> Self {
        Error::External(err)
    }
}

impl From<&str> for Error {
    fn from(err: &str) -> Self {
        Error::External(err.to_string())
    }
}

impl From<squirrel_mcp::error::MCPError> for Error {
    fn from(err: squirrel_mcp::error::MCPError) -> Self {
        Error::External(format!("MCP error: {}", err))
    }
} 