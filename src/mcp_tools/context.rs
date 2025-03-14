use std::fmt;
use serde::{Deserialize, Serialize};
use crate::core::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    pub id: String,
    pub name: String,
    pub description: String,
    pub metadata: serde_json::Value,
}

impl Context {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            metadata: serde_json::Value::Null,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

pub struct ContextManager {
    contexts: Vec<Context>,
}

impl ContextManager {
    pub fn new() -> Self {
        Self {
            contexts: Vec::new(),
        }
    }

    pub fn add_context(&mut self, context: Context) {
        self.contexts.push(context);
    }

    pub fn get_context(&self, id: &str) -> Option<&Context> {
        self.contexts.iter().find(|c| c.id == id)
    }

    pub fn get_context_mut(&mut self, id: &str) -> Option<&mut Context> {
        self.contexts.iter_mut().find(|c| c.id == id)
    }

    pub fn remove_context(&mut self, id: &str) -> Option<Context> {
        if let Some(index) = self.contexts.iter().position(|c| c.id == id) {
            Some(self.contexts.remove(index))
        } else {
            None
        }
    }
} 