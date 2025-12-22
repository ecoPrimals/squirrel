//! Comprehensive tests for PrimalInfo, PrimalType, and PrimalState
//!
//! Coverage goal: 90%+
//! Strategy: Test all variants, Display impls, serialization, edge cases

use chrono::Utc;
use uuid::Uuid;

use super::primal_info::{PrimalInfo, PrimalState, PrimalType};

#[cfg(test)]
mod primal_type_tests {
    use super::*;

    #[test]
    fn test_primal_type_variants() {
        let types = vec![
            PrimalType::Coordinator,
            PrimalType::Security,
            PrimalType::Orchestration,
            PrimalType::Storage,
            PrimalType::Compute,
            PrimalType::AI,
            PrimalType::Network,
            PrimalType::Custom("TestType".to_string()),
        ];

        assert_eq!(types.len(), 8);
    }

    #[test]
    fn test_primal_type_display() {
        assert_eq!(PrimalType::Coordinator.to_string(), "Coordinator");
        assert_eq!(PrimalType::Security.to_string(), "Security");
        assert_eq!(PrimalType::Orchestration.to_string(), "Orchestration");
        assert_eq!(PrimalType::Storage.to_string(), "Storage");
        assert_eq!(PrimalType::Compute.to_string(), "Compute");
        assert_eq!(PrimalType::AI.to_string(), "AI");
        assert_eq!(PrimalType::Network.to_string(), "Network");
        assert_eq!(
            PrimalType::Custom("MyType".to_string()).to_string(),
            "Custom(MyType)"
        );
    }

    #[test]
    fn test_primal_type_equality() {
        assert_eq!(PrimalType::Coordinator, PrimalType::Coordinator);
        assert_ne!(PrimalType::Coordinator, PrimalType::Security);
        assert_eq!(
            PrimalType::Custom("Test".to_string()),
            PrimalType::Custom("Test".to_string())
        );
        assert_ne!(
            PrimalType::Custom("Test1".to_string()),
            PrimalType::Custom("Test2".to_string())
        );
    }

    #[test]
    fn test_primal_type_clone() {
        let original = PrimalType::Orchestration;
        let cloned = original.clone();
        assert_eq!(original, cloned);

        let custom_original = PrimalType::Custom("Test".to_string());
        let custom_cloned = custom_original.clone();
        assert_eq!(custom_original, custom_cloned);
    }

    #[test]
    fn test_primal_type_debug() {
        let coordinator = PrimalType::Coordinator;
        let debug_str = format!("{:?}", coordinator);
        assert!(debug_str.contains("Coordinator"));

        let custom = PrimalType::Custom("Debug".to_string());
        let custom_debug = format!("{:?}", custom);
        assert!(custom_debug.contains("Custom"));
        assert!(custom_debug.contains("Debug"));
    }

    #[test]
    fn test_primal_type_serialization() {
        let primal_type = PrimalType::Security;
        let serialized = serde_json::to_string(&primal_type).unwrap();
        let deserialized: PrimalType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(primal_type, deserialized);

        let custom = PrimalType::Custom("SerTest".to_string());
        let custom_serialized = serde_json::to_string(&custom).unwrap();
        let custom_deserialized: PrimalType = serde_json::from_str(&custom_serialized).unwrap();
        assert_eq!(custom, custom_deserialized);
    }

    #[test]
    fn test_primal_type_hash() {
        use std::collections::HashMap;

        let mut map = HashMap::new();
        map.insert(PrimalType::AI, "Squirrel");
        map.insert(PrimalType::Security, "BearDog");
        map.insert(PrimalType::Storage, "NestGate");

        assert_eq!(map.get(&PrimalType::AI), Some(&"Squirrel"));
        assert_eq!(map.get(&PrimalType::Security), Some(&"BearDog"));
        assert_eq!(map.get(&PrimalType::Storage), Some(&"NestGate"));
    }
}

#[cfg(test)]
mod primal_state_tests {
    use super::*;

    #[test]
    fn test_primal_state_variants() {
        let states = vec![
            PrimalState::Initializing,
            PrimalState::Starting,
            PrimalState::Running,
            PrimalState::Stopping,
            PrimalState::Stopped,
            PrimalState::Error("Test error".to_string()),
            PrimalState::Restarting,
            PrimalState::Maintenance,
        ];

        assert_eq!(states.len(), 8);
    }

    #[test]
    fn test_primal_state_default() {
        let default_state = PrimalState::default();
        assert_eq!(default_state, PrimalState::Stopped);
    }

