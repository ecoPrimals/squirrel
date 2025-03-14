use std::fmt;

/// Trait for UI events that can be handled by components
pub trait Event: fmt::Debug {
    /// Returns the event type
    fn event_type(&self) -> EventType;
}

/// Types of UI events
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    /// Keyboard event
    Key,
    /// Mouse event
    Mouse,
    /// Window resize event
    Resize,
    /// Focus event
    Focus,
    /// Custom event
    Custom,
} 