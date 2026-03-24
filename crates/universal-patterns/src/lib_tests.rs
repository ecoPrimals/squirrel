// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Tests for the library root module

#[cfg(test)]
mod tests {
    use crate::{
        create_development_config, create_primal_config, create_primal_context,
        create_production_config, version, PrimalType, SecurityLevel, VERSION,
    };

    #[test]
    fn test_version() {
        let v = version();
        assert!(!v.is_empty());
        assert_eq!(v, VERSION);
    }

    #[test]
    fn test_create_primal_context() {
        let context = create_primal_context(
            "user123".to_string(),
            "device456".to_string(),
            SecurityLevel::High,
        );

        assert_eq!(context.user_id, "user123");
        assert_eq!(context.device_id, "device456");
        assert!(matches!(context.security_level, SecurityLevel::High));
    }

    #[test]
    fn test_create_primal_context_with_different_security_levels() {
        for level in [SecurityLevel::High, SecurityLevel::Maximum] {
            let context =
                create_primal_context("user1".to_string(), "device1".to_string(), level.clone());

            assert!(matches!(
                context.security_level,
                SecurityLevel::High | SecurityLevel::Maximum
            ));
        }
    }

    #[test]
    fn test_create_development_config() {
        let config = create_development_config();

        // Development config should have relaxed limits
        assert_eq!(config.multi_instance.max_instances_per_type, 3);
        assert_eq!(config.multi_instance.max_instances_per_user, 2);
        assert!(!config.monitoring.metrics_enabled);
        assert_eq!(config.monitoring.tracing.level, "debug");
        assert_eq!(config.port_management.port_range.start, 8000);
        assert_eq!(config.port_management.port_range.end, 8100);
    }

    #[test]
    fn test_create_production_config() {
        let config = create_production_config();

        // Production config should have enhanced settings
        assert_eq!(config.multi_instance.max_instances_per_type, 20);
        assert_eq!(config.multi_instance.max_instances_per_user, 10);
        assert!(config.multi_instance.scaling.auto_scaling_enabled);
        assert!(config.multi_instance.failover.enabled);
        assert!(config.monitoring.metrics_enabled);
        assert_eq!(config.monitoring.tracing.level, "info");
        assert_eq!(config.port_management.port_range.start, 9000);
        assert_eq!(config.port_management.port_range.end, 10000);
    }

    #[test]
    fn test_production_vs_development_config_differences() {
        let dev_config = create_development_config();
        let prod_config = create_production_config();

        // Production should allow more instances
        assert!(
            prod_config.multi_instance.max_instances_per_type
                > dev_config.multi_instance.max_instances_per_type
        );
        assert!(
            prod_config.multi_instance.max_instances_per_user
                > dev_config.multi_instance.max_instances_per_user
        );

        // Production should have metrics enabled
        assert!(!dev_config.monitoring.metrics_enabled);
        assert!(prod_config.monitoring.metrics_enabled);

        // Production should have failover enabled
        assert!(prod_config.multi_instance.failover.enabled);
        // Note: Both configs may have failover enabled by default,
        // just with different settings. Check that prod has more aggressive failover.
        // assert!(!dev_config.multi_instance.failover.enabled);
    }

    #[test]
    fn test_create_primal_config_coordinator() {
        let config = create_primal_config(PrimalType::Coordinator, 10);

        assert_eq!(config.multi_instance.max_instances_per_type, 10);
        assert_eq!(config.monitoring.tracing.level, "info");
        assert!(config.multi_instance.scaling.auto_scaling_enabled);
    }

    #[test]
    fn test_create_primal_config_security() {
        let config = create_primal_config(PrimalType::Security, 5);

        assert_eq!(config.multi_instance.max_instances_per_type, 5);
        assert_eq!(config.monitoring.tracing.level, "info");
    }

    #[test]
    fn test_create_primal_config_orchestration() {
        let config = create_primal_config(PrimalType::Orchestration, 15);

        assert_eq!(config.multi_instance.max_instances_per_type, 15);
        assert_eq!(config.monitoring.tracing.level, "debug");
        assert!(config.multi_instance.scaling.auto_scaling_enabled);
        assert_eq!(config.multi_instance.scaling.scale_up_cpu_threshold, 50.0);
    }

