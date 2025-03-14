/// UI components for building interactive terminal interfaces.
///
/// This module provides a collection of reusable UI components that can be used
/// to build interactive terminal user interfaces. Components include progress bars,
/// headers, input fields, and status indicators.
///
/// # Component Types
///
/// - `App`: Main application component
/// - `Event`: Event handling system
/// - `Header`: Header and title components
/// - `Input`: Input field and text entry components
/// - `Progress`: Progress bar and loading indicators
/// - `Registry`: Component registration and management
/// - `Status`: Status message and notification components
/// - `Table`: Table and grid components
///
/// # Examples
///
/// ```
/// use crate::ui::components::{ComponentManager, Header};
///
/// let manager = ComponentManager::new();
/// let header = Header::new("My Application");
/// manager.mount(header).expect("Failed to mount header");
/// ```

pub mod app;
pub mod header;
pub mod input;
pub mod progress;
pub mod status;
pub mod table;
pub mod registry;
pub mod error;
pub mod event;
pub mod layout;

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::fmt::{self, Display};
use std::hash::Hash;
use std::io::Write;
use thiserror::Error;

use ratatui::prelude::{Frame, Rect, Color, Style};
use crate::ui::theme::{Theme, Themeable, ColorRole, StyleRole};

pub use app::App;
pub use header::Header;
pub use input::Input;
pub use progress::Progress;
pub use status::Status;
pub use table::Table;
pub use registry::{ComponentId, ComponentRegistry};
pub use error::ComponentError;
pub use event::Event;
pub use layout::{Layout, Rect, Size};

/// A trait that defines the behavior of a UI component
pub trait Component: Send + Sync {
    /// Draw the component to the terminal
    fn draw(&self, frame: &mut Frame<'_>, area: Rect) -> std::result::Result<(), ComponentError>;
    
    /// Handle an event
    fn handle_event(&mut self, event: &dyn Event) -> std::result::Result<(), ComponentError>;
    
    /// Get the required size for the component
    fn required_size(&self) -> Size;
    
    /// Get the component's ID
    fn id(&self) -> ComponentId;
    
    /// Get the component's parent ID if it has one
    fn parent_id(&self) -> Option<ComponentId> {
        None
    }
}

/// A container for managing child components
#[derive(Debug)]
pub struct Container {
    id: ComponentId,
    parent_id: Option<ComponentId>,
    children: Vec<Box<dyn Component>>,
    layout: Layout,
    theme: Theme,
}

impl Container {
    pub fn new(id: ComponentId) -> Self {
        Self {
            id,
            parent_id: None,
            children: Vec::new(),
            layout: Layout::default(),
            theme: Theme::default(),
        }
    }

    pub fn with_parent(id: ComponentId, parent_id: ComponentId) -> Self {
        let mut container = Self::new(id);
        container.parent_id = Some(parent_id);
        container
    }

    pub fn add_child(&mut self, component: Box<dyn Component>) {
        self.children.push(component);
    }

    pub fn remove_child(&mut self, id: ComponentId) -> Option<Box<dyn Component>> {
        if let Some(index) = self.children.iter().position(|c| c.id() == id) {
            Some(self.children.remove(index))
        } else {
            None
        }
    }

    pub fn children(&self) -> &[Box<dyn Component>] {
        &self.children
    }

    pub fn children_mut(&mut self) -> &mut [Box<dyn Component>] {
        &mut self.children
    }
}

impl Component for Container {
    fn draw(&self, frame: &mut Frame<'_>, area: Rect) -> std::result::Result<(), ComponentError> {
        let areas = self.layout.split(area, self.children.len());
        for (child, area) in self.children.iter().zip(areas) {
            child.draw(frame, area)?;
        }
        Ok(())
    }

    fn handle_event(&mut self, event: &dyn Event) -> std::result::Result<(), ComponentError> {
        for child in &mut self.children {
            child.handle_event(event)?;
        }
        Ok(())
    }

    fn required_size(&self) -> Size {
        // Calculate the required size based on the layout and children
        let mut width = 0;
        let mut height = 0;
        for child in &self.children {
            let size = child.required_size();
            match self.layout.direction() {
                ratatui::layout::Direction::Horizontal => {
                    width += size.width;
                    height = height.max(size.height);
                }
                ratatui::layout::Direction::Vertical => {
                    width = width.max(size.width);
                    height += size.height;
                }
            }
        }
        Size::new(width, height)
    }

    fn id(&self) -> ComponentId {
        self.id
    }

    fn parent_id(&self) -> Option<ComponentId> {
        self.parent_id
    }
}

impl Themeable for Container {
    fn apply_theme(&mut self, theme: &Theme) -> std::result::Result<(), ComponentError> {
        self.theme = theme.clone();
        for child in &mut self.children {
            child.apply_theme(theme)?;
        }
        Ok(())
    }

