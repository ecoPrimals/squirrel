use std::fmt;
use crate::core::error::Result;
use crate::ui::components::{Component, ComponentRegistry};
use crate::ui::events::Event;
use crate::ui::theme::Theme;

pub struct GUI {
    registry: ComponentRegistry,
    theme: Theme,
}

impl GUI {
    pub fn new() -> Result<Self> {
        let registry = ComponentRegistry::new();
        let theme = Theme::default();

        Ok(Self {
            registry,
            theme,
        })
    }

    pub fn init(&mut self) -> Result<()> {
        // TODO: Initialize GUI backend
        Ok(())
    }

    pub fn cleanup(&mut self) -> Result<()> {
        // TODO: Cleanup GUI backend
        Ok(())
    }

    pub fn draw(&mut self) -> Result<()> {
        // TODO: Implement GUI rendering
        Ok(())
    }

    pub fn handle_event(&mut self, event: Box<dyn Event>) -> Result<bool> {
        for component in self.registry.components_mut() {
            if let Err(e) = component.handle_event(event) {
                eprintln!("Error handling event: {}", e);
            }
        }

        Ok(false)
    }

    pub fn register_component(&mut self, component: Box<dyn Component>) -> Result<()> {
        self.registry.register(component)
    }
} 