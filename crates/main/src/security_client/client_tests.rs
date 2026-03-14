// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Tests for Universal Security Client

#[cfg(test)]
mod tests {
    use super::super::client::UniversalSecurityClient;
    use super::super::types::{RiskLevel, SecurityClientConfig};
    use crate::security::types::{SecurityContext, SecurityRequest, SecurityRequestType};
    use crate::universal::PrimalContext;
    use crate::universal_primal_ecosystem::UniversalPrimalEcosystem;
    use std::collections::HashMap;
    use std::sync::Arc;

    fn test_context() -> PrimalContext {
        PrimalContext::default()
    }

    #[test]
    fn test_security_client_new() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let config = SecurityClientConfig::default();
        let client = UniversalSecurityClient::new(ecosystem, config, test_context());
        assert!(client.get_security_config().operation_timeout.as_secs() > 0);
    }

    #[tokio::test]
    async fn test_security_client_initialize() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let config = SecurityClientConfig::default();
        let client = UniversalSecurityClient::new(ecosystem, config, test_context());
        client
            .initialize()
            .await
            .expect("initialize should succeed");
    }

    #[tokio::test]
    async fn test_security_client_authenticate_no_providers() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let config = SecurityClientConfig::default();
        let client = UniversalSecurityClient::new(ecosystem, config, test_context());
        client.initialize().await.expect("initialize");
        let mut creds = HashMap::new();
        creds.insert("password".to_string(), "secret".to_string());
        let result = client.authenticate("user1", creds, RiskLevel::Low).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_security_client_get_config() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let config = SecurityClientConfig::default();
        let client = UniversalSecurityClient::new(ecosystem, config.clone(), test_context());
        let retrieved = client.get_security_config();
        assert_eq!(retrieved.operation_timeout, config.operation_timeout);
    }

    #[test]
    fn test_security_client_update_config() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let config = SecurityClientConfig::default();
        let mut client = UniversalSecurityClient::new(ecosystem, config, test_context());
        let new_config = SecurityClientConfig::default();
        client
            .update_security_config(new_config.clone())
            .expect("update");
        assert_eq!(
            client.get_security_config().max_retries,
            new_config.max_retries
        );
    }

    #[test]
    fn test_apply_ai_security_routing() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let config = SecurityClientConfig::default();
        let client = UniversalSecurityClient::new(ecosystem, config, test_context());
        let mut request = SecurityRequest {
            request_id: "test-1".to_string(),
            request_type: SecurityRequestType::Authentication,
            payload: serde_json::json!({}),
            metadata: HashMap::new(),
            context: SecurityContext::default(),
            timestamp: chrono::Utc::now(),
        };
        client
            .apply_ai_security_routing(&mut request)
            .expect("apply routing");
    }

    #[test]
    fn test_get_ai_security_insights() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let config = SecurityClientConfig::default();
        let client = UniversalSecurityClient::new(ecosystem, config, test_context());
        let insights = client.get_ai_security_insights();
        assert!(insights.get("threat_landscape").is_some());
        assert!(insights.get("recommended_capabilities").is_some());
        assert!(insights.get("ai_confidence").is_some());
    }

    #[test]
    fn test_validate_ai_config_compatibility() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let config = SecurityClientConfig::default();
        let client = UniversalSecurityClient::new(ecosystem, config, test_context());
        let result = client.validate_ai_config_compatibility().expect("validate");
        assert!(result);
    }

    #[test]
    fn test_get_config_based_recommendations() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let config = SecurityClientConfig::default();
        let client = UniversalSecurityClient::new(ecosystem, config, test_context());
        let recommendations = client.get_config_based_recommendations();
        assert!(!recommendations.is_empty());
        assert!(recommendations[0].get("category").is_some());
    }

    #[test]
    fn test_update_ai_metadata() {
        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(test_context()));
        let config = SecurityClientConfig::default();
        let mut client = UniversalSecurityClient::new(ecosystem, config, test_context());
        let patterns = vec![
            serde_json::json!({"pattern_type": "auth", "threat_score": 0.2, "provider_used": "bear"}),
        ];
        client
            .update_ai_metadata(patterns)
            .expect("update metadata");
    }
}
