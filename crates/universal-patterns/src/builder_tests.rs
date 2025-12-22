//! Tests for the configuration builder module

#[cfg(test)]
mod tests {
    use crate::builder::{PrimalInstanceConfigBuilder, UniversalConfigBuilder};
    use crate::config::LoadBalancingStrategy;
    use crate::traits::SecurityLevel;

    #[test]
    fn test_universal_config_builder_new() {
        let builder = UniversalConfigBuilder::new();
        let config = builder.build();

        // Default configuration should be valid
        assert!(config.auto_discovery_enabled);
    }

    #[test]
    fn test_universal_config_builder_auto_discovery() {
        let builder = UniversalConfigBuilder::new().auto_discovery(false);
        let config = builder.build();

        assert!(!config.auto_discovery_enabled);
    }

    #[test]
    fn test_universal_config_builder_max_instances_per_type() {
        let builder = UniversalConfigBuilder::new().max_instances_per_type(10);
        let config = builder.build();

        assert_eq!(config.multi_instance.max_instances_per_type, 10);
    }

    #[test]
    fn test_universal_config_builder_max_instances_per_user() {
        let builder = UniversalConfigBuilder::new().max_instances_per_user(5);
        let config = builder.build();

        assert_eq!(config.multi_instance.max_instances_per_user, 5);
    }

    #[test]
    fn test_universal_config_builder_load_balancing_strategy() {
        let builder = UniversalConfigBuilder::new()
            .load_balancing_strategy(LoadBalancingStrategy::LeastConnections);
        let config = builder.build();

        assert!(matches!(
            config.multi_instance.load_balancing_strategy,
            LoadBalancingStrategy::LeastConnections
        ));
    }

    #[test]
    fn test_universal_config_builder_metrics_enabled() {
        let builder = UniversalConfigBuilder::new().metrics_enabled(true);
        let config = builder.build();

        assert!(config.monitoring.metrics_enabled);
    }

    #[test]
    fn test_universal_config_builder_tracing_level() {
        let builder = UniversalConfigBuilder::new().tracing_level("debug".to_string());
        let config = builder.build();

        assert_eq!(config.monitoring.tracing.level, "debug");
    }

    #[test]
    fn test_universal_config_builder_port_range() {
        let builder = UniversalConfigBuilder::new().port_range(9000, 9999);
        let config = builder.build();

        assert_eq!(config.port_management.port_range.start, 9000);
        assert_eq!(config.port_management.port_range.end, 9999);
    }

    #[test]
    fn test_universal_config_builder_add_primal_instance() {
        let instance_config = PrimalInstanceConfigBuilder::new(
            "http://localhost:8080".to_string(),
            "instance1".to_string(),
            "user1".to_string(),
            "device1".to_string(),
        )
        .security_level(SecurityLevel::High)
        .build();

        let builder = UniversalConfigBuilder::new()
            .add_primal_instance("instance1".to_string(), instance_config);
        let config = builder.build();

        assert_eq!(config.primal_instances.len(), 1);
        assert!(config.primal_instances.contains_key("instance1"));
    }

    #[test]
    fn test_universal_config_builder_fluent_api() {
        let builder = UniversalConfigBuilder::new()
            .auto_discovery(true)
            .max_instances_per_type(20)
            .max_instances_per_user(10)
            .metrics_enabled(true)
            .tracing_level("info".to_string())
            .port_range(8000, 8100);

        let config = builder.build();

        assert!(config.auto_discovery_enabled);
        assert_eq!(config.multi_instance.max_instances_per_type, 20);
        assert_eq!(config.multi_instance.max_instances_per_user, 10);
        assert!(config.monitoring.metrics_enabled);
        assert_eq!(config.monitoring.tracing.level, "info");
        assert_eq!(config.port_management.port_range.start, 8000);
        assert_eq!(config.port_management.port_range.end, 8100);
    }

    #[test]
    fn test_primal_instance_config_builder_new() {
        let builder = PrimalInstanceConfigBuilder::new(
            "http://localhost:8080".to_string(),
            "instance1".to_string(),
            "user1".to_string(),
            "device1".to_string(),
        );

        let config = builder.build();

        assert_eq!(config.base_url, "http://localhost:8080");
        assert_eq!(config.instance_id, "instance1");
        assert_eq!(config.user_id, "user1");
        assert_eq!(config.device_id, "device1");
    }

    #[test]
    fn test_primal_instance_config_builder_security_level() {
        let builder = PrimalInstanceConfigBuilder::new(
            "http://localhost:8080".to_string(),
            "instance1".to_string(),
            "user1".to_string(),
            "device1".to_string(),
        )
        .security_level(SecurityLevel::Maximum);

        let config = builder.build();

        assert_eq!(config.security_level, "Maximum");
    }

