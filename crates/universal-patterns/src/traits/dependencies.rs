// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Primal dependency types.

use serde::{Deserialize, Serialize};
use std::hash::Hash;

/// Primal dependencies enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimalDependency {
    /// Requires authentication
    RequiresAuthentication {
        /// List of required authentication methods
        methods: Vec<String>,
    },
    /// Requires encryption
    RequiresEncryption {
        /// List of required encryption algorithms
        algorithms: Vec<String>,
    },
    /// Requires storage
    RequiresStorage {
        /// List of required storage types
        types: Vec<String>,
    },
    /// Requires compute
    RequiresCompute {
        /// List of required compute types
        types: Vec<String>,
    },
    /// Requires AI
    RequiresAI {
        /// List of required AI capabilities
        capabilities: Vec<String>,
    },
    /// Custom dependency
    Custom {
        /// Name of the custom dependency
        name: String,
        /// Custom requirements for the dependency
        requirements: String, // Changed from HashMap to String to fix Hash issues
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_requires_authentication_serde() {
        let dep = PrimalDependency::RequiresAuthentication {
            methods: vec!["oauth2".to_string(), "jwt".to_string()],
        };
        let json = serde_json::to_string(&dep).unwrap();
        let deserialized: PrimalDependency = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, dep);
    }

    #[test]
    fn test_requires_encryption_serde() {
        let dep = PrimalDependency::RequiresEncryption {
            algorithms: vec!["AES-256".to_string()],
        };
        let json = serde_json::to_string(&dep).unwrap();
        let deserialized: PrimalDependency = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, dep);
    }

    #[test]
    fn test_requires_storage_serde() {
        let dep = PrimalDependency::RequiresStorage {
            types: vec!["kv".to_string(), "blob".to_string()],
        };
        let json = serde_json::to_string(&dep).unwrap();
        let deserialized: PrimalDependency = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, dep);
    }

    #[test]
    fn test_requires_compute_serde() {
        let dep = PrimalDependency::RequiresCompute {
            types: vec!["gpu".to_string()],
        };
        let json = serde_json::to_string(&dep).unwrap();
        let deserialized: PrimalDependency = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, dep);
    }

    #[test]
    fn test_requires_ai_serde() {
        let dep = PrimalDependency::RequiresAI {
            capabilities: vec!["text-generation".to_string(), "embedding".to_string()],
        };
        let json = serde_json::to_string(&dep).unwrap();
        let deserialized: PrimalDependency = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, dep);
    }

    #[test]
    fn test_custom_dependency_serde() {
        let dep = PrimalDependency::Custom {
            name: "external-api".to_string(),
            requirements: "v2+".to_string(),
        };
        let json = serde_json::to_string(&dep).unwrap();
        let deserialized: PrimalDependency = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, dep);
    }

    #[test]
    fn test_dependency_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(PrimalDependency::RequiresAuthentication { methods: vec![] });
        set.insert(PrimalDependency::RequiresEncryption { algorithms: vec![] });
        set.insert(PrimalDependency::RequiresStorage { types: vec![] });
        set.insert(PrimalDependency::RequiresCompute { types: vec![] });
        set.insert(PrimalDependency::RequiresAI {
            capabilities: vec![],
        });
        set.insert(PrimalDependency::Custom {
            name: "x".to_string(),
            requirements: "y".to_string(),
        });
        assert_eq!(set.len(), 6);
    }
}
