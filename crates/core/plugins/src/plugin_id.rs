// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Plugin ID type
//!
//! This module provides a type-safe plugin identifier that supports both
//! UUID-based and string-based IDs for backward compatibility and flexibility.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::{Hash, Hasher};
use uuid::Uuid;

/// Type-safe plugin identifier
///
/// This provides a modern, flexible plugin ID that:
/// - Supports any string format (UUIDs, names, custom IDs)
/// - Provides type safety (can't mix up with regular strings)
/// - Zero-cost abstraction (newtype pattern)
/// - Backward compatible with UUID-based systems
///
/// # Examples
///
/// ```
/// use squirrel_plugins::PluginId;
/// use uuid::Uuid;
///
/// // Create from UUID (backward compatibility)
/// let id1 = PluginId::from(Uuid::new_v4());
///
/// // Create from string
/// let id2 = PluginId::from_string("my-plugin");
///
/// // Generate new UUID-based ID
/// let id3 = PluginId::new();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginId(String);

impl PluginId {
    /// Generate a new UUID-based plugin ID
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Create a plugin ID from a string
    #[must_use]
    pub fn from_string(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Get the ID as a string slice
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert to owned String
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }

    /// Parse from a UUID string
    ///
    /// # Errors
    ///
    /// Returns an error if the string is not a valid UUID.
    pub fn from_uuid_string(s: &str) -> Result<Self, uuid::Error> {
        Uuid::parse_str(s)?;
        Ok(Self(s.to_string()))
    }

    /// Try to parse as UUID
    ///
    /// Returns Some(Uuid) if this ID is a valid UUID, None otherwise.
    #[must_use]
    pub fn as_uuid(&self) -> Option<Uuid> {
        Uuid::parse_str(&self.0).ok()
    }
}

impl Default for PluginId {
    fn default() -> Self {
        Self::new()
    }
}

// Backward compatibility: Convert from UUID
impl From<Uuid> for PluginId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid.to_string())
    }
}

// Convert from String
impl From<String> for PluginId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

// Convert from &str
impl From<&str> for PluginId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

// Convert to String (consumes self)
impl From<PluginId> for String {
    fn from(id: PluginId) -> Self {
        id.0
    }
}

// Display implementation
impl fmt::Display for PluginId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Equality based on string content
impl PartialEq for PluginId {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for PluginId {}

// Hash based on string content
impl Hash for PluginId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

// Compare with strings directly
impl PartialEq<str> for PluginId {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for PluginId {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl PartialEq<String> for PluginId {
    fn eq(&self, other: &String) -> bool {
        self.0 == *other
    }
}

// Compare with UUID for backward compatibility
impl PartialEq<Uuid> for PluginId {
    fn eq(&self, other: &Uuid) -> bool {
        self.as_uuid().map_or(false, |u| u == *other)
    }
}

impl AsRef<str> for PluginId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_id_creation() {
        let id1 = PluginId::new();
        assert!(id1.as_uuid().is_some());

        let id2 = PluginId::from_string("custom-id");
        assert_eq!(id2.as_str(), "custom-id");
        assert!(id2.as_uuid().is_none());
    }

    #[test]
    fn test_plugin_id_from_uuid() {
        let uuid = Uuid::new_v4();
        let id = PluginId::from(uuid);
        assert_eq!(id.as_uuid(), Some(uuid));
    }

    #[test]
    fn test_plugin_id_equality() {
        let id1 = PluginId::from_string("test");
        let id2 = PluginId::from_string("test");
        let id3 = PluginId::from_string("other");

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
        assert_eq!(id1, "test");
        assert_ne!(id1, "other");
    }

    #[test]
    fn test_plugin_id_hash() {
        use std::collections::HashMap;

        let id = PluginId::from_string("test");
        let mut map = HashMap::new();
        map.insert(id.clone(), "value");

        assert_eq!(map.get(&id), Some(&"value"));
    }

    #[test]
    fn test_plugin_id_display() {
        let id = PluginId::from_string("test-plugin");
        assert_eq!(format!("{}", id), "test-plugin");
    }
}