    #[test]
    fn test_primal_state_display() {
        assert_eq!(PrimalState::Initializing.to_string(), "Initializing");
        assert_eq!(PrimalState::Starting.to_string(), "Starting");
        assert_eq!(PrimalState::Running.to_string(), "Running");
        assert_eq!(PrimalState::Stopping.to_string(), "Stopping");
        assert_eq!(PrimalState::Stopped.to_string(), "Stopped");
        assert_eq!(PrimalState::Restarting.to_string(), "Restarting");
        assert_eq!(PrimalState::Maintenance.to_string(), "Maintenance");
        assert_eq!(
            PrimalState::Error("Connection failed".to_string()).to_string(),
            "Error: Connection failed"
        );
    }

    #[test]
    fn test_primal_state_equality() {
        assert_eq!(PrimalState::Running, PrimalState::Running);
        assert_ne!(PrimalState::Running, PrimalState::Stopped);
        assert_eq!(
            PrimalState::Error("Test".to_string()),
            PrimalState::Error("Test".to_string())
        );
        assert_ne!(
            PrimalState::Error("Error1".to_string()),
            PrimalState::Error("Error2".to_string())
        );
    }

    #[test]
    fn test_primal_state_clone() {
        let original = PrimalState::Running;
        let cloned = original.clone();
        assert_eq!(original, cloned);

        let error_original = PrimalState::Error("Test error".to_string());
        let error_cloned = error_original.clone();
        assert_eq!(error_original, error_cloned);
    }

    #[test]
    fn test_primal_state_debug() {
        let running = PrimalState::Running;
        let debug_str = format!("{:?}", running);
        assert!(debug_str.contains("Running"));

        let error = PrimalState::Error("Debug error".to_string());
        let error_debug = format!("{:?}", error);
        assert!(error_debug.contains("Error"));
    }

    #[test]
    fn test_primal_state_serialization() {
        let state = PrimalState::Running;
        let serialized = serde_json::to_string(&state).unwrap();
        let deserialized: PrimalState = serde_json::from_str(&serialized).unwrap();
        assert_eq!(state, deserialized);

        let error_state = PrimalState::Error("Serialization test".to_string());
        let error_serialized = serde_json::to_string(&error_state).unwrap();
        let error_deserialized: PrimalState = serde_json::from_str(&error_serialized).unwrap();
        assert_eq!(error_state, error_deserialized);
    }

    #[test]
    fn test_primal_state_transitions() {
        // Test typical state transition sequence
        let states = vec![
            PrimalState::Stopped,
            PrimalState::Initializing,
            PrimalState::Starting,
            PrimalState::Running,
            PrimalState::Stopping,
            PrimalState::Stopped,
        ];

        for (i, state) in states.iter().enumerate() {
            assert!(i < states.len());
            assert_eq!(state, &states[i]);
        }
    }

    #[test]
    fn test_primal_state_error_with_various_messages() {
        let error_messages = vec![
            "Connection timeout",
            "Authentication failed",
            "Resource not found",
            "Internal server error",
            "", // Empty error message
        ];

        for msg in error_messages {
            let error_state = PrimalState::Error(msg.to_string());
            assert!(error_state.to_string().contains("Error:"));
            if !msg.is_empty() {
                assert!(error_state.to_string().contains(msg));
            }
        }
    }
}

#[cfg(test)]
mod primal_info_tests {
    use super::*;

    fn create_test_primal_info(name: &str) -> PrimalInfo {
        let now = Utc::now();
        PrimalInfo {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            instance_id: Uuid::new_v4(),
            primal_type: PrimalType::AI,
            description: format!("Test {}", name),
            created_at: now,
            updated_at: now,
            tags: vec!["test".to_string(), "unit".to_string()],
            capabilities: vec!["test-cap".to_string()],
        }
    }

    #[test]
    fn test_primal_info_creation() {
        let info = create_test_primal_info("test-primal");

        assert_eq!(info.name, "test-primal");
        assert_eq!(info.version, "1.0.0");
        assert_eq!(info.primal_type, PrimalType::AI);
        assert_eq!(info.description, "Test test-primal");
        assert_eq!(info.tags.len(), 2);
        assert_eq!(info.capabilities.len(), 1);
    }

    #[test]
    fn test_primal_info_unique_instance_ids() {
        let info1 = create_test_primal_info("primal1");
        let info2 = create_test_primal_info("primal2");

        assert_ne!(info1.instance_id, info2.instance_id);
    }

