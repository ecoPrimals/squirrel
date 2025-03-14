use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use crate::ui::error::UiError;
use crate::ui::Component;
use crate::ui::{layout::Rect, theme::Theme};
use std::fmt;
use std::io;

/// A unique identifier for a UI component.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct ComponentId {
    /// The string value of the component ID.
    id: String,
}

impl ComponentId {
    /// Creates a new component ID from any value that can be converted into a String.
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }

    /// Returns the string representation of the component ID.
    pub fn as_str(&self) -> &str {
        &self.id
    }
}

/// The current state of a UI component.
#[derive(Debug, Clone)]
pub struct ComponentState {
    /// Key-value pairs representing the component's state.
    values: HashMap<String, String>,
}

impl ComponentState {
    /// Creates a new empty component state.
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    /// Sets a state value for the given key.
    pub fn set(&mut self, key: &str, value: String) {
        self.values.insert(key.to_string(), value);
    }

    /// Gets the state value for the given key.
    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    /// Removes the state value for the given key.
    pub fn remove(&mut self, key: &str) {
        self.values.remove(key);
    }

    /// Removes all state values.
    pub fn clear(&mut self) {
        self.values.clear();
    }
}

impl Default for ComponentState {
    fn default() -> Self {
        Self::new()
    }
}

/// A registry for managing UI components and their state.
///
/// The component registry maintains a collection of components and their associated
/// state. It provides methods for registering, updating, and retrieving components
/// and their state information.
pub struct ComponentRegistry {
    /// The components registered in the system.
    components: RwLock<HashMap<ComponentId, Arc<Mutex<dyn Component>>>>,
}

impl fmt::Debug for ComponentRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ComponentRegistry")
            .field("components", &"<components>")
            .finish()
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ComponentRegistry {
    /// Creates a new empty component registry.
    pub fn new() -> Self {
        Self {
            components: RwLock::new(HashMap::new()),
        }
    }

    /// Registers a component with the given ID
    pub fn register<T>(&self, component: Arc<Mutex<T>>) -> Result<(), UiError>
    where
        T: Component + 'static
    {
        let mut components = self.components.write()
            .map_err(|_| UiError::Component("Failed to lock components for writing".to_string()))?;
        
        let id = {
            let guard = component.lock()
                .map_err(|_| UiError::Component("Failed to lock component".to_string()))?;
            guard.id().clone()
        };

        components.insert(id, component);
        Ok(())
    }

    /// Gets a reference to a component by ID
    pub fn get(&self, id: &ComponentId) -> Option<Arc<Mutex<dyn Component>>> {
        self.components.read().ok()?.get(id).cloned()
    }

    /// Gets all registered components
    pub fn get_all(&self) -> Vec<Arc<Mutex<dyn Component>>> {
        self.components.read()
            .map(|components| components.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Unregisters a component by ID
    pub fn unregister(&self, id: &ComponentId) -> Result<(), UiError> {
        let mut components = self.components.write()
            .map_err(|_| UiError::Component("Failed to lock components for writing".to_string()))?;
        components.remove(id);
        Ok(())
    }

    /// Clears all registered components
    pub fn clear_all(&self) -> Result<(), UiError> {
        let mut components = self.components.write()
            .map_err(|_| UiError::Component("Failed to lock components for writing".to_string()))?;
        components.clear();
        Ok(())
    }

    /// Updates all registered components.
    pub fn update_all(&self) -> Result<(), UiError> {
        let components = self.components.read()
            .map_err(|_| UiError::Component("Failed to lock components for reading".to_string()))?;
        
        for component in components.values() {
            let mut guard = component.lock()
                .map_err(|_| UiError::Component("Failed to lock component".to_string()))?;
            guard.update()
                .map_err(|e| UiError::Component(e.to_string()))?;
        }
        Ok(())
    }

    /// Renders all registered components.
    pub fn render_all<W: io::Write>(
        &self,
        writer: &mut W,
        rect: Rect,
        theme: &Theme,
    ) -> Result<(), UiError> {
        let components = self.components.read()
            .map_err(|_| UiError::Component("Failed to lock components for reading".to_string()))?;
        
        for component in components.values() {
            let guard = component.lock()
                .map_err(|_| UiError::Component("Failed to lock component".to_string()))?;
            guard.render(writer, rect, theme)
                .map_err(|e| UiError::Component(e.to_string()))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::layout::Size;

    struct TestComponent {
        id: ComponentId,
        #[allow(dead_code)]
        value: i32,
    }

    impl TestComponent {
        fn new(id: impl Into<String>, value: i32) -> Self {
            Self {
                id: ComponentId::new(id),
                value,
            }
        }
    }

    impl Component for TestComponent {
        fn id(&self) -> &ComponentId {
            &self.id
        }

        fn render(&self, _writer: &mut dyn io::Write, _rect: Rect, _theme: &Theme) -> io::Result<()> {
            Ok(())
        }

        fn minimum_size(&self) -> Size {
            Size::new(0, 0)
        }

        fn preferred_size(&self) -> Size {
            Size::new(0, 0)
        }

        fn update(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_component_registration() {
        let registry = ComponentRegistry::new();
        let component = TestComponent::new("test", 42);
        let component = Arc::new(Mutex::new(component));
        registry.register(component.clone()).unwrap();

        let id = ComponentId::new("test");
        let retrieved = registry.get(&id).unwrap();
        let guard = retrieved.lock().unwrap();
        assert_eq!(guard.id().as_str(), "test");
    }

    #[test]
    fn test_component_type_lookup() {
        let registry = ComponentRegistry::new();
        let component = TestComponent::new("test", 42);
        let component = Arc::new(Mutex::new(component));
        registry.register(component.clone()).unwrap();

        let components = registry.get_all();
        assert_eq!(components.len(), 1);
    }

    #[test]
    fn test_component_unregister() {
        let registry = ComponentRegistry::new();
        let component = TestComponent::new("test", 42);
        let component = Arc::new(Mutex::new(component));
        registry.register(component.clone()).unwrap();

        let id = ComponentId::new("test");
        registry.unregister(&id).unwrap();

        assert!(registry.get(&id).is_none());
    }

    #[test]
    fn test_component_clear_all() {
        let registry = ComponentRegistry::new();
        let component1 = TestComponent::new("test1", 42);
        let component2 = TestComponent::new("test2", 43);
        
        registry.register(Arc::new(Mutex::new(component1))).unwrap();
        registry.register(Arc::new(Mutex::new(component2))).unwrap();

        registry.clear_all().unwrap();

        assert_eq!(registry.get_all().len(), 0);
    }
} 