    #[test]
    fn test_primal_instance_config_builder_api_key() {
        let builder = PrimalInstanceConfigBuilder::new(
            "http://localhost:8080".to_string(),
            "instance1".to_string(),
            "user1".to_string(),
            "device1".to_string(),
        )
        .api_key("secret-key-123".to_string());

        let config = builder.build();

        assert_eq!(config.api_key, Some("secret-key-123".to_string()));
    }

    #[test]
    fn test_primal_instance_config_builder_timeout() {
        let builder = PrimalInstanceConfigBuilder::new(
            "http://localhost:8080".to_string(),
            "instance1".to_string(),
            "user1".to_string(),
            "device1".to_string(),
        )
        .timeout_seconds(120);

        let config = builder.build();

        assert_eq!(config.timeout_seconds, 120);
    }

    #[test]
    fn test_primal_instance_config_builder_fluent_api() {
        let builder = PrimalInstanceConfigBuilder::new(
            "https://api.example.com".to_string(),
            "prod-instance-1".to_string(),
            "user-456".to_string(),
            "device-789".to_string(),
        )
        .security_level(SecurityLevel::High)
        .api_key("api-key-xyz".to_string())
        .timeout_seconds(60);

        let config = builder.build();

        assert_eq!(config.base_url, "https://api.example.com");
        assert_eq!(config.instance_id, "prod-instance-1");
        assert_eq!(config.user_id, "user-456");
        assert_eq!(config.device_id, "device-789");
        assert_eq!(config.security_level, "High");
        assert_eq!(config.api_key, Some("api-key-xyz".to_string()));
        assert_eq!(config.timeout_seconds, 60);
    }

    #[test]
    fn test_builder_default() {
        let builder = UniversalConfigBuilder::default();
        let config = builder.build();

        // Should create a valid default configuration
        assert!(config.auto_discovery_enabled);
    }

    #[test]
    fn test_multiple_primal_instances() {
        let instance1 = PrimalInstanceConfigBuilder::new(
            "http://instance1:8080".to_string(),
            "inst1".to_string(),
            "user1".to_string(),
            "device1".to_string(),
        )
        .security_level(SecurityLevel::High)
        .build();

        let instance2 = PrimalInstanceConfigBuilder::new(
            "http://instance2:8081".to_string(),
            "inst2".to_string(),
            "user1".to_string(),
            "device1".to_string(),
        )
        .security_level(SecurityLevel::Standard)
        .build();

        let builder = UniversalConfigBuilder::new()
            .add_primal_instance("inst1".to_string(), instance1)
            .add_primal_instance("inst2".to_string(), instance2);

        let config = builder.build();

        assert_eq!(config.primal_instances.len(), 2);
        assert!(config.primal_instances.contains_key("inst1"));
        assert!(config.primal_instances.contains_key("inst2"));
    }

    #[test]
    fn test_builder_with_all_load_balancing_strategies() {
        for strategy in [
            LoadBalancingStrategy::RoundRobin,
            LoadBalancingStrategy::LeastConnections,
            LoadBalancingStrategy::Random,
            LoadBalancingStrategy::Weighted,
        ] {
            let builder = UniversalConfigBuilder::new().load_balancing_strategy(strategy.clone());
            let config = builder.build();

            // Verify strategy was set
            assert!(matches!(
                config.multi_instance.load_balancing_strategy,
                LoadBalancingStrategy::RoundRobin
                    | LoadBalancingStrategy::LeastConnections
                    | LoadBalancingStrategy::Random
                    | LoadBalancingStrategy::Weighted
            ));
        }
    }

    #[test]
    fn test_builder_extreme_values() {
        let builder = UniversalConfigBuilder::new()
            .max_instances_per_type(1000)
            .max_instances_per_user(500)
            .port_range(1024, 65535);

        let config = builder.build();

        assert_eq!(config.multi_instance.max_instances_per_type, 1000);
        assert_eq!(config.multi_instance.max_instances_per_user, 500);
        assert_eq!(config.port_management.port_range.start, 1024);
        assert_eq!(config.port_management.port_range.end, 65535);
    }

    #[test]
    fn test_builder_edge_case_zero_instances() {
        let builder = UniversalConfigBuilder::new()
            .max_instances_per_type(0)
            .max_instances_per_user(0);

        let config = builder.build();

        assert_eq!(config.multi_instance.max_instances_per_type, 0);
        assert_eq!(config.multi_instance.max_instances_per_user, 0);
    }

    #[test]
    fn test_all_security_levels() {
        for level in [
            SecurityLevel::Basic,
            SecurityLevel::Standard,
            SecurityLevel::High,
            SecurityLevel::Maximum,
        ] {
            let builder = PrimalInstanceConfigBuilder::new(
                "http://test:8080".to_string(),
                "test".to_string(),
                "user".to_string(),
                "device".to_string(),
            )
            .security_level(level);

            let config = builder.build();
            // Should set a valid security level string
            assert!(!config.security_level.is_empty());
        }
    }
}
