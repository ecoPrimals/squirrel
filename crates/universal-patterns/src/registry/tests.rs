//! Tests for the universal primal registry

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::traits::*;
    use std::sync::Arc;

    // Mock PrimalProvider for testing
    struct MockPrimalProvider {
        id: String,
        instance_id: String,
        primal_type: PrimalType,
        capabilities: Vec<PrimalCapability>,
        health: PrimalHealth,
        context: PrimalContext,
    }

    impl MockPrimalProvider {
        fn new(
            id: &str,
            instance_id: &str,
            primal_type: PrimalType,
            capabilities: Vec<PrimalCapability>,
        ) -> Self {
            Self {
                id: id.to_string(),
                instance_id: instance_id.to_string(),
                primal_type,
                capabilities,
                health: PrimalHealth::Healthy,
                context: PrimalContext::default(),
            }
        }

        fn with_context(mut self, context: PrimalContext) -> Self {
            self.context = context;
            self
        }

        fn with_health(mut self, health: PrimalHealth) -> Self {
            self.health = health;
            self
        }
    }

    #[async_trait::async_trait]
    impl PrimalProvider for MockPrimalProvider {
        fn primal_id(&self) -> &str {
            &self.id
        }

        fn instance_id(&self) -> &str {
            &self.instance_id
        }

        fn context(&self) -> &PrimalContext {
            &self.context
        }

        fn primal_type(&self) -> PrimalType {
            self.primal_type.clone()
        }

        fn capabilities(&self) -> Vec<PrimalCapability> {
            self.capabilities.clone()
        }

        fn dependencies(&self) -> Vec<PrimalDependency> {
            vec![]
        }

        async fn health_check(&self) -> PrimalHealth {
            self.health.clone()
        }

        fn endpoints(&self) -> PrimalEndpoints {
            PrimalEndpoints {
                primary: format!("http://localhost:8080/{}", self.instance_id),
                health: format!("http://localhost:8080/{}/health", self.instance_id),
                metrics: None,
                admin: None,
                websocket: None,
                custom: String::new(),
            }
        }

        async fn handle_primal_request(
            &self,
            request: PrimalRequest,
        ) -> PrimalResult<PrimalResponse> {
            Ok(PrimalResponse {
                request_id: request.id,
                response_type: PrimalResponseType::Inference,
                payload: std::collections::HashMap::new(),
                timestamp: chrono::Utc::now(),
                success: true,
                error_message: None,
                metadata: Some(std::collections::HashMap::new()),
            })
        }

        async fn initialize(&mut self, _config: serde_json::Value) -> PrimalResult<()> {
            Ok(())
        }

        async fn shutdown(&mut self) -> PrimalResult<()> {
            Ok(())
        }

        fn can_serve_context(&self, context: &PrimalContext) -> bool {
            self.context.user_id == context.user_id || self.context.user_id.is_empty()
        }

        fn dynamic_port_info(&self) -> Option<DynamicPortInfo> {
            None
        }
    }

    #[tokio::test]
    async fn test_registry_new() {
        let registry = UniversalPrimalRegistry::new();
        let stats = registry.get_statistics().await;

        assert_eq!(stats.total_primals, 0);
        assert_eq!(stats.total_capabilities, 0);
    }

    #[tokio::test]
    async fn test_register_primal_for_context() {
        let registry = UniversalPrimalRegistry::new();

        let context = PrimalContext {
            user_id: "user1".to_string(),
            device_id: "device1".to_string(),
            ..Default::default()
        };

        let primal = Arc::new(
            MockPrimalProvider::new(
                "primal1",
                "instance1",
                PrimalType::AI,
                vec![PrimalCapability::ModelInference {
                    models: vec!["test-model".to_string()],
                }],
            )
            .with_context(context.clone()),
        );

        let result = registry
            .register_primal_for_context(primal, context, None)
            .await;

        assert!(result.is_ok());

        let stats = registry.get_statistics().await;
        assert_eq!(stats.total_primals, 1);
    }

    #[tokio::test]
    async fn test_register_duplicate_primal_fails() {
        let registry = UniversalPrimalRegistry::new();

        let context = PrimalContext::default();
        let primal = Arc::new(MockPrimalProvider::new(
            "primal1",
            "instance1",
            PrimalType::AI,
            vec![],
        ));

        // First registration should succeed
        let result1 = registry
            .register_primal_for_context(primal.clone(), context.clone(), None)
            .await;
        assert!(result1.is_ok());

        // Second registration of same instance should fail
        let result2 = registry
            .register_primal_for_context(primal, context, None)
            .await;
        assert!(result2.is_err());
    }

    #[tokio::test]
    async fn test_register_unhealthy_primal_fails() {
        let registry = UniversalPrimalRegistry::new();

        let context = PrimalContext::default();
        let primal = Arc::new(
            MockPrimalProvider::new("primal1", "instance1", PrimalType::AI, vec![]).with_health(
                PrimalHealth::Unhealthy {
                    reason: "Test unhealthy".to_string(),
                },
            ),
        );

        let result = registry
            .register_primal_for_context(primal, context, None)
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_find_for_context() {
        let registry = UniversalPrimalRegistry::new();

        let context1 = PrimalContext {
            user_id: "user1".to_string(),
            device_id: "device1".to_string(),
            ..Default::default()
        };

        let context2 = PrimalContext {
            user_id: "user2".to_string(),
            device_id: "device2".to_string(),
            ..Default::default()
        };

        let primal1 = Arc::new(
            MockPrimalProvider::new("primal1", "instance1", PrimalType::AI, vec![])
                .with_context(context1.clone()),
        );

        let primal2 = Arc::new(
            MockPrimalProvider::new("primal2", "instance2", PrimalType::Security, vec![])
                .with_context(context2.clone()),
        );

        registry
            .register_primal_for_context(primal1, context1.clone(), None)
            .await
            .unwrap();
        registry
            .register_primal_for_context(primal2, context2.clone(), None)
            .await
            .unwrap();

        let found = registry.find_for_context(&context1).await;
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].instance_id(), "instance1");
    }

    #[tokio::test]
    async fn test_find_by_capability_for_context() {
        let registry = UniversalPrimalRegistry::new();

        let context = PrimalContext {
            user_id: "user1".to_string(),
            device_id: "device1".to_string(),
            ..Default::default()
        };

        let primal = Arc::new(
            MockPrimalProvider::new(
                "primal1",
                "instance1",
                PrimalType::AI,
                vec![PrimalCapability::ModelInference {
                    models: vec!["test-model".to_string()],
                }],
            )
            .with_context(context.clone()),
        );

        registry
            .register_primal_for_context(primal, context.clone(), None)
            .await
            .unwrap();

        let found = registry
            .find_by_capability_for_context(
                &PrimalCapability::ModelInference {
                    models: vec!["test-model".to_string()],
                },
                &context,
            )
            .await;

        assert_eq!(found.len(), 1);
        assert_eq!(found[0].instance_id(), "instance1");

        // Different capability should return empty
        let not_found = registry
            .find_by_capability_for_context(
                &PrimalCapability::ComputerVision {
                    models: vec!["vision-model".to_string()],
                },
                &context,
            )
            .await;
        assert_eq!(not_found.len(), 0);
    }

    #[tokio::test]
    async fn test_get_instances_by_type() {
        let registry = UniversalPrimalRegistry::new();

        let context = PrimalContext::default();

        let primal1 = Arc::new(MockPrimalProvider::new(
            "primal1",
            "instance1",
            PrimalType::AI,
            vec![],
        ));

        let primal2 = Arc::new(MockPrimalProvider::new(
            "primal2",
            "instance2",
            PrimalType::AI,
            vec![],
        ));

        let primal3 = Arc::new(MockPrimalProvider::new(
            "primal3",
            "instance3",
            PrimalType::Security,
            vec![],
        ));

        registry
            .register_primal_for_context(primal1, context.clone(), None)
            .await
            .unwrap();
        registry
            .register_primal_for_context(primal2, context.clone(), None)
            .await
            .unwrap();
        registry
            .register_primal_for_context(primal3, context.clone(), None)
            .await
            .unwrap();

        let ai_instances = registry.get_instances_by_type(PrimalType::AI).await;
        assert_eq!(ai_instances.len(), 2);

        let security_instances = registry.get_instances_by_type(PrimalType::Security).await;
        assert_eq!(security_instances.len(), 1);
    }

    #[tokio::test]
    async fn test_get_instances_for_user() {
        let registry = UniversalPrimalRegistry::new();

        let context1 = PrimalContext {
            user_id: "user1".to_string(),
            device_id: "device1".to_string(),
            ..Default::default()
        };

        let context2 = PrimalContext {
            user_id: "user2".to_string(),
            device_id: "device2".to_string(),
            ..Default::default()
        };

        let primal1 = Arc::new(
            MockPrimalProvider::new("primal1", "instance1", PrimalType::AI, vec![])
                .with_context(context1.clone()),
        );

        let primal2 = Arc::new(
            MockPrimalProvider::new("primal2", "instance2", PrimalType::AI, vec![])
                .with_context(context1.clone()),
        );

        let primal3 = Arc::new(
            MockPrimalProvider::new("primal3", "instance3", PrimalType::Security, vec![])
                .with_context(context2.clone()),
        );

        registry
            .register_primal_for_context(primal1, context1.clone(), None)
            .await
            .unwrap();
        registry
            .register_primal_for_context(primal2, context1.clone(), None)
            .await
            .unwrap();
        registry
            .register_primal_for_context(primal3, context2.clone(), None)
            .await
            .unwrap();

        let user1_instances = registry.get_instances_for_user("user1").await;
        assert_eq!(user1_instances.len(), 2);

        let user2_instances = registry.get_instances_for_user("user2").await;
        assert_eq!(user2_instances.len(), 1);
    }

    #[tokio::test]
    async fn test_unregister_instance() {
        let registry = UniversalPrimalRegistry::new();

        let context = PrimalContext::default();
        let primal = Arc::new(MockPrimalProvider::new(
            "primal1",
            "instance1",
            PrimalType::AI,
            vec![],
        ));

        registry
            .register_primal_for_context(primal, context, None)
            .await
            .unwrap();

        let stats_before = registry.get_statistics().await;
        assert_eq!(stats_before.total_primals, 1);

        registry.unregister_instance("instance1").await.unwrap();

        let stats_after = registry.get_statistics().await;
        assert_eq!(stats_after.total_primals, 0);
    }

    #[tokio::test]
    async fn test_unregister_nonexistent_instance() {
        let registry = UniversalPrimalRegistry::new();

        let result = registry.unregister_instance("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_port_info_management() {
        let registry = UniversalPrimalRegistry::new();

        let context = PrimalContext::default();
        let port_info = DynamicPortInfo {
            assigned_port: 8080,
            port_type: PortType::Http,
            status: PortStatus::Active,
            assigned_at: chrono::Utc::now(),
            lease_duration: chrono::Duration::hours(1),
        };

        let primal = Arc::new(MockPrimalProvider::new(
            "primal1",
            "instance1",
            PrimalType::AI,
            vec![],
        ));

        registry
            .register_primal_for_context(primal, context, Some(port_info.clone()))
            .await
            .unwrap();

        let retrieved = registry.get_port_info("instance1").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().assigned_port, 8080);
    }

    #[tokio::test]
    async fn test_update_port_info() {
        let registry = UniversalPrimalRegistry::new();

        let context = PrimalContext::default();
        let initial_port = DynamicPortInfo {
            assigned_port: 8080,
            port_type: PortType::Http,
            status: PortStatus::Active,
            assigned_at: chrono::Utc::now(),
            lease_duration: chrono::Duration::hours(1),
        };

        let primal = Arc::new(MockPrimalProvider::new(
            "primal1",
            "instance1",
            PrimalType::AI,
            vec![],
        ));

        registry
            .register_primal_for_context(primal, context, Some(initial_port))
            .await
            .unwrap();

        let new_port = DynamicPortInfo {
            assigned_port: 9090,
            port_type: PortType::Http,
            status: PortStatus::Active,
            assigned_at: chrono::Utc::now(),
            lease_duration: chrono::Duration::hours(2),
        };

        registry
            .update_port_info("instance1", new_port)
            .await
            .unwrap();

        let updated = registry.get_port_info("instance1").await;
        assert_eq!(updated.unwrap().assigned_port, 9090);
    }

    #[tokio::test]
    async fn test_get_enhanced_statistics() {
        let registry = UniversalPrimalRegistry::new();

        let context1 = PrimalContext {
            user_id: "user1".to_string(),
            device_id: "device1".to_string(),
            ..Default::default()
        };

        let context2 = PrimalContext {
            user_id: "user2".to_string(),
            device_id: "device2".to_string(),
            ..Default::default()
        };

        let port_info = DynamicPortInfo {
            assigned_port: 8080,
            port_type: PortType::Http,
            status: PortStatus::Active,
            assigned_at: chrono::Utc::now(),
            lease_duration: chrono::Duration::hours(1),
        };

        let primal1 = Arc::new(
            MockPrimalProvider::new("primal1", "instance1", PrimalType::AI, vec![])
                .with_context(context1.clone()),
        );

        let primal2 = Arc::new(
            MockPrimalProvider::new("primal2", "instance2", PrimalType::Security, vec![])
                .with_context(context2.clone()),
        );

        registry
            .register_primal_for_context(primal1, context1, Some(port_info.clone()))
            .await
            .unwrap();
        registry
            .register_primal_for_context(primal2, context2, None)
            .await
            .unwrap();

        let stats = registry.get_enhanced_statistics().await;

        assert_eq!(stats.total_instances, 2);
        assert_eq!(stats.total_users, 2);
        assert_eq!(stats.total_ports_managed, 1);
        assert_eq!(*stats.instances_by_type.get(&PrimalType::AI).unwrap(), 1);
        assert_eq!(
            *stats.instances_by_type.get(&PrimalType::Security).unwrap(),
            1
        );
    }

    #[tokio::test]
    async fn test_health_check_all() {
        let registry = UniversalPrimalRegistry::new();

        let context = PrimalContext::default();

        let healthy_primal = Arc::new(
            MockPrimalProvider::new("primal1", "instance1", PrimalType::AI, vec![])
                .with_health(PrimalHealth::Healthy),
        );

        let degraded_primal = Arc::new(
            MockPrimalProvider::new("primal2", "instance2", PrimalType::Security, vec![])
                .with_health(PrimalHealth::Degraded {
                    issues: vec!["Slow response".to_string()],
                }),
        );

        registry
            .register_primal_for_context(healthy_primal, context.clone(), None)
            .await
            .unwrap();
        registry
            .register_primal_for_context(degraded_primal, context.clone(), None)
            .await
            .unwrap();

        let health_checks = registry.health_check_all().await;

        assert_eq!(health_checks.len(), 2);

        // Check that we have one healthy and one degraded
        let healthy_count = health_checks
            .iter()
            .filter(|(_, h)| matches!(h, PrimalHealth::Healthy))
            .count();
        let degraded_count = health_checks
            .iter()
            .filter(|(_, h)| matches!(h, PrimalHealth::Degraded { .. }))
            .count();

        assert_eq!(healthy_count, 1);
        assert_eq!(degraded_count, 1);
    }

    #[tokio::test]
    async fn test_get_statistics() {
        let registry = UniversalPrimalRegistry::new();

        let context = PrimalContext::default();

        let primal1 = Arc::new(MockPrimalProvider::new(
            "primal1",
            "instance1",
            PrimalType::AI,
            vec![
                PrimalCapability::ModelInference {
                    models: vec!["test-model".to_string()],
                },
                PrimalCapability::ComputerVision {
                    models: vec!["vision-model".to_string()],
                },
            ],
        ));

        let primal2 = Arc::new(MockPrimalProvider::new(
            "primal2",
            "instance2",
            PrimalType::Security,
            vec![PrimalCapability::Authentication {
                methods: vec!["oauth2".to_string()],
            }],
        ));

        registry
            .register_primal_for_context(primal1, context.clone(), None)
            .await
            .unwrap();
        registry
            .register_primal_for_context(primal2, context.clone(), None)
            .await
            .unwrap();

        let stats = registry.get_statistics().await;

        assert_eq!(stats.total_primals, 2);
        assert_eq!(*stats.primal_counts.get(&PrimalType::AI).unwrap(), 1);
        assert_eq!(*stats.primal_counts.get(&PrimalType::Security).unwrap(), 1);
        // Should have 3 unique capabilities indexed
        assert!(stats.total_capabilities >= 3);
    }

    #[tokio::test]
    async fn test_route_request_to_instance() {
        let registry = UniversalPrimalRegistry::new();

        let context = PrimalContext::default();
        let primal = Arc::new(MockPrimalProvider::new(
            "primal1",
            "instance1",
            PrimalType::AI,
            vec![],
        ));

        registry
            .register_primal_for_context(primal, context, None)
            .await
            .unwrap();

        let request = PrimalRequest {
            id: uuid::Uuid::new_v4(),
            request_type: PrimalRequestType::Infer,
            payload: std::collections::HashMap::new(),
            timestamp: chrono::Utc::now(),
            context: None,
            priority: None,
            security_level: None,
        };

        let result = registry
            .route_request_to_instance(request, "instance1")
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.success);
    }

    #[tokio::test]
    async fn test_route_request_to_nonexistent_instance() {
        let registry = UniversalPrimalRegistry::new();

        let request = PrimalRequest {
            id: uuid::Uuid::new_v4(),
            request_type: PrimalRequestType::Infer,
            payload: std::collections::HashMap::new(),
            timestamp: chrono::Utc::now(),
            context: None,
            priority: None,
            security_level: None,
        };

        let result = registry
            .route_request_to_instance(request, "nonexistent")
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_auto_discover() {
        let mut registry = UniversalPrimalRegistry::new();

        // Auto-discovery should complete without error
        let result = registry.auto_discover().await;
        assert!(result.is_ok());

        // Currently returns empty vec (handled by songbird)
        let discovered = result.unwrap();
        assert_eq!(discovered.len(), 0);
    }

    #[tokio::test]
    async fn test_initialize_with_config() {
        let mut registry = UniversalPrimalRegistry::new();

        let config = crate::config::UniversalPrimalConfig::default();

        let result = registry.initialize_with_config(&config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_multiple_instances_same_type() {
        let registry = UniversalPrimalRegistry::new();

        let context = PrimalContext::default();

        for i in 1..=5 {
            let primal = Arc::new(MockPrimalProvider::new(
                &format!("primal{}", i),
                &format!("instance{}", i),
                PrimalType::AI,
                vec![],
            ));

            registry
                .register_primal_for_context(primal, context.clone(), None)
                .await
                .unwrap();
        }

        let instances = registry.get_instances_by_type(PrimalType::AI).await;
        assert_eq!(instances.len(), 5);

        let stats = registry.get_statistics().await;
        assert_eq!(stats.total_primals, 5);
    }

    #[tokio::test]
    async fn test_context_isolation() {
        let registry = UniversalPrimalRegistry::new();

        let context1 = PrimalContext {
            user_id: "user1".to_string(),
            device_id: "device1".to_string(),
            ..Default::default()
        };

        let context2 = PrimalContext {
            user_id: "user2".to_string(),
            device_id: "device2".to_string(),
            ..Default::default()
        };

        let primal1 = Arc::new(
            MockPrimalProvider::new("primal1", "instance1", PrimalType::AI, vec![])
                .with_context(context1.clone()),
        );

        let primal2 = Arc::new(
            MockPrimalProvider::new("primal2", "instance2", PrimalType::AI, vec![])
                .with_context(context2.clone()),
        );

        registry
            .register_primal_for_context(primal1, context1.clone(), None)
            .await
            .unwrap();
        registry
            .register_primal_for_context(primal2, context2.clone(), None)
            .await
            .unwrap();

        // Each context should only see their own primal
        let user1_primals = registry.find_for_context(&context1).await;
        assert_eq!(user1_primals.len(), 1);
        assert_eq!(user1_primals[0].instance_id(), "instance1");

        let user2_primals = registry.find_for_context(&context2).await;
        assert_eq!(user2_primals.len(), 1);
        assert_eq!(user2_primals[0].instance_id(), "instance2");
    }
}
