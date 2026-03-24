// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Compatibility utilities for plugin system interoperability
//!
//! This module provides conversion utilities between Squirrel's two plugin systems:
//! - **Internal system**: UUID-based with dependencies (`plugin::PluginMetadata`)
//! - **External system**: String-based, no dependencies (`squirrel_interfaces::plugins::PluginMetadata`)
//!
//! # When to Use These Conversions
//!
//! **Rare** - Most code should use one system consistently. Use conversions only when:
//! - Bridging internal and external plugin systems
//! - Migrating from internal to external (or vice versa)
//! - Testing or diagnostics
//!
//! # Important Notes
//!
//! ⚠️ **Conversions are lossy**:
//! - **Internal → External**: Loses dependency information
//! - **External → Internal**: Cannot reconstruct dependencies, generates new UUID if needed
//!
//! See **ADR-005** for complete architectural decision.

// Backward compatibility: conversion between internal and squirrel_interfaces PluginMetadata
#[expect(deprecated, reason = "backward compat: PluginMetadata conversion during migration")]
use crate::plugin::PluginMetadata as InternalMetadata;
use squirrel_interfaces::plugins::PluginMetadata as ExternalMetadata;
use uuid::Uuid;

/// Convert internal (UUID-based) metadata to external (String-based).
///
/// # What Happens
///
/// - UUID `id` is converted to String via `to_string()`
/// - All other fields are cloned directly
/// - ⚠️ **Dependencies are lost** (external system doesn't have dependencies)
///
/// # Example
///
/// ```rust,ignore
/// use squirrel_core::plugins::{plugin::PluginMetadata, compat};
///
/// let internal = PluginMetadata::new("test", "1.0.0", "Test", "Author")
///     .with_dependency(some_uuid);
///
/// let external = compat::to_external(&internal);
/// // external.id is a String, dependencies are gone
/// ```
// Backward compatibility: conversion layer during PluginMetadata migration
#[expect(deprecated, reason = "backward compat: PluginMetadata conversion during migration")]
pub fn to_external(internal: &InternalMetadata) -> ExternalMetadata {
    ExternalMetadata {
        id: internal.id.to_string(),
        name: internal.name.clone(),
        version: internal.version.clone(),
        description: internal.description.clone(),
        author: internal.author.clone(),
        capabilities: internal.capabilities.clone(),
    }
}

/// Convert external (String-based) metadata to internal (UUID-based).
///
/// # What Happens
///
/// - If `id` is a valid UUID string, parses it
/// - If `id` is NOT a valid UUID, generates a new random UUID
/// - All other fields are cloned directly
/// - ⚠️ **Dependencies are empty** (external plugins don't have dependencies)
///
/// # Example
///
/// ```rust,ignore
/// use squirrel_interfaces::plugins::PluginMetadata;
/// use squirrel_core::plugins::compat;
///
/// let external = PluginMetadata::new("test", "1.0.0", "Test", "Author");
///
/// let internal = compat::to_internal(&external);
/// // internal.id is a UUID (generated if external.id wasn't valid UUID)
/// // internal.dependencies is empty Vec
/// ```
// Backward compatibility: conversion layer during PluginMetadata migration
#[expect(deprecated, reason = "backward compat: PluginMetadata conversion during migration")]
pub fn to_internal(external: &ExternalMetadata) -> InternalMetadata {
    let id = Uuid::parse_str(&external.id).unwrap_or_else(|_| {
        // If external ID is not a valid UUID, generate a new one
        // This is expected behavior - external IDs are user-provided strings
        Uuid::new_v4()
    });

    InternalMetadata {
        id,
        name: external.name.clone(),
        version: external.version.clone(),
        description: external.description.clone(),
        author: external.author.clone(),
        capabilities: external.capabilities.clone(),
        dependencies: Vec::new(), // External plugins don't have dependencies
    }
}

/// Convert a collection of internal metadata to external.
///
/// Convenience wrapper around `to_external` for collections.
// Backward compatibility: conversion layer during PluginMetadata migration
#[expect(deprecated, reason = "backward compat: PluginMetadata conversion during migration")]
pub fn to_external_vec(internal: &[InternalMetadata]) -> Vec<ExternalMetadata> {
    internal.iter().map(to_external).collect()
}