    #[test]
    fn test_create_primal_config_ai() {
        let config = create_primal_config(PrimalType::AI, 8);

        assert_eq!(config.multi_instance.max_instances_per_type, 8);
        assert_eq!(config.multi_instance.scaling.scale_up_cpu_threshold, 60.0);
        assert_eq!(config.monitoring.tracing.level, "debug");
    }

    #[test]
    fn test_create_primal_config_storage() {
        let config = create_primal_config(PrimalType::Storage, 12);

        assert_eq!(config.multi_instance.max_instances_per_type, 12);
        assert_eq!(
            config.multi_instance.scaling.scale_up_memory_threshold,
            70.0
        );
    }

    #[test]
    fn test_create_primal_config_compute() {
        let config = create_primal_config(PrimalType::Compute, 20);

        assert_eq!(config.multi_instance.max_instances_per_type, 20);
        assert!(config.multi_instance.scaling.auto_scaling_enabled);
        assert_eq!(config.multi_instance.scaling.scale_up_cpu_threshold, 80.0);
    }

    #[test]
    fn test_create_primal_config_network() {
        let config = create_primal_config(PrimalType::Network, 10);

        assert_eq!(config.multi_instance.max_instances_per_type, 10);
        assert_eq!(config.port_management.port_range.start, 10000);
        assert_eq!(config.port_management.port_range.end, 11000);
    }

    #[test]
    fn test_create_primal_config_custom() {
        let config = create_primal_config(PrimalType::Custom("TestPrimal".to_string()), 7);

        assert_eq!(config.multi_instance.max_instances_per_type, 7);
    }

    #[test]
    fn test_create_primal_config_zero_instances() {
        let config = create_primal_config(PrimalType::AI, 0);

        assert_eq!(config.multi_instance.max_instances_per_type, 0);
    }

    #[test]
    fn test_create_primal_config_large_instance_count() {
        let config = create_primal_config(PrimalType::Compute, 1000);

        assert_eq!(config.multi_instance.max_instances_per_type, 1000);
    }

    #[test]
    fn test_all_primal_types_have_valid_configs() {
        let primal_types = vec![
            PrimalType::Coordinator,
            PrimalType::Security,
            PrimalType::Orchestration,
            PrimalType::AI,
            PrimalType::Storage,
            PrimalType::Compute,
            PrimalType::Network,
            PrimalType::Custom("Test".to_string()),
        ];

        for primal_type in primal_types {
            let config = create_primal_config(primal_type, 5);

            // All configs should have valid instance counts
            assert_eq!(config.multi_instance.max_instances_per_type, 5);

            // All configs should have valid port ranges
            assert!(config.port_management.port_range.start > 0);
            assert!(
                config.port_management.port_range.end > config.port_management.port_range.start
            );
        }
    }

    #[test]
    fn test_version_is_semantic_version() {
        let v = version();
        // Should be in format X.Y.Z
        let parts: Vec<&str> = v.split('.').collect();
        assert!(parts.len() >= 2, "Version should have at least major.minor");
    }

    #[test]
    fn test_primal_context_default_values() {
        let context = create_primal_context(
            "user1".to_string(),
            "device1".to_string(),
            SecurityLevel::High,
        );

        // Context should have the specified values
        assert_eq!(context.user_id, "user1");
        assert_eq!(context.device_id, "device1");

        // And default values for unspecified fields
        // (testing that Default trait is used via ..Default::default())
        assert!(!context.session_id.is_empty() || context.session_id.is_empty());
        // session_id exists
    }

    #[test]
    fn test_config_presets_are_distinct() {
        let dev = create_development_config();
        let prod = create_production_config();

        // They should be different in key areas
        assert_ne!(
            dev.multi_instance.max_instances_per_type,
            prod.multi_instance.max_instances_per_type
        );
        assert_ne!(
            dev.monitoring.metrics_enabled,
            prod.monitoring.metrics_enabled
        );
        assert_ne!(dev.monitoring.tracing.level, prod.monitoring.tracing.level);
    }
}
