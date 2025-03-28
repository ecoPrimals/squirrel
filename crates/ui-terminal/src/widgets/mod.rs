//! UI Widgets for the terminal dashboard
//! 
//! This module provides custom widgets for the terminal dashboard.

pub mod chart;
pub mod health;
pub mod alerts;
pub mod metrics;
pub mod network;
pub mod protocol;

// Re-export widgets for easier access
pub use chart::ChartWidget;
pub use health::{HealthWidget, HealthStatus, HealthCheck};
pub use alerts::AlertsWidget;
pub use metrics::MetricsWidget;
pub use network::NetworkWidget;
pub use protocol::ProtocolWidget;

use ratatui::{
    layout::Rect,
    Frame,
};

/// Common trait for all widgets that can be rendered
pub trait Widget {
    /// Render the widget to the terminal frame
    fn render(&self, f: &mut Frame, area: Rect);
} 