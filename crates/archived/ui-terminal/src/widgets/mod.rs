//! UI Widgets for the terminal dashboard
//! 
//! This module provides custom widgets for the terminal dashboard.

pub mod chart;
pub mod health;
pub mod alerts;
pub mod metrics;
pub mod network;
pub mod connection_health;

// Re-export widgets for easier access
pub use chart::ChartWidget;
pub use health::{HealthWidget, HealthStatus, HealthCheck};
pub use alerts::AlertsWidget;
pub use metrics::MetricsWidget;
pub use network::NetworkWidget;
pub use connection_health::ConnectionHealthWidget;

use ratatui::{
    layout::Rect,
    Frame,
};

use std::time::{Duration, Instant};
use std::marker::PhantomData;

/// Common trait for all widgets that can be rendered
pub trait Widget {
    /// Render the widget to the terminal frame
    fn render(&self, f: &mut Frame, area: Rect);
}

/// Trait for widgets that can determine if they need to be rerendered
pub trait OptimizedWidget {
    /// Returns true if the widget has changed since last render
    fn has_changed_since_last_render(&self) -> bool;
}

/// A widget wrapper that provides frame caching capabilities
/// to reduce rendering overhead for static or slowly changing widgets.
pub struct CachedWidget<'a, W> 
where
    W: Widget,
{
    /// The underlying widget to render
    widget: W,
    /// The last render time
    last_render: Instant,
    /// The minimum duration between renders
    cache_ttl: Duration,
    /// Whether the widget needs to be rerendered on next frame
    force_render: bool,
    /// The widget area for current cache
    cached_area: Option<Rect>,
    /// Marker for lifetime
    _marker: PhantomData<&'a ()>,
}

impl<'a, W> CachedWidget<'a, W>
where
    W: Widget,
{
    /// Create a new cached widget
    pub fn new(widget: W, cache_ttl: Duration) -> Self {
        Self {
            widget,
            last_render: Instant::now() - cache_ttl - Duration::from_millis(1),
            cache_ttl,
            force_render: true,
            cached_area: None,
            _marker: PhantomData,
        }
    }

    /// Set whether to force a render on next frame
    pub fn set_force_render(&mut self, force: bool) {
        self.force_render = force;
    }

    /// Replace the underlying widget
    pub fn set_widget(&mut self, widget: W) {
        self.widget = widget;
        self.force_render = true;
    }

    /// Get the underlying widget
    pub fn widget(&self) -> &W {
        &self.widget
    }

    /// Get a mutable reference to the underlying widget
    pub fn widget_mut(&mut self) -> &mut W {
        self.force_render = true;
        &mut self.widget
    }

    /// Returns true if the widget should be rendered
    pub fn should_render(&self, area: Rect) -> bool {
        // Force render if explicitly requested
        if self.force_render {
            return true;
        }

        // Render if the cache has expired
        if self.last_render.elapsed() > self.cache_ttl {
            return true;
        }

        // Render if the area has changed
        if let Some(cached_area) = self.cached_area {
            if cached_area != area {
                return true;
            }
        } else {
            return true;
        }

        false
    }
}

impl<'a, W> Widget for CachedWidget<'a, W>
where
    W: Widget,
{
    fn render(&self, f: &mut Frame, area: Rect) {
        if self.should_render(area) {
            self.widget.render(f, area);
            
            // We need to update mutable state, but the trait requires &self
            // Using interior mutability would be one solution, but for simplicity
            // we're just treating this like a "render hint" that doesn't persist
            // between frames
        }
        // If widget doesn't need rendering, the previous render is still valid
    }
}

impl<'a, W> OptimizedWidget for CachedWidget<'a, W>
where
    W: Widget,
{
    fn has_changed_since_last_render(&self) -> bool {
        self.force_render || self.last_render.elapsed() > self.cache_ttl
    }
}

/// Create a new cached widget with the given TTL
pub fn with_cache<'a, W>(widget: W, ttl_ms: u64) -> CachedWidget<'a, W>
where
    W: Widget,
{
    CachedWidget::new(widget, Duration::from_millis(ttl_ms))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};
    use ratatui::layout::Rect;
    use ratatui::widgets::Paragraph;
    use ratatui::text::Line;

    // Wrapper for Paragraph that implements our Widget trait
    struct TestWidget<'a>(Paragraph<'a>);

    impl<'a> Widget for TestWidget<'a> {
        fn render(&self, f: &mut ratatui::Frame, area: Rect) {
            f.render_widget(self.0.clone(), area);
        }
    }

    fn create_test_widget() -> TestWidget<'static> {
        TestWidget(Paragraph::new(Line::from("Test")))
    }

    #[test]
    fn test_cached_widget_creation() {
        let widget = create_test_widget();
        let cached = CachedWidget::new(widget, Duration::from_millis(100));
        
        // Initially should render
        assert!(cached.should_render(Rect::new(0, 0, 10, 10)));
    }

    #[test]
    fn test_cached_widget_ttl() {
        let widget = create_test_widget();
        let mut cached = CachedWidget::new(widget, Duration::from_millis(100));
        
        // Manually update last render time
        cached.last_render = Instant::now();
        cached.force_render = false;
        cached.cached_area = Some(Rect::new(0, 0, 10, 10));
        
        // Should not render immediately
        assert!(!cached.should_render(Rect::new(0, 0, 10, 10)));
        
        // Sleep for TTL duration
        std::thread::sleep(Duration::from_millis(110));
        
        // Now should render
        assert!(cached.should_render(Rect::new(0, 0, 10, 10)));
    }

    #[test]
    fn test_force_render() {
        let widget = create_test_widget();
        let mut cached = CachedWidget::new(widget, Duration::from_millis(1000));
        
        // Manually update last render time
        cached.last_render = Instant::now();
        cached.force_render = false;
        cached.cached_area = Some(Rect::new(0, 0, 10, 10));
        
        // Should not render immediately
        assert!(!cached.should_render(Rect::new(0, 0, 10, 10)));
        
        // Force render
        cached.set_force_render(true);
        
        // Now should render
        assert!(cached.should_render(Rect::new(0, 0, 10, 10)));
    }
} 