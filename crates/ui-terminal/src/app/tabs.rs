/// Represents the main application tabs available in the UI.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ActiveTab {
    /// System overview dashboard
    Overview,
    /// Detailed system metrics
    System,
    /// Network statistics
    Network,
    /// Protocol information
    Protocol,
    /// System alerts and notifications
    Alerts,
}

impl Default for ActiveTab {
    fn default() -> Self {
        Self::System
    }
}

impl std::fmt::Display for ActiveTab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Overview => "Overview",
            Self::System => "System",
            Self::Network => "Network",
            Self::Protocol => "Protocol",
            Self::Alerts => "Alerts",
        };
        write!(f, "{}", name)
    }
} 