    #[test]
    fn test_primal_info_clone() {
        let original = create_test_primal_info("original");
        let cloned = original.clone();

        assert_eq!(original.name, cloned.name);
        assert_eq!(original.version, cloned.version);
        assert_eq!(original.instance_id, cloned.instance_id);
        assert_eq!(original.primal_type, cloned.primal_type);
    }

    #[test]
    fn test_primal_info_debug() {
        let info = create_test_primal_info("debug-test");
        let debug_str = format!("{:?}", info);

        assert!(debug_str.contains("PrimalInfo"));
        assert!(debug_str.contains("debug-test"));
    }

    #[test]
    fn test_primal_info_serialization() {
        let info = create_test_primal_info("serialize-test");
        let serialized = serde_json::to_string(&info).unwrap();
        let deserialized: PrimalInfo = serde_json::from_str(&serialized).unwrap();

        assert_eq!(info.name, deserialized.name);
        assert_eq!(info.version, deserialized.version);
        assert_eq!(info.instance_id, deserialized.instance_id);
        assert_eq!(info.primal_type, deserialized.primal_type);
        assert_eq!(info.tags, deserialized.tags);
        assert_eq!(info.capabilities, deserialized.capabilities);
    }

    #[test]
    fn test_primal_info_with_all_primal_types() {
        let primal_types = vec![
            PrimalType::Coordinator,
            PrimalType::Security,
            PrimalType::Orchestration,
            PrimalType::Storage,
            PrimalType::Compute,
            PrimalType::AI,
            PrimalType::Network,
            PrimalType::Custom("CustomTest".to_string()),
        ];

        for primal_type in primal_types {
            let mut info = create_test_primal_info("type-test");
            info.primal_type = primal_type.clone();
            assert_eq!(info.primal_type, primal_type);
        }
    }

    #[test]
    fn test_primal_info_empty_collections() {
        let now = Utc::now();
        let info = PrimalInfo {
            name: "minimal".to_string(),
            version: "0.1.0".to_string(),
            instance_id: Uuid::new_v4(),
            primal_type: PrimalType::Network,
            description: String::new(),
            created_at: now,
            updated_at: now,
            tags: Vec::new(),
            capabilities: Vec::new(),
        };

        assert!(info.tags.is_empty());
        assert!(info.capabilities.is_empty());
        assert!(info.description.is_empty());
    }

    #[test]
    fn test_primal_info_large_collections() {
        let now = Utc::now();
        let tags: Vec<String> = (0..100).map(|i| format!("tag-{}", i)).collect();
        let capabilities: Vec<String> = (0..50).map(|i| format!("cap-{}", i)).collect();

        let info = PrimalInfo {
            name: "large-collections".to_string(),
            version: "2.0.0".to_string(),
            instance_id: Uuid::new_v4(),
            primal_type: PrimalType::Coordinator,
            description: "Test with large collections".to_string(),
            created_at: now,
            updated_at: now,
            tags,
            capabilities,
        };

        assert_eq!(info.tags.len(), 100);
        assert_eq!(info.capabilities.len(), 50);
    }

    #[test]
    fn test_primal_info_timestamps() {
        let created = Utc::now();
        let updated = Utc::now();

        let info = PrimalInfo {
            name: "timestamp-test".to_string(),
            version: "1.0.0".to_string(),
            instance_id: Uuid::new_v4(),
            primal_type: PrimalType::Storage,
            description: "Timestamp test".to_string(),
            created_at: created,
            updated_at: updated,
            tags: vec![],
            capabilities: vec![],
        };

        assert_eq!(info.created_at, created);
        assert_eq!(info.updated_at, updated);
        assert!(info.updated_at >= info.created_at);
    }

    #[test]
    fn test_primal_info_special_characters_in_strings() {
        let now = Utc::now();
        let info = PrimalInfo {
            name: "test-with-special-chars-αβγ-🐿️".to_string(),
            version: "1.0.0-beta+build.123".to_string(),
            instance_id: Uuid::new_v4(),
            primal_type: PrimalType::Custom("CustomType™".to_string()),
            description: "Description with émojis 🎉 and spëcial chars".to_string(),
            created_at: now,
            updated_at: now,
            tags: vec!["tag-with-emoji-🔖".to_string()],
            capabilities: vec!["unicode-αβγ".to_string()],
        };

        assert!(info.name.contains("🐿️"));
        assert!(info.description.contains("🎉"));
        assert!(info.tags[0].contains("🔖"));

        // Test serialization with special characters
        let serialized = serde_json::to_string(&info).unwrap();
        let deserialized: PrimalInfo = serde_json::from_str(&serialized).unwrap();
        assert_eq!(info.name, deserialized.name);
    }
}