    fn get_style(&self) -> Style {
        self.theme.get_style(StyleRole::Default)
    }

    fn get_color(&self, role: ColorRole) -> Color {
        self.theme.get_color(role)
    }

    fn theme(&self) -> Option<&Theme> {
        self.theme.as_ref()
    }

    fn theme_mut(&mut self) -> &mut Theme {
        self.theme.get_or_insert_with(Theme::default)
    }

    fn set_theme(&mut self, theme: Theme) {
        self.theme = Some(theme);
    }
}

/// Trait for UI components that can be styled with a theme.
pub trait Themeable {
    /// Applies the given theme to the component.
    fn apply_theme(&mut self, theme: &Theme) -> std::result::Result<(), ComponentError>;
    
    /// Returns the component's current theme.
    fn theme(&self) -> Option<&Theme>;

    /// Returns a mutable reference to the component's theme.
    fn theme_mut(&mut self) -> &mut Theme;

    /// Sets the theme for the component.
    fn set_theme(&mut self, theme: Theme);

    fn get_style(&self) -> Style {
        self.theme().map_or_else(Style::default, |t| t.get_style(StyleRole::Default))
    }

    fn get_color(&self, role: ColorRole) -> Color {
        self.theme().map_or_else(|| Color::White, |t| t.get_color(role))
    }
}

/// Trait for components that can be resized.
pub trait Resizable {
    /// Resizes the component to the given size.
    fn resize(&mut self, size: Size) -> std::result::Result<(), ComponentError>;
    
    /// Returns whether the component can be resized.
    fn resizable(&self) -> bool {
        true
    }
}

/// Trait for components that can be scrolled.
pub trait Scrollable {
    /// Scrolls the component by the given offset.
    fn scroll(&mut self, offset: i32) -> std::result::Result<(), ComponentError>;
    
    /// Returns whether the component can be scrolled.
    fn scrollable(&self) -> bool {
        true
    }
    
    /// Returns the current scroll position.
    fn scroll_position(&self) -> i32;
    
    /// Returns the maximum scroll position.
    fn max_scroll(&self) -> i32;
}

/// The main component manager for the UI.
///
/// The component manager is responsible for managing the lifecycle of UI components,
/// including mounting, unmounting, rendering, and updating components.
#[derive(Debug)]
pub struct ComponentManager {
    /// The registry of all UI components, protected by a mutex for thread safety.
    registry: Arc<Mutex<ComponentRegistry>>,
}

impl ComponentManager {
    /// Creates a new component manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            registry: Arc::new(Mutex::new(ComponentRegistry::new())),
        }
    }

    /// Mounts a component into the UI system.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type of component to mount, must implement `Component`
    ///
    /// # Errors
    ///
    /// Returns `ComponentError::LockError` if the registry mutex is poisoned.
    /// Returns `ComponentError::ComponentExists` if a component with the same ID already exists.
    pub fn mount<T>(&self, component: T) -> std::result::Result<(), ComponentError>
    where
        T: Component + 'static
    {
        let component = Arc::new(Mutex::new(component));
        let registry = self.registry.lock().map_err(|e| ComponentError::Lock(e.to_string()))?;
        registry.register(component)
    }

    /// Unmounts a component from the UI system.
    ///
    /// # Errors
    ///
    /// Returns `ComponentError::LockError` if the registry mutex is poisoned.
    /// Returns `ComponentError::ComponentNotFound` if the component doesn't exist.
    pub fn unmount(&self, id: &ComponentId) -> std::result::Result<(), ComponentError> {
        let registry = self.registry.lock().map_err(|e| ComponentError::Lock(e.to_string()))?;
        registry.unregister(id)
    }

    /// Renders all components.
    ///
    /// # Type Parameters
    ///
    /// * `W` - The type of writer to render to
    ///
    /// # Errors
    ///
    /// Returns `ComponentError::LockError` if the registry mutex is poisoned.
    /// Returns `ComponentError::RenderError` if rendering fails.
    pub fn render<W>(&self, writer: &mut W, rect: Rect, theme: &Theme) -> std::result::Result<(), ComponentError>
    where
        W: Write,
    {
        let registry = self.registry.lock().map_err(|e| ComponentError::Lock(e.to_string()))?;
        registry.render_all(writer, rect, theme)
    }

    /// Updates all components.
    ///
    /// # Errors
    ///
    /// Returns `ComponentError::LockError` if the registry mutex is poisoned.
    /// Returns `ComponentError::UpdateError` if updating fails.
    pub fn update(&self) -> std::result::Result<(), ComponentError> {
        let registry = self.registry.lock().map_err(|e| ComponentError::Lock(e.to_string()))?;
        registry.update_all()
    }

    /// Gets a reference to the component registry.
    ///
    /// # Errors
    ///
    /// Returns `ComponentError::LockError` if the registry mutex is poisoned.
    pub fn registry(&self) -> std::result::Result<impl std::ops::Deref<Target = ComponentRegistry>, ComponentError> {
        self.registry.lock().map_err(|e| ComponentError::Lock(e.to_string()))
    }
}

