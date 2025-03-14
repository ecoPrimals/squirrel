use std::fmt;
use serde::{Deserialize, Serialize};
use crate::core::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub key: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identifier {
    pub id: String,
    pub name: String,
    pub version: Version,
}

impl Identifier {
    pub fn new(id: impl Into<String>, name: impl Into<String>, version: Version) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            version,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub identifier: Identifier,
    pub description: String,
    pub metadata: Vec<Metadata>,
}

impl Resource {
    pub fn new(identifier: Identifier) -> Self {
        Self {
            identifier,
            description: String::new(),
            metadata: Vec::new(),
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    pub fn add_metadata(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.metadata.push(Metadata {
            key: key.into(),
            value,
        });
    }
} 