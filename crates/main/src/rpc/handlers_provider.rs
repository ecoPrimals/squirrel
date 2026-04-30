// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Provider registration domain JSON-RPC handlers — `provider.register`,
//! `provider.list`, `provider.deregister`.
//!
//! Springs in a composition call `provider.register` to announce their
//! capabilities and socket path to Squirrel. This allows Squirrel to route
//! future capability-based requests to them without relying on filesystem
//! scanning or external discovery services.

use super::jsonrpc_server::{JsonRpcError, JsonRpcServer, error_codes};
use crate::universal_adapters::registry::UniversalServiceRegistry;
use crate::universal_adapters::{
    IntegrationPreferences, ResourceSpec, ServiceCapability, ServiceCategory, ServiceEndpoint,
    ServiceMetadata, UniversalServiceRegistration,
};
use serde_json::Value;
use std::collections::HashMap;
use tracing::{info, warn};

impl JsonRpcServer {
    /// Handle `provider.register` — a spring registers its capabilities with Squirrel.
    ///
    /// # Wire format
    ///
    /// ```json
    /// {
    ///   "provider_id": "neuralspring-01",
    ///   "socket": "/run/user/1000/biomeos/neuralspring-desktop-nucleus.sock",
    ///   "capabilities": ["neural.validate", "neural.simulate"],
    ///   "version": "0.9.24",
    ///   "domain": "neural"
    /// }
    /// ```
    ///
    /// Optional fields: `endpoint` (HTTP URL), `priority` (0–255, default 50),
    /// `metadata` (arbitrary JSON object).
    #[allow(clippy::too_many_lines)]
    pub(crate) async fn handle_provider_register(
        &self,
        params: Option<Value>,
    ) -> Result<Value, JsonRpcError> {
        let params = params.ok_or_else(|| JsonRpcError {
            code: error_codes::INVALID_PARAMS,
            message: "provider.register requires params".into(),
            data: None,
        })?;

        let provider_id = params
            .get("provider_id")
            .and_then(Value::as_str)
            .ok_or_else(|| JsonRpcError {
                code: error_codes::INVALID_PARAMS,
                message: "provider.register requires 'provider_id' (string)".into(),
                data: None,
            })?;

        let trimmed_id = provider_id.trim();
        if trimmed_id.is_empty() || trimmed_id.len() > 256 {
            return Err(JsonRpcError {
                code: error_codes::INVALID_PARAMS,
                message: "provider_id must be 1–256 non-whitespace characters".into(),
                data: None,
            });
        }

        let raw_capabilities = params
            .get("capabilities")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(String::from)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        if raw_capabilities.is_empty() {
            return Err(JsonRpcError {
                code: error_codes::INVALID_PARAMS,
                message: "provider.register requires non-empty 'capabilities' array".into(),
                data: None,
            });
        }

        let socket = params.get("socket").and_then(Value::as_str);
        let endpoint = params.get("endpoint").and_then(Value::as_str);

        if socket.is_none() && endpoint.is_none() {
            return Err(JsonRpcError {
                code: error_codes::INVALID_PARAMS,
                message: "provider.register requires 'socket' or 'endpoint'".into(),
                data: None,
            });
        }

        let mut endpoints = Vec::new();
        if let Some(s) = socket {
            endpoints.push(ServiceEndpoint {
                name: format!("{trimmed_id}-uds"),
                url: format!("unix://{s}"),
                protocol: "jsonrpc-2.0".to_string(),
                port: None,
                path: Some(s.to_string()),
            });
        }
        if let Some(e) = endpoint {
            endpoints.push(ServiceEndpoint {
                name: format!("{trimmed_id}-http"),
                url: e.to_string(),
                protocol: "http".to_string(),
                port: None,
                path: None,
            });
        }

        let version = params
            .get("version")
            .and_then(Value::as_str)
            .unwrap_or("0.0.0");
        let domain = params
            .get("domain")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let priority = params
            .get("priority")
            .and_then(Value::as_u64)
            .map_or(50, |p| p.min(255) as u8);

        let capabilities: Vec<ServiceCapability> = raw_capabilities
            .iter()
            .map(|cap| ServiceCapability::Custom {
                domain: domain.to_string(),
                capability: cap.clone(),
                parameters: HashMap::new(),
            })
            .collect();

        // Deterministic service_id from provider_id for upsert and name-based deregister.
        // We hash the provider_id bytes into a UUID v4 layout (bits are deterministic).
        let service_uuid = deterministic_uuid(trimmed_id);

        let registration = UniversalServiceRegistration {
            service_id: service_uuid,
            metadata: ServiceMetadata {
                name: trimmed_id.to_string(),
                category: ServiceCategory::Custom {
                    category: domain.to_string(),
                    subcategories: vec![],
                },
                version: version.to_string(),
                description: format!("{domain} provider"),
                maintainer: trimmed_id.to_string(),
                protocols: vec!["jsonrpc-2.0".to_string()],
            },
            capabilities,
            endpoints,
            resources: ResourceSpec {
                cpu_cores: None,
                memory_gb: None,
                storage_gb: None,
                network_bandwidth: None,
                custom_resources: HashMap::new(),
            },
            integration: IntegrationPreferences {
                preferred_protocols: vec!["jsonrpc-2.0".to_string()],
                retry_policy: "exponential".to_string(),
                timeout_seconds: 30,
                load_balancing_weight: priority,
            },
            extensions: params
                .get("metadata")
                .and_then(Value::as_object)
                .map(|o| o.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                .unwrap_or_default(),
            registration_timestamp: chrono::Utc::now(),
            service_version: version.to_string(),
            instance_id: trimmed_id.to_string(),
            priority,
        };

        self.provider_registry
            .register_service(registration)
            .await
            .map_err(|e| JsonRpcError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Registration failed: {e}"),
                data: None,
            })?;

        info!(
            provider = trimmed_id,
            domain = domain,
            "Provider registered via provider.register"
        );

        Ok(serde_json::json!({
            "success": true,
            "provider_id": trimmed_id,
            "message": format!("Provider '{trimmed_id}' registered"),
        }))
    }