impl Default for ComponentManager {
    fn default() -> Self {
        Self::new()
    }
}

/// A container for managing multiple components.
///
/// Unlike `ComponentManager`, `Container` is not thread-safe and is designed
/// for managing components within a single thread.
#[derive(Debug)]
pub struct Container {
    /// The registry that manages all components in this container.
    registry: ComponentRegistry,
}

impl Container {
    /// Creates a new container.
    #[must_use]
    pub fn new() -> Self {
        Self {
            registry: ComponentRegistry::new(),
        }
    }

    /// Registers a component with the container.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type of component to register, must implement `Component`
    ///
    /// # Errors
    ///
    /// Returns `ComponentError::ComponentExists` if a component with the same ID already exists.
    pub fn register<T>(&mut self, component: T) -> std::result::Result<(), ComponentError>
    where
        T: Component + 'static
    {
        let component = Arc::new(Mutex::new(component));
        self.registry.register(component)
    }

    /// Unregisters a component from the container.
    ///
    /// # Errors
    ///
    /// Returns `ComponentError::ComponentNotFound` if the component doesn't exist.
    pub fn unregister(&mut self, id: &ComponentId) -> std::result::Result<(), ComponentError> {
        self.registry.unregister(id)
    }

    /// Renders all components in the container.
    ///
    /// # Type Parameters
    ///
    /// * `W` - The type of writer to render to
    ///
    /// # Errors
    ///
    /// Returns `ComponentError::RenderError` if rendering fails.
    pub fn render<W>(&self, writer: &mut W, rect: Rect, theme: &Theme) -> std::result::Result<(), ComponentError>
    where
        W: Write,
    {
        self.registry.render_all(writer, rect, theme)
    }

    /// Updates all components in the container.
    ///
    /// # Errors
    ///
    /// Returns `ComponentError::UpdateError` if updating fails.
    pub fn update(&mut self) -> std::result::Result<(), ComponentError> {
        self.registry.update_all()
    }

    /// Gets a reference to the component registry.
    #[must_use]
    pub const fn registry(&self) -> &ComponentRegistry {
        &self.registry
    }

    /// Gets a mutable reference to the component registry.
    pub fn registry_mut(&mut self) -> &mut ComponentRegistry {
        &mut self.registry
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

    #[derive(Debug)]
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

        fn handle_event(&mut self, _event: &Event) -> io::Result<()> {
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
        assert!(manager.mount(component).is_ok());

        // Test update and render
        assert!(manager.update().is_ok());
        assert!(manager.render(&mut Vec::new(), Rect::default(), &Theme::default()).is_ok());
        assert!(updated.load(Ordering::SeqCst));
        assert!(rendered.load(Ordering::SeqCst));

        // Test unmount
        assert!(manager.unmount(&id).is_ok());
    }

    #[test]
    fn test_container_lifecycle() {
        let mut container = Container::new();
        let component = TestComponent::new("test");
        
        let updated = component.updated.clone();
        let rendered = component.rendered.clone();
        let id = component.id().clone();

        // Test register
        assert!(container.register(component).is_ok());

        // Test update and render
        assert!(container.update().is_ok());
        assert!(container.render(&mut Vec::new(), Rect::default(), &Theme::default()).is_ok());
        assert!(updated.load(Ordering::SeqCst));
        assert!(rendered.load(Ordering::SeqCst));

        // Test unregister
        assert!(container.unregister(&id).is_ok());
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ComponentError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Lock error: {0}")]
    Lock(String),
    #[error("Theme error: {0}")]
    Theme(String),
    #[error("Component error: {0}")]
    Component(String),
    #[error("Event error: {0}")]
    Event(String),
    #[error("Layout error: {0}")]
    Layout(String),
    #[error("Registry error: {0}")]
    Registry(String),
}

impl fmt::Display for ComponentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComponentError::Io(err) => write!(f, "IO error: {}", err),
            ComponentError::Lock(msg) => write!(f, "Lock error: {}", msg),
            ComponentError::Theme(msg) => write!(f, "Theme error: {}", msg),
            ComponentError::Component(msg) => write!(f, "Component error: {}", msg),
            ComponentError::Layout(msg) => write!(f, "Layout error: {}", msg),
            ComponentError::Event(msg) => write!(f, "Event error: {}", msg),
            ComponentError::Registry(msg) => write!(f, "Registry error: {}", msg),
        }
    }
}

impl std::error::Error for ComponentError {} 