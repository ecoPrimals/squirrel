use dashboard_core::data::DashboardData;
use ratatui::{
    layout::Rect,
    Frame,
};
use crossterm::event::{KeyCode, MouseEvent};

/// Widget manager trait
pub trait WidgetManager {
    /// Get widget name
    fn name(&self) -> &str;
    
    /// Check if widget is enabled
    fn enabled(&self) -> bool;
    
    /// Enable or disable widget
    fn set_enabled(&mut self, enabled: bool);
    
    /// Update widget state from dashboard data
    fn update(&mut self, data: &DashboardData);
    
    /// Render widget to frame
    fn render(&self, f: &mut Frame, area: Rect);
    
    /// Handle keyboard input
    fn handle_input(&mut self, key: crossterm::event::KeyEvent) -> bool;
    
    /// Process a tick event for animations or periodic updates
    fn tick(&mut self) {
        // Default implementation does nothing
    }
    
    /// Handle a key code event
    fn handle_key(&mut self, key: KeyCode) -> bool {
        // Default implementation delegates to handle_input
        self.handle_input(crossterm::event::KeyEvent::new(
            key,
            crossterm::event::KeyModifiers::empty(),
        ))
    }
    
    /// Handle a mouse event
    fn handle_mouse(&mut self, _event: MouseEvent) -> bool {
        // Default implementation does nothing
        false
    }
} 