    /// Handle `provider.list` — enumerate all registered providers.
    pub(crate) async fn handle_provider_list(&self) -> Result<Value, JsonRpcError> {
        let services = self
            .provider_registry
            .list_all_services()
            .await
            .map_err(|e| JsonRpcError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Failed to list providers: {e}"),
                data: None,
            })?;

        let providers: Vec<Value> = services
            .iter()
            .map(|s| {
                serde_json::json!({
                    "provider_id": s.name.as_ref(),
                    "domain": s.category.as_ref(),
                    "capabilities": s.capabilities.iter().map(capability_name).collect::<Vec<_>>(),
                    "endpoints": s.endpoints.iter().map(std::convert::AsRef::as_ref).collect::<Vec<_>>(),
                    "health": format!("{:?}", s.health),
                    "priority": s.priority,
                })
            })
            .collect();

        Ok(serde_json::json!({
            "providers": providers,
            "count": providers.len(),
        }))
    }

    /// Handle `provider.deregister` — remove a provider by ID.
    pub(crate) async fn handle_provider_deregister(
        &self,
        params: Option<Value>,
    ) -> Result<Value, JsonRpcError> {
        let params = params.ok_or_else(|| JsonRpcError {
            code: error_codes::INVALID_PARAMS,
            message: "provider.deregister requires params".into(),
            data: None,
        })?;

        let provider_id = params
            .get("provider_id")
            .and_then(Value::as_str)
            .ok_or_else(|| JsonRpcError {
                code: error_codes::INVALID_PARAMS,
                message: "provider.deregister requires 'provider_id' (string)".into(),
                data: None,
            })?;

        let service_uuid = deterministic_uuid(provider_id);

        self.provider_registry
            .deregister_service(&service_uuid.to_string())
            .await
            .map_err(|e| JsonRpcError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Deregistration failed: {e}"),
                data: None,
            })?;

        warn!(provider = provider_id, "Provider deregistered");

        Ok(serde_json::json!({
            "success": true,
            "provider_id": provider_id,
            "message": format!("Provider '{provider_id}' deregistered"),
        }))
    }
}

/// Produce a deterministic UUID from a provider name.
/// Uses FNV-style mixing into UUID v4 format bits for reproducibility.
fn deterministic_uuid(name: &str) -> uuid::Uuid {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    name.hash(&mut hasher);
    let h = hasher.finish();
    let mut bytes = [0u8; 16];
    bytes[..8].copy_from_slice(&h.to_le_bytes());
    // Mirror upper half for full 128 bits
    let mut hasher2 = std::collections::hash_map::DefaultHasher::new();
    h.hash(&mut hasher2);
    let h2 = hasher2.finish();
    bytes[8..].copy_from_slice(&h2.to_le_bytes());
    uuid::Uuid::from_bytes(bytes)
}

