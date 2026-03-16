// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! # biome.yaml Manifest Support for biomeOS Integration
//!
//! This module provides comprehensive support for parsing and processing biome.yaml
//! manifest files, enabling agent deployment, service discovery, and resource
//! management within the biomeOS ecosystem.

mod parser;
mod types;

pub use parser::{BiomeManifestParser, ManifestParserConfig};
pub use types::*;

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::collections::HashMap;

    #[test]
    fn test_manifest_creation() {
        let metadata = BiomeMetadata {
            name: "test-biome".to_string(),
            description: "Test biome manifest".to_string(),
            version: "1.0.0".to_string(),
            biomeos_version: "0.1.0".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            author: "test-author".to_string(),
            labels: HashMap::new(),
        };

        let manifest = BiomeManifest {
            metadata,
            agents: Vec::new(),
            services: HashMap::new(),
            resources: BiomeResources {
                limits: ResourceLimits {
                    memory_gb: 8.0,
                    cpu_cores: 4.0,
                    storage_gb: 100.0,
                    network_bandwidth_mbps: 1000.0,
                },
                reservations: ResourceReservations {
                    memory_gb: 2.0,
                    cpu_cores: 1.0,
                    storage_gb: 10.0,
                },
                policies: ResourcePolicies {
                    memory_over_commit: false,
                    cpu_over_commit: true,
                    storage_over_commit: false,
                    resource_quotas: HashMap::new(),
                },
            },
            security: BiomeSecurity {
                authentication: AuthenticationConfig {
                    enabled: true,
                    method: "oauth2".to_string(),
                    providers: vec!["github".to_string()],
                },
                authorization: AuthorizationConfig {
                    enabled: true,
                    method: "rbac".to_string(),
                    policies: vec!["default".to_string()],
                },
                encryption: EncryptionConfig::default(),
                tokens: TokenConfig {
                    enabled: true,
                    expiration_seconds: 3600,
                    refresh_enabled: true,
                },
                policies: SecurityPolicies {
                    network_policies: Vec::new(),
                    pod_security_policies: Vec::new(),
                    rbac_policies: Vec::new(),
                },
            },
            storage: BiomeStorage {
                volumes: Vec::new(),
                volume_claim_templates: Vec::new(),
                storage_classes: Vec::new(),
            },
            networking: BiomeNetworking {
                ingress: IngressConfig {
                    enabled: true,
                    host: "test.example.com".to_string(),
                    tls_enabled: true,
                    annotations: HashMap::new(),
                },
                network_policies: Vec::new(),
                dns: DnsConfig {
                    enabled: true,
                    servers: vec!["8.8.8.8".to_string()],
                    search_domains: vec![],
                },
            },
            primals: HashMap::new(),
        };

        assert_eq!(manifest.metadata.name, "test-biome");
        assert_eq!(manifest.metadata.version, "1.0.0");
        assert!(manifest.security.authentication.enabled);
        assert!(manifest.networking.ingress.enabled);
    }

    #[test]
    fn test_agent_spec_creation() {
        let agent = AgentSpec {
            name: "test-agent".to_string(),
            capabilities: vec!["chat".to_string(), "search".to_string()],
            ai_provider: "openai".to_string(),
            model: "gpt-4".to_string(),
            execution_environment: ExecutionEnvironment::Container,
            resource_limits: AgentResourceLimits::default(),
            resources: AgentResourceLimits::default(),
            manifest: None,
            security: AgentSecurity::default(),
            storage: AgentStorage::default(),
            environment: HashMap::new(),
            config: HashMap::new(),
        };

        assert_eq!(agent.name, "test-agent");
        assert_eq!(agent.capabilities.len(), 2);
        assert_eq!(agent.ai_provider, "openai");
        assert_eq!(agent.execution_environment, ExecutionEnvironment::Container);
    }

    #[tokio::test]
    async fn test_manifest_parser() {
        let parser = BiomeManifestParser::new();
        assert!(parser.config.strict_validation);

        let yaml_content = r#"
metadata:
  name: test-biome
  description: Test biome
  version: 1.0.0
  biomeos_version: 0.1.0
  created_at: 2024-01-01T00:00:00Z
  updated_at: 2024-01-01T00:00:00Z
  author: test-author
  labels: {}
agents: []
services: {}
resources:
  limits:
    memory_gb: 8.0
    cpu_cores: 4.0
    storage_gb: 100.0
    network_bandwidth_mbps: 1000.0
  reservations:
    memory_gb: 2.0
    cpu_cores: 1.0
    storage_gb: 10.0
  policies:
    memory_over_commit: false
    cpu_over_commit: true
    storage_over_commit: false
    resource_quotas: {}
security:
  authentication:
    enabled: true
    method: oauth2
    providers: [github]
  authorization:
    enabled: true
    method: rbac
    policies: [default]
  encryption:
    enabled: true
    algorithm: AES256
    key_size: 256
    at_rest: true
    in_transit: true
  tokens:
    enabled: true
    expiration_seconds: 3600
    refresh_enabled: true
  policies:
    network_policies: []
    pod_security_policies: []
    rbac_policies: []
storage:
  volumes: []
  volume_claim_templates: []
  storage_classes: []
networking:
  ingress:
    enabled: true
    host: test.example.com
    tls_enabled: true
    annotations: {}
  network_policies: []
  dns:
    enabled: true
    servers: ["8.8.8.8"]
    search_domains: ["test.local"]
primals: {}
"#;

        let result = parser.parse_content(yaml_content).await;
        if let Err(ref e) = result {
            eprintln!("Manifest parsing error: {:?}", e);
        }
        assert!(
            result.is_ok(),
            "Failed to parse manifest: {:?}",
            result.as_ref().unwrap_err()
        );

        let manifest = result.expect("parse_content succeeded");
        assert_eq!(manifest.metadata.name, "test-biome");
        assert_eq!(manifest.metadata.version, "1.0.0");
    }
}
