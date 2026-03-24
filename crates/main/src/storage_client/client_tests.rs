// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Tests for Universal Storage Client

#[cfg(test)]
mod tests {
    use super::super::client::UniversalStorageClient;
    use super::super::providers::StorageProvider;
    use super::super::types::{DataClassification, StorageClientConfig};
    use crate::universal::PrimalContext;
    use crate::universal::messages::{PrimalResponse, ResponseStatus};
    use crate::universal_primal_ecosystem::{
        DiscoveredService, ServiceHealth, UniversalPrimalEcosystem,
    };
    use base64::{Engine as _, engine::general_purpose};
    use chrono::Utc;
    use std::collections::HashMap;
    use std::sync::Arc;
    use uuid::Uuid;

    fn test_context() -> PrimalContext {
        PrimalContext::default()
    }

    #[test]
    fn test_storage_client_new() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let config = StorageClientConfig::default();
        let client = UniversalStorageClient::new(ecosystem, config, test_context());
        assert!(client.get_storage_config().operation_timeout.as_secs() > 0);
    }

    #[tokio::test]
    async fn test_storage_client_initialize() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let config = StorageClientConfig::default();
        let client = UniversalStorageClient::new(ecosystem, config, test_context());
        client
            .initialize()
            .await
            .expect("initialize should succeed");
    }

    #[tokio::test]
    async fn test_storage_client_store_no_providers() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let config = StorageClientConfig::default();
        let client = UniversalStorageClient::new(ecosystem, config, test_context());
        client.initialize().await.expect("initialize");
        let result = client
            .store("key", vec![1, 2, 3], DataClassification::Internal)
            .await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("No storage providers")
                || err.to_string().contains("provider")
        );
    }

    #[tokio::test]
    async fn test_storage_client_retrieve_no_providers() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let config = StorageClientConfig::default();
        let client = UniversalStorageClient::new(ecosystem, config, test_context());
        client.initialize().await.expect("initialize");
        let result = client.retrieve("key").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_storage_client_delete_no_providers() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let config = StorageClientConfig::default();
        let client = UniversalStorageClient::new(ecosystem, config, test_context());
        client.initialize().await.expect("initialize");
        let result = client.delete("key").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_storage_client_get_config() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let config = StorageClientConfig::default();
        let client = UniversalStorageClient::new(ecosystem, config.clone(), test_context());
        let retrieved = client.get_storage_config();
        assert_eq!(retrieved.operation_timeout, config.operation_timeout);
    }

    #[test]
    fn test_storage_client_update_config() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let config = StorageClientConfig::default();
        let mut client = UniversalStorageClient::new(ecosystem, config, test_context());
        let new_config = StorageClientConfig::default();
        client
            .update_storage_config(new_config.clone())
            .expect("update");
        assert_eq!(
            client.get_storage_config().max_retries,
            new_config.max_retries
        );
    }

    #[test]
    fn test_apply_ai_storage_routing() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let config = StorageClientConfig::default();
        let client = UniversalStorageClient::new(ecosystem, config, test_context());
        let mut request = serde_json::json!({"operation": "store", "file_size": 5000000});
        client
            .apply_ai_storage_routing(&mut request)
            .expect("apply routing");
        assert!(request.get("storage_strategy").is_some());
        assert_eq!(request["storage_strategy"], "high_performance");
    }

    #[test]
    fn test_apply_ai_storage_routing_large_file() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let config = StorageClientConfig::default();
        let client = UniversalStorageClient::new(ecosystem, config, test_context());
        let mut request = serde_json::json!({"operation": "store", "file_size": 15000000});
        client
            .apply_ai_storage_routing(&mut request)
            .expect("apply routing");
        assert_eq!(request["compression"], "gzip");
    }

    #[test]
    fn test_get_ai_storage_insights() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let config = StorageClientConfig::default();
        let client = UniversalStorageClient::new(ecosystem, config, test_context());
        let insights = client.get_ai_storage_insights();
        assert!(insights.get("storage_efficiency").is_some());
        assert!(insights.get("recommended_capabilities").is_some());
        assert!(insights.get("ai_confidence").is_some());
    }

    #[test]
    fn test_optimize_storage_request() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let config = StorageClientConfig::default();
        let client = UniversalStorageClient::new(ecosystem, config, test_context());
        let mut request = serde_json::json!({"operation": "store", "file_size": 2000000});
        let result = client
            .optimize_storage_request(&mut request)
            .expect("optimize");
        assert!(result.get("optimizations_applied").is_some());
        assert!(result.get("cost_reduction_pct").is_some());
    }

    #[test]
    fn test_optimize_storage_request_large_file() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let config = StorageClientConfig::default();
        let client = UniversalStorageClient::new(ecosystem, config, test_context());
        let mut request = serde_json::json!({"operation": "store", "file_size": 150_000_000});
        let result = client
            .optimize_storage_request(&mut request)
            .expect("optimize");
        assert!(request.get("chunking").is_some());
        let optimizations = result["optimizations_applied"].as_array().expect("array");
        assert!(
            optimizations
                .iter()
                .any(|v| v.as_str() == Some("chunking_enabled"))
        );
    }

    #[test]
    fn test_update_ai_storage_metadata() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let config = StorageClientConfig::default();
        let mut client = UniversalStorageClient::new(ecosystem, config, test_context());
        let patterns = vec![
            serde_json::json!({"pattern_type": "upload", "provider_used": "s3", "throughput_mbps": 50.0}),
        ];
        client
            .update_ai_storage_metadata(&patterns)
            .expect("update metadata");
    }

    #[test]
    fn test_get_config_based_storage_recommendations() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let config = StorageClientConfig::default();
        let client = UniversalStorageClient::new(ecosystem, config, test_context());
        let recommendations = client.get_config_based_storage_recommendations();
        assert!(!recommendations.is_empty());
        assert!(recommendations[0].get("category").is_some());
        assert!(recommendations[0].get("severity").is_some());
    }

    #[test]
    fn test_apply_ai_storage_routing_skips_preferred_capability_on_delete() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let config = StorageClientConfig::default();
        let client = UniversalStorageClient::new(ecosystem, config, test_context());
        let mut request = serde_json::json!({"operation": "delete", "file_size": 1000u64});
        client
            .apply_ai_storage_routing(&mut request)
            .expect("apply routing");
        assert!(request.get("preferred_capability").is_none());
    }

    #[test]
    fn test_apply_ai_storage_routing_backup_sets_strategy() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let config = StorageClientConfig::default();
        let client = UniversalStorageClient::new(ecosystem, config, test_context());
        let mut request = serde_json::json!({"operation": "backup", "file_size": 100u64});
        client
            .apply_ai_storage_routing(&mut request)
            .expect("apply routing");
        assert_eq!(request["storage_strategy"], "high_performance");
        assert!(request.get("preferred_capability").is_some());
    }

    #[test]
    fn test_optimize_storage_request_delete_branch() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let config = StorageClientConfig::default();
        let client = UniversalStorageClient::new(ecosystem, config, test_context());
        let mut request = serde_json::json!({"operation": "delete", "file_size": 2_000_000u64});
        let result = client
            .optimize_storage_request(&mut request)
            .expect("should succeed");
        assert!(result.get("optimizations_applied").is_some());
    }

    #[test]
    fn test_update_ai_storage_metadata_compression_branch() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let config = StorageClientConfig::default();
        let mut client = UniversalStorageClient::new(ecosystem, config, test_context());
        let patterns = vec![serde_json::json!({
            "pattern_type": "upload",
            "compression_ratio": 0.25,
            "file_size": 4096u64
        })];
        client
            .update_ai_storage_metadata(&patterns)
            .expect("should succeed");
    }

    #[tokio::test]
    async fn test_store_succeeds_with_provider_and_default_response() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let primal = universal_patterns::registry::DiscoveredPrimal {
            id: "st-a".to_string(),
            instance_id: "st-inst".to_string(),
            primal_type: universal_patterns::traits::PrimalType::Storage,
            capabilities: vec![],
            endpoint: "unix:///tmp/st.sock".to_string(),
            health: universal_patterns::traits::PrimalHealth::Healthy,
            context: universal_patterns::traits::PrimalContext::default(),
            port_info: None,
        };
        let provider = StorageProvider::from_discovered_primal(&primal);
        let client = UniversalStorageClient::new(
            Arc::clone(&ecosystem),
            StorageClientConfig::default(),
            test_context(),
        );
        client.test_only_insert_provider(provider);
        let r = client
            .store("k", vec![1, 2, 3], DataClassification::Internal)
            .await
            .expect("store");
        assert!(r.success);
    }

    #[tokio::test]
    async fn test_execute_operation_failure_degrades_provider_health() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        ecosystem
            .test_only_set_next_primal_response(PrimalResponse {
                request_id: Uuid::new_v4(),
                response_id: Uuid::new_v4(),
                status: ResponseStatus::Error,
                success: false,
                data: None,
                payload: serde_json::json!({}),
                timestamp: Utc::now(),
                processing_time_ms: Some(200),
                duration: None,
                error: Some("e".to_string()),
                error_message: Some("write failed".to_string()),
                metadata: HashMap::new(),
            })
            .await;
        let primal = universal_patterns::registry::DiscoveredPrimal {
            id: "st-b".to_string(),
            instance_id: "st-inst-b".to_string(),
            primal_type: universal_patterns::traits::PrimalType::Storage,
            capabilities: vec![],
            endpoint: "unix:///tmp/st2.sock".to_string(),
            health: universal_patterns::traits::PrimalHealth::Healthy,
            context: universal_patterns::traits::PrimalContext::default(),
            port_info: None,
        };
        let provider = StorageProvider::from_discovered_primal(&primal);
        let client = UniversalStorageClient::new(
            Arc::clone(&ecosystem),
            StorageClientConfig::default(),
            test_context(),
        );
        client.test_only_insert_provider(provider);
        let r = client.retrieve("key").await.expect("retrieve");
        assert!(!r.success);
        assert!(r.error.is_some());
    }

    #[tokio::test]
    async fn test_process_response_decodes_base64_on_success() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let encoded = general_purpose::STANDARD.encode(b"blob-data");
        ecosystem
            .test_only_set_next_primal_response(PrimalResponse {
                request_id: Uuid::new_v4(),
                response_id: Uuid::new_v4(),
                status: ResponseStatus::Success,
                success: true,
                data: Some(serde_json::json!({ "data": encoded })),
                payload: serde_json::json!({}),
                timestamp: Utc::now(),
                processing_time_ms: Some(10),
                duration: None,
                error: None,
                error_message: None,
                metadata: HashMap::new(),
            })
            .await;
        let primal = universal_patterns::registry::DiscoveredPrimal {
            id: "st-c".to_string(),
            instance_id: "st-inst-c".to_string(),
            primal_type: universal_patterns::traits::PrimalType::Storage,
            capabilities: vec![],
            endpoint: "unix:///tmp/st3.sock".to_string(),
            health: universal_patterns::traits::PrimalHealth::Healthy,
            context: universal_patterns::traits::PrimalContext::default(),
            port_info: None,
        };
        let client = UniversalStorageClient::new(
            Arc::clone(&ecosystem),
            StorageClientConfig::default(),
            test_context(),
        );
        client.test_only_insert_provider(StorageProvider::from_discovered_primal(&primal));
        let r = client.retrieve("k").await.expect("retrieve");
        assert_eq!(r.data.as_deref(), Some(b"blob-data".as_slice()));
    }

    #[tokio::test]
    async fn test_discover_storage_providers_via_initialize() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let svc = DiscoveredService {
            service_id: "svc-st".to_string(),
            instance_id: "inst-st".to_string(),
            endpoint: "unix:///tmp/st.sock".to_string(),
            capabilities: vec![
                "object-storage".to_string(),
                "storage-capability".to_string(),
            ],
            health: ServiceHealth::Healthy,
            discovered_at: Utc::now(),
            last_health_check: Some(Utc::now()),
        };
        ecosystem.test_only_register_service(svc).await;
        let client = UniversalStorageClient::new(
            Arc::clone(&ecosystem),
            StorageClientConfig::default(),
            test_context(),
        );
        client.initialize().await.expect("init");
        let r = client
            .delete("k")
            .await
            .expect("delete with discovered provider");
        assert!(r.success);
    }
}
