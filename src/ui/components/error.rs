use std::fmt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ComponentError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Lock error")]
    Lock,
    
    #[error("Theme error: {0}")]
    Theme(String),
    
    #[error("Component error: {0}")]
    Component(String),
    
    #[error("Event error: {0}")]
    Event(String),
    
    #[error("Layout error: {0}")]
    Layout(String),
    
    #[error("Registry error: {0}")]
    Registry(String),
} 