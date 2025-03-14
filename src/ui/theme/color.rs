/// Color roles used in the UI theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorRole {
    /// Primary color for important elements
    Primary,
    /// Secondary color for less important elements
    Secondary,
    /// Color for success states
    Success,
    /// Color for warning states
    Warning,
    /// Color for error states
    Error,
    /// Color for informational states
    Info,
    /// Background color
    Background,
    /// Foreground color
    Foreground,
    /// Border color
    Border,
    /// Selection color
    Selection,
    /// Highlight color
    Highlight,
    /// Disabled color
    Disabled,
    /// Text color
    Text,
} 