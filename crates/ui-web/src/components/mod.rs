//! UI components for the Squirrel Web UI.
//!
//! This module provides UI components for the web-based user interface.

mod layout;
mod commands;
mod jobs;
mod status;
mod logs;
mod auth;

pub use layout::{Layout, Header, Footer, Sidebar, Navigation};
pub use commands::{CommandList, CommandDetail, CommandForm};
pub use jobs::{JobList, JobDetail, JobStatus};
pub use status::{SystemStatus, StatusIndicator, StatusCard};
pub use logs::{LogViewer, LogEntry, LogFilter};
pub use auth::{LoginForm, UserProfile, AuthStatus};

/// Component rendering trait
pub trait Component {
    /// Render the component to HTML
    fn render(&self) -> String;
    
    /// Get the component ID
    fn id(&self) -> String;
    
    /// Get the component name
    fn name(&self) -> String;
    
    /// Get the component type
    fn component_type(&self) -> ComponentType;
}

/// Component type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentType {
    /// Container component (layout, etc.)
    Container,
    /// Content component (data display, etc.)
    Content,
    /// Input component (form, etc.)
    Input,
    /// Navigation component (menu, etc.)
    Navigation,
    /// Indicator component (status, etc.)
    Indicator,
}

/// Create a new basic header component
pub fn create_header() -> Header {
    Header::new("Squirrel Web Interface", "0.1.0")
}

/// Create a new basic footer component
pub fn create_footer() -> Footer {
    Footer::new("Squirrel Web Interface", "2024")
}

/// Create a new basic navigation component
pub fn create_navigation() -> Navigation {
    let mut nav = Navigation::new();
    nav.add_item("Dashboard", "/");
    nav.add_item("Commands", "/commands");
    nav.add_item("Jobs", "/jobs");
    nav.add_item("Status", "/status");
    nav.add_item("Logs", "/logs");
    nav
}

/// Create a new basic layout component
pub fn create_layout() -> Layout {
    Layout::new(
        create_header(),
        create_navigation(),
        create_footer(),
    )
} 