/// Convert a collection of external metadata to internal.
///
/// Convenience wrapper around `to_internal` for collections.
// Backward compatibility: conversion layer during PluginMetadata migration
#[expect(deprecated, reason = "backward compat: PluginMetadata conversion during migration")]
pub fn to_internal_vec(external: &[ExternalMetadata]) -> Vec<InternalMetadata> {
    external.iter().map(to_internal).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_internal_to_external() {
        let internal = InternalMetadata {
            id: Uuid::new_v4(),
            name: "test-plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A test plugin".to_string(),
            author: "Test Author".to_string(),
            capabilities: vec!["cap1".to_string(), "cap2".to_string()],
            dependencies: Vec::new(),
            dependencies: vec![Uuid::new_v4()], // Will be lost
        };

        let external = to_external(&internal);

        assert_eq!(external.id, internal.id.to_string());
        assert_eq!(external.name, internal.name);
        assert_eq!(external.version, internal.version);
        assert_eq!(external.description, internal.description);
        assert_eq!(external.author, internal.author);
        assert_eq!(external.capabilities, internal.capabilities);
        // Note: dependencies are lost, this is expected
    }

    #[test]
    fn test_external_to_internal_with_valid_uuid() {
        let uuid = Uuid::new_v4();
        let external = ExternalMetadata {
            id: uuid.to_string(), // Valid UUID string
            name: "test-plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A test plugin".to_string(),
            author: "Test Author".to_string(),
            capabilities: vec!["cap1".to_string()],
            dependencies: Vec::new(),
        };

        let internal = to_internal(&external);

        assert_eq!(internal.id, uuid); // UUID was preserved
        assert_eq!(internal.name, external.name);
        assert_eq!(internal.version, external.version);
        assert_eq!(internal.dependencies.len(), 0); // No dependencies
    }

    #[test]
    fn test_external_to_internal_with_non_uuid() {
        let external = ExternalMetadata {
            id: "my-custom-plugin-id".to_string(), // Not a UUID
            name: "test-plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A test plugin".to_string(),
            author: "Test Author".to_string(),
            capabilities: vec![],
            dependencies: Vec::new(),
        };

        let internal = to_internal(&external);

        // UUID was generated (not the external string)
        assert_ne!(internal.id.to_string(), external.id);
        assert_eq!(internal.name, external.name);
        assert_eq!(internal.dependencies.len(), 0);
    }

    #[test]
    fn test_roundtrip_loses_dependencies() {
        let original = InternalMetadata {
            id: Uuid::new_v4(),
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: "test".to_string(),
            author: "test".to_string(),
            capabilities: vec![],
            dependencies: Vec::new(),
            dependencies: vec![Uuid::new_v4(), Uuid::new_v4()],
        };

        let external = to_external(&original);
        let back = to_internal(&external);

        // Basic fields survive
        assert_eq!(original.name, back.name);
        assert_eq!(original.version, back.version);

        // Dependencies don't survive roundtrip (this is expected)
        assert_eq!(back.dependencies.len(), 0);
        assert_ne!(original.dependencies.len(), back.dependencies.len());
    }

    #[test]
    fn test_vec_conversions() {
        let internals = vec![
            InternalMetadata {
                id: Uuid::new_v4(),
                name: "plugin1".to_string(),
                version: "1.0.0".to_string(),
                description: "test".to_string(),
                author: "test".to_string(),
                capabilities: vec![],
            dependencies: Vec::new(),
                dependencies: vec![],
            },
            InternalMetadata {
                id: Uuid::new_v4(),
                name: "plugin2".to_string(),
                version: "2.0.0".to_string(),
                description: "test".to_string(),
                author: "test".to_string(),
                capabilities: vec![],
            dependencies: Vec::new(),
                dependencies: vec![],
            },
        ];

        let externals = to_external_vec(&internals);
        assert_eq!(externals.len(), 2);
        assert_eq!(externals[0].name, "plugin1");
        assert_eq!(externals[1].name, "plugin2");

        let back = to_internal_vec(&externals);
        assert_eq!(back.len(), 2);
        assert_eq!(back[0].name, "plugin1");
        assert_eq!(back[1].name, "plugin2");
    }
}
