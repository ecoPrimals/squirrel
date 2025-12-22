//! Visualization Type System
//!
//! Organized into logical modules for better maintainability:
//! - `core`: Core visualization types and structures
//! - `config`: Configuration structures and settings
//! - `theme`: Theme and layout types
//! - `display`: Display implementations and conversions

#![allow(dead_code)] // Many types defined for future use

pub mod config;
pub mod core;
pub mod display;
pub mod theme;

// Re-export commonly used types for backward compatibility
pub use config::*;
pub use core::*;
pub use theme::*;

// Display trait implementations are in the display module but don't need re-export
// as they're automatically available through trait imports
