use thiserror::Error;

/// Error types for the ui-terminal crate
#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Terminal error: {0}")]
    TerminalError(String),

    // Added variants for provider errors
    #[error("Data provider communication error: {0}")]
    DataProvider(String),

    #[error("Provider specific error: {0}")]
    ProviderSpecificError(String),

    // Add other UI-specific errors as needed
    // e.g., #[error("Configuration error: {0}")]
    // Config(String),

    // Placeholder for errors from the data provider or core layers if needed
    // #[error("Data provider error: {0}")]
    // DataProvider(String),
}

pub type Result<T> = std::result::Result<T, Error>; 