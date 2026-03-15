// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Tests for Universal Storage Client

#[cfg(test)]
mod tests {
    use super::super::client::UniversalStorageClient;
    use super::super::types::{DataClassification, StorageClientConfig};
    use crate::universal::PrimalContext;
    use crate::universal_primal_ecosystem::UniversalPrimalEcosystem;
    use std::sync::Arc;

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
        assert!(optimizations
            .iter()
            .any(|v| v.as_str() == Some("chunking_enabled")));
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
            .update_ai_storage_metadata(patterns)
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
}