/// Extract a human-readable name from a `ServiceCapability`.
fn capability_name(cap: &ServiceCapability) -> String {
    match cap {
        ServiceCapability::Custom {
            capability, domain, ..
        } => {
            if capability.contains('.') {
                capability.clone()
            } else {
                format!("{domain}.{capability}")
            }
        }
        ServiceCapability::Security { functions, .. } => {
            format!("security[{}]", functions.join(","))
        }
        ServiceCapability::Coordination { patterns, .. } => {
            format!("coordination[{}]", patterns.join(","))
        }
        ServiceCapability::DataManagement { operations, .. } => {
            format!("data[{}]", operations.join(","))
        }
        ServiceCapability::Computation { types, .. } => {
            format!("compute[{}]", types.join(","))
        }
        ServiceCapability::ArtificialIntelligence { tasks, .. } => {
            format!("ai[{}]", tasks.join(","))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::rpc::JsonRpcServer;
    use serde_json::json;

    #[tokio::test]
    async fn register_provider_success() {
        let server = JsonRpcServer::new("/tmp/provider-reg-test.sock".to_string());
        let params = Some(json!({
            "provider_id": "neuralspring-01",
            "socket": "/tmp/neuralspring.sock",
            "capabilities": ["neural.validate", "neural.simulate"],
            "version": "0.9.24",
            "domain": "neural"
        }));
        let result = server
            .handle_provider_register(params)
            .await
            .expect("should succeed");
        assert_eq!(
            result.get("success").and_then(serde_json::Value::as_bool),
            Some(true)
        );
        assert_eq!(
            result.get("provider_id").and_then(|v| v.as_str()),
            Some("neuralspring-01")
        );
    }

    #[tokio::test]
    async fn register_requires_capabilities() {
        let server = JsonRpcServer::new("/tmp/provider-reg-test2.sock".to_string());
        let params = Some(json!({
            "provider_id": "test",
            "socket": "/tmp/x.sock",
            "capabilities": []
        }));
        let result = server.handle_provider_register(params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn register_requires_socket_or_endpoint() {
        let server = JsonRpcServer::new("/tmp/provider-reg-test3.sock".to_string());
        let params = Some(json!({
            "provider_id": "test",
            "capabilities": ["foo.bar"]
        }));
        let result = server.handle_provider_register(params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn list_providers_after_registration() {
        let server = JsonRpcServer::new("/tmp/provider-list-test.sock".to_string());
        let params = Some(json!({
            "provider_id": "spring-a",
            "socket": "/tmp/a.sock",
            "capabilities": ["data.fetch"],
            "domain": "data"
        }));
        server
            .handle_provider_register(params)
            .await
            .expect("register");
        let list = server.handle_provider_list().await.expect("list");
        assert_eq!(
            list.get("count").and_then(serde_json::Value::as_u64),
            Some(1)
        );
    }

    #[tokio::test]
    async fn deregister_provider() {
        let server = JsonRpcServer::new("/tmp/provider-dereg-test.sock".to_string());
        let params = Some(json!({
            "provider_id": "to-remove",
            "socket": "/tmp/rm.sock",
            "capabilities": ["x.y"],
            "domain": "x"
        }));
        server
            .handle_provider_register(params)
            .await
            .expect("register");

        let list = server.handle_provider_list().await.expect("list");
        assert_eq!(
            list.get("count").and_then(serde_json::Value::as_u64),
            Some(1)
        );

        let dereg = Some(json!({ "provider_id": "to-remove" }));
        server
            .handle_provider_deregister(dereg)
            .await
            .expect("deregister");

        let list = server
            .handle_provider_list()
            .await
            .expect("list after dereg");
        assert_eq!(
            list.get("count").and_then(serde_json::Value::as_u64),
            Some(0)
        );
    }

    #[tokio::test]
    async fn register_with_http_endpoint() {
        let server = JsonRpcServer::new("/tmp/provider-http-test.sock".to_string());
        let params = Some(json!({
            "provider_id": "ollama-local",
            "endpoint": "http://localhost:11434",
            "capabilities": ["inference.generate", "inference.embed"],
            "version": "0.3.0",
            "domain": "inference"
        }));
        let result = server
            .handle_provider_register(params)
            .await
            .expect("HTTP endpoint registration should succeed");
        assert_eq!(
            result.get("success").and_then(serde_json::Value::as_bool),
            Some(true)
        );
    }

    #[tokio::test]
    async fn upsert_semantics() {
        let server = JsonRpcServer::new("/tmp/provider-upsert-test.sock".to_string());
        let params = || {
            Some(json!({
                "provider_id": "same-id",
                "socket": "/tmp/same.sock",
                "capabilities": ["a.b"],
                "domain": "a"
            }))
        };
        server
            .handle_provider_register(params())
            .await
            .expect("first");
        server
            .handle_provider_register(params())
            .await
            .expect("second (upsert)");

        let list = server.handle_provider_list().await.expect("list");
        assert_eq!(
            list.get("count").and_then(serde_json::Value::as_u64),
            Some(1),
            "Duplicate registration should upsert, not duplicate"
        );
    }
}
