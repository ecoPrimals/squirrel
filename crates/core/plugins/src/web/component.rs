// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Component handling for web plugins
//!
//! This module provides structs and traits for handling UI components in web plugins.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

/// Types of UI components that can be provided by web plugins
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComponentType {
    /// A complete page
    Page,
    /// A partial component to be embedded in a page
    Partial,
    /// A navigation item
    Navigation,
    /// A dashboard widget
    Widget,
    /// A modal dialog
    Modal,
    /// A form component
    Form,
    /// A custom component type
    Custom(String),
}

impl ComponentType {
    /// Check if this is a page component
    #[must_use]
    pub const fn is_page(&self) -> bool {
        matches!(self, Self::Page)
    }

    /// Check if this is a partial component
    #[must_use]
    pub const fn is_partial(&self) -> bool {
        matches!(self, Self::Partial)
    }

    /// Check if this is a navigation component
    #[must_use]
    pub const fn is_navigation(&self) -> bool {
        matches!(self, Self::Navigation)
    }

    /// Check if this is a widget component
    #[must_use]
    pub const fn is_widget(&self) -> bool {
        matches!(self, Self::Widget)
    }

    /// Check if this is a modal component
    #[must_use]
    pub const fn is_modal(&self) -> bool {
        matches!(self, Self::Modal)
    }

    /// Check if this is a form component
    #[must_use]
    pub const fn is_form(&self) -> bool {
        matches!(self, Self::Form)
    }

    /// Check if this is a custom component
    #[must_use]
    pub const fn is_custom(&self) -> bool {
        matches!(self, Self::Custom(_))
    }
}

/// Represents a UI component provided by a web plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebComponent {
    /// Unique identifier for the component
    pub id: Uuid,
    /// Human-readable name of the component
    pub name: String,
    /// Description of the component
    pub description: String,
    /// Type of component
    pub component_type: ComponentType,
    /// Additional properties for the component
    pub properties: HashMap<String, Value>,
    /// Route path for page components (optional)
    pub route: Option<String>,
    /// Priority for ordering components (higher numbers come first)
    pub priority: i32,
    /// Required permissions to access this component
    pub permissions: Vec<String>,
    /// Navigation parent for hierarchical nav items (optional)
    pub parent: Option<String>,
    /// Icon name (optional)
    pub icon: Option<String>,
}

impl WebComponent {
    /// Create a new web component
    #[must_use]
    pub fn new(id: Uuid, name: String, description: String, component_type: ComponentType) -> Self {
        Self {
            id,
            name,
            description,
            component_type,
            properties: HashMap::new(),
            route: None,
            priority: 0,
            permissions: vec![],
            parent: None,
            icon: None,
        }
    }

    /// Add a property to the component
    #[must_use]
    pub fn with_property(mut self, key: &str, value: Value) -> Self {
        self.properties.insert(key.to_string(), value);
        self
    }

    /// Set the route for a page component
    #[must_use]
    pub fn with_route(mut self, route: &str) -> Self {
        self.route = Some(route.to_string());
        self
    }

    /// Set the priority for ordering
    #[must_use]
    pub const fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Add a required permission
    #[must_use]
    pub fn with_permission(mut self, permission: &str) -> Self {
        self.permissions.push(permission.to_string());
        self
    }

    /// Set the parent for navigation items
    #[must_use]
    pub fn with_parent(mut self, parent: &str) -> Self {
        self.parent = Some(parent.to_string());
        self
    }

    /// Set the icon
    #[must_use]
    pub fn with_icon(mut self, icon: &str) -> Self {
        self.icon = Some(icon.to_string());
        self
    }

    /// Check if user has permission to access this component
    #[must_use]
    pub fn check_permission(&self, user_permissions: &[String]) -> bool {
        if self.permissions.is_empty() {
            return true;
        }
        self.permissions
            .iter()
            .any(|p| user_permissions.contains(p))
    }
}

#[cfg(test)]
mod tests {
    use super::{ComponentType, WebComponent};
    use serde_json::json;
    use uuid::Uuid;

    #[test]
    fn component_type_variants_and_predicates() {
        let page = ComponentType::Page;
        assert!(page.is_page());
        let partial = ComponentType::Partial;
        assert!(partial.is_partial());
        let nav = ComponentType::Navigation;
        assert!(nav.is_navigation());
        let widget = ComponentType::Widget;
        assert!(widget.is_widget());
        let modal = ComponentType::Modal;
        assert!(modal.is_modal());
        let form = ComponentType::Form;
        assert!(form.is_form());
        let custom = ComponentType::Custom("x".into());
        assert!(custom.is_custom());
        for variant in [page, partial, nav, widget, modal, form, custom] {
            let d = format!("{variant:?}");
            assert!(!d.is_empty());
            let c2 = variant.clone();
            assert_eq!(variant, c2);
        }
    }

    #[test]
    fn web_component_builder_and_permission_check() {
        let id = Uuid::new_v4();
        let c = WebComponent::new(id, "n".into(), "d".into(), ComponentType::Page)
            .with_property("k", json!(1))
            .with_route("/r")
            .with_priority(3)
            .with_permission("p1")
            .with_parent("nav")
            .with_icon("ico");
        assert!(!c.check_permission(&[]));
        assert!(!c.check_permission(&["other".into()]));
        assert!(c.check_permission(&["p1".into()]));
        let open = WebComponent::new(id, "n2".into(), "d".into(), ComponentType::Widget);
        assert!(open.check_permission(&[]));
        let json = serde_json::to_string(&c).expect("should succeed");
        let back: WebComponent = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(back.name, c.name);
        assert_eq!(back.priority, 3);
    }
}
