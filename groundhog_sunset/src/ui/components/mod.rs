/// UI components for building interactive terminal interfaces.
///
/// This module provides a collection of reusable UI components that can be used
/// to build interactive terminal user interfaces. Components include progress bars,
/// headers, input fields, and status indicators.
/// Progress bar and loading indicator components.
pub mod progress;
/// Event handling system for UI components.
pub mod event;
/// Component registration and management.
pub mod registry;
/// Header and title components.
pub mod header;
/// Input field and text entry components.
pub mod input;
/// Status message and notification components.
pub mod status;
/// Main application component.
pub mod app;
/// Table components.
pub mod table;

use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use crate::ui::{Component, Rect, Theme};
use crate::ui::error::UiError;
use crate::ui::components::registry::ComponentRegistry;

// Re-exports
pub use event::EventHandler;
pub use header::Header;
pub use input::Input;
pub use progress::Progress;
pub use registry::{ComponentId, ComponentState};
pub use status::Status;
pub use app::App;

/// The main component manager for the UI.
pub struct ComponentManager {
    /// The registry of all UI components, protected by a mutex for thread safety.
    registry: Arc<Mutex<ComponentRegistry>>,
}

impl ComponentManager {
    /// Creates a new component manager.
    pub fn new() -> Self {
        Self {
            registry: Arc::new(Mutex::new(ComponentRegistry::new())),
        }
    }

    /// Mounts a component into the UI system.
    pub fn mount<T>(&self, component: T) -> Result<(), UiError>
    where
        T: Component + 'static
    {
        let component = Arc::new(Mutex::new(component));
        let registry = self.registry.lock().map_err(|_| UiError::LockError)?;
        registry.register(component)
    }

    /// Unmounts a component from the UI system.
    pub fn unmount(&self, id: &ComponentId) -> Result<(), UiError> {
        let registry = self.registry.lock().map_err(|_| UiError::LockError)?;
        registry.unregister(id)
    }

    /// Renders all components.
    pub fn render<W>(&self, writer: &mut W, rect: Rect, theme: &Theme) -> Result<(), UiError>
    where
        W: Write,
    {
        let registry = self.registry.lock().map_err(|_| UiError::LockError)?;
        registry.render_all(writer, rect, theme)
    }

    /// Updates all components.
    pub fn update(&self) -> Result<(), UiError> {
        let registry = self.registry.lock().map_err(|_| UiError::LockError)?;
        registry.update_all()
    }
}

impl Default for ComponentManager {
    fn default() -> Self {
        Self::new()
    }
}

/// A container for managing multiple components.
pub struct Container {
    /// The registry that manages all components in this container.
    registry: ComponentRegistry,
}

impl Container {
    /// Creates a new container.
    pub fn new() -> Self {
        Self {
            registry: ComponentRegistry::new(),
        }
    }

    /// Registers a component with the container.
    pub fn register<T>(&mut self, component: T) -> Result<(), UiError>
    where
        T: Component + 'static
    {
        let component = Arc::new(Mutex::new(component));
        self.registry.register(component)
    }

    /// Unregisters a component from the container.
    pub fn unregister(&mut self, id: &ComponentId) -> Result<(), UiError> {
        self.registry.unregister(id)
    }

    /// Renders all components in the container.
    pub fn render<W>(&self, writer: &mut W, rect: Rect, theme: &Theme) -> Result<(), UiError>
    where
        W: Write,
    {
        self.registry.render_all(writer, rect, theme)
    }

    /// Updates all components in the container.
    pub fn update(&mut self) -> Result<(), UiError> {
        self.registry.update_all()
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    struct TestComponent {
        id: ComponentId,
        updated: Arc<AtomicBool>,
        rendered: Arc<AtomicBool>,
    }

    impl TestComponent {
        fn new(id: impl Into<String>) -> Self {
            Self {
                id: ComponentId::new(id),
                updated: Arc::new(AtomicBool::new(false)),
                rendered: Arc::new(AtomicBool::new(false)),
            }
        }
    }

    impl Component for TestComponent {
        fn id(&self) -> &ComponentId {
            &self.id
        }

        fn render(&self, _writer: &mut dyn Write, _rect: Rect, _theme: &Theme) -> io::Result<()> {
            self.rendered.store(true, Ordering::SeqCst);
            Ok(())
        }

        fn minimum_size(&self) -> Size {
            Size::new(0, 0)
        }

        fn preferred_size(&self) -> Size {
            Size::new(0, 0)
        }

        fn update(&mut self) -> io::Result<()> {
            self.updated.store(true, Ordering::SeqCst);
            Ok(())
        }
    }

    #[test]
    fn test_component_lifecycle() {
        let manager = ComponentManager::new();
        let component = TestComponent::new("test");
        
        let updated = component.updated.clone();
        let rendered = component.rendered.clone();
        let id = component.id().clone();

        // Test mount
        manager.mount(component).unwrap();

        // Test update and render
        manager.update().unwrap();
        manager.render(&mut Vec::new(), Rect::default(), &Theme::default()).unwrap();
        assert!(updated.load(Ordering::SeqCst));
        assert!(rendered.load(Ordering::SeqCst));

        // Test unmount
        manager.unmount(&id).unwrap();
    }
} 