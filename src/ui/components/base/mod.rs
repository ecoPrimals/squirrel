use std::sync::{Arc, RwLock};
use std::io::{self, Write};
use crate::ui::error::UiError;
use crate::ui::Component;
use super::{ComponentId, ComponentState};
use crate::ui::layout::{Rect, Size};
use crate::ui::theme::Theme;

/// A base component implementation that provides common functionality
pub struct BaseComponent {
    id: ComponentId,
    state: Arc<RwLock<ComponentState>>,
}

impl BaseComponent {
    pub fn new(id: ComponentId) -> Self {
        Self {
            id,
            state: Arc::new(RwLock::new(ComponentState::new())),
        }
    }

    /// Returns a reference to the component's state
    pub fn state(&self) -> Result<ComponentState, UiError> {
        self.state.read()
            .map(|guard| (*guard).clone())
            .map_err(|_| UiError::Component("Failed to acquire read lock".to_string()))
    }

    /// Sets a value in the component's state
    pub fn set_state(&self, key: &str, value: String) -> Result<(), UiError> {
        self.state.write()
            .map_err(|_| UiError::Component("Failed to acquire write lock".to_string()))
            .map(|mut guard| {
                guard.set(key, value);
            })
    }

    /// Gets a value from the component's state
    pub fn get_state(&self, key: &str) -> Result<Option<String>, UiError> {
        self.state.read()
            .map_err(|_| UiError::Component("Failed to acquire read lock".to_string()))
            .map(|guard| guard.get(key).cloned())
    }

    /// Removes a value from the component's state
    pub fn remove_state(&self, key: &str) -> Result<(), UiError> {
        self.state.write()
            .map_err(|_| UiError::Component("Failed to acquire write lock".to_string()))
            .map(|mut guard| {
                guard.remove(key);
            })
    }

    /// Clears all state
    pub fn clear_state(&self) -> Result<(), UiError> {
        self.state.write()
            .map_err(|_| UiError::Component("Failed to acquire write lock".to_string()))
            .map(|mut guard| {
                guard.clear();
            })
    }
}

impl Component for BaseComponent {
    fn id(&self) -> &ComponentId {
        &self.id
    }

    fn render(&self, _writer: &mut dyn Write, _rect: Rect, _theme: &Theme) -> io::Result<()> {
        Ok(())
    }

    fn minimum_size(&self) -> Size {
        Size::new(1, 1)
    }

    fn preferred_size(&self) -> Size {
        self.minimum_size()
    }
}

#[allow(dead_code)]
pub trait Focusable {
    fn focus(&mut self);
    fn blur(&mut self);
    fn is_focused(&self) -> bool;
}

#[allow(dead_code)]
pub trait Enableable {
    fn enable(&mut self);
    fn disable(&mut self);
    fn is_enabled(&self) -> bool;
}

#[allow(dead_code)]
pub trait Visible {
    fn show(&mut self);
    fn hide(&mut self);
    fn is_visible(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestComponent {
        base: BaseComponent,
    }

    impl TestComponent {
        fn new(id: impl Into<String>) -> Self {
            Self {
                base: BaseComponent::new(ComponentId::new(id)),
            }
        }
    }

    impl Component for TestComponent {
        fn id(&self) -> &ComponentId {
            self.base.id()
        }

        fn render(&self, _writer: &mut dyn Write, _rect: Rect, _theme: &Theme) -> io::Result<()> {
            // Basic implementation for testing
            Ok(())
        }

        fn minimum_size(&self) -> Size {
            Size::new(10, 1) // Minimal size for testing
        }

        fn preferred_size(&self) -> Size {
            Size::new(20, 1) // Preferred size for testing
        }
    }

    impl Focusable for TestComponent {
        fn focus(&mut self) {
            // Implementation for testing
        }

        fn blur(&mut self) {
            // Implementation for testing
        }

        fn is_focused(&self) -> bool {
            false // Default implementation for testing
        }
    }

    impl Enableable for TestComponent {
        fn enable(&mut self) {
            // Implementation for testing
        }

        fn disable(&mut self) {
            // Implementation for testing
        }

        fn is_enabled(&self) -> bool {
            true // Default implementation for testing
        }
    }

    impl Visible for TestComponent {
        fn show(&mut self) {
            // Implementation for testing
        }

        fn hide(&mut self) {
            // Implementation for testing
        }

        fn is_visible(&self) -> bool {
            true // Default implementation for testing
        }
    }

    #[test]
    fn test_component_state() {
        let component = TestComponent::new("test");
        component.base.set_state("test", "value".to_string()).unwrap();

        let value = component.base.get_state("test").unwrap().unwrap();
        assert_eq!(value, "value");

        component.base.remove_state("test").unwrap();
        assert!(component.base.get_state("test").unwrap().is_none());
    }
} 