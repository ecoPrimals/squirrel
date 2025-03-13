//! Reporting module for generating and managing reports.
//!
//! This module provides reporting capabilities for the DataScienceBioLab project.

pub mod generators;
pub mod formats;
pub mod templates;

/// Re-export commonly used items
pub use generators::*;
pub use formats::*;
pub use templates::*; 