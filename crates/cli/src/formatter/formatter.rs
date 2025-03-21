use thiserror::Error;
use std::io;
use squirrel_commands::CommandError;

#[derive(Debug, Error)]
pub enum FormatterError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Error formatting: {0}")]
    Format(String),
    #[error("Unknown formatter: {0}")]
    UnknownFormatter(String),
}

impl From<FormatterError> for CommandError {
    fn from(error: FormatterError) -> Self {
        CommandError::new(error.to_string())
    }
} 