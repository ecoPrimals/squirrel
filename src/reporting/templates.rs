//! Report template management.

use crate::reporting::ReportTemplate;
use anyhow::Result;
use std::collections::HashMap;

/// Manages report templates
pub struct TemplateManager {
    /// Available templates
    templates: HashMap<String, ReportTemplate>,
}

impl TemplateManager {
    /// Creates a new template manager
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    /// Adds a template to the manager
    pub fn add_template(&mut self, template: ReportTemplate) {
        self.templates.insert(template.id.clone(), template);
    }

    /// Gets a template by ID
    pub fn get_template(&self, id: &str) -> Option<&ReportTemplate> {
        self.templates.get(id)
    }

    /// Lists all available templates
    pub fn list_templates(&self) -> Vec<&ReportTemplate> {
        self.templates.values().collect()
    }

    /// Creates a new template
    pub fn create_template(&mut self, name: String, content: String, variables: Vec<String>) -> ReportTemplate {
        let template = ReportTemplate {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            content,
            variables,
        };
        self.add_template(template.clone());
        template
    }
} 