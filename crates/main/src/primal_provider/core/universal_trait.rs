// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! [`UniversalPrimalProvider`] trait implementation for [`super::SquirrelPrimalProvider`].

use std::collections::HashMap;

use tracing::info;
use uuid::Uuid;

use crate::error::PrimalError;
use crate::universal::{
    DynamicPortInfo, EcosystemRequest, EcosystemResponse, PrimalCapability, PrimalContext,
    PrimalDependency, PrimalEndpoints, PrimalHealth, PrimalRequest, PrimalResponse, PrimalType,
    ResponseStatus, ServiceMeshStatus, UniversalPrimalProvider, UniversalResult,
};

use super::SquirrelPrimalProvider;

// ============================================================================
// UNIVERSAL PRIMAL PROVIDER TRAIT IMPLEMENTATION
// ============================================================================

impl UniversalPrimalProvider for SquirrelPrimalProvider {
    /// Get the primal ID
    fn primal_id(&self) -> &'static str {
        "squirrel"
    }

    /// Get the instance ID
    fn instance_id(&self) -> &str {
        &self.instance_id
    }

    /// Get the context
    fn context(&self) -> &PrimalContext {
        &self.context
    }

    /// Get the primal type
    fn primal_type(&self) -> PrimalType {
        PrimalType::AI
    }

    /// Get available capabilities
    fn capabilities(&self) -> Vec<PrimalCapability> {
        vec![
            PrimalCapability::ModelInference {
                models: vec!["gpt-4".to_string(), "claude-3".to_string()],
            },
            PrimalCapability::AgentFramework {
                frameworks: vec!["langchain".to_string(), "autogen".to_string()],
                mcp_support: true,
            },
            PrimalCapability::NaturalLanguage {
                languages: vec!["en".to_string(), "es".to_string()],
            },
        ]
    }

    /// Get dependencies
    fn dependencies(&self) -> Vec<PrimalDependency> {
        vec![
            PrimalDependency {
                primal_type: "Storage".to_string(),
                required: false,
                optional: true,
                required_capabilities: vec![],
                capabilities: vec![],
                min_version: Some("1.0.0".to_string()),
                preferred_instance: None,
            },
            PrimalDependency {
                primal_type: "Compute".to_string(),
                required: false,
                optional: true,
                required_capabilities: vec![],
                capabilities: vec![],
                min_version: Some("1.0.0".to_string()),
                preferred_instance: None,
            },
        ]
    }

    /// Perform health check
    async fn health_check(&self) -> PrimalHealth {
        super::super::health_monitoring::HealthReporting::generate_health_report(self)
    }

    /// Get service endpoints
    fn endpoints(&self) -> PrimalEndpoints {
        let host = std::env::var("SERVICE_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = std::env::var("SERVICE_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);
        let base_url = format!("http://{host}:{port}");
        let ws_url = format!("ws://{host}:{port}/ws");

        PrimalEndpoints {
            http: Some(base_url.clone()), // Added http field
            grpc: None,                   // Added grpc field (optional)
            primary: Some(base_url.clone()),
            health: Some(format!("{base_url}/health")),
            metrics: Some(format!("{base_url}/metrics")),
            admin: Some(format!("{base_url}/admin")),
            websocket: Some(ws_url),
            mcp: Some(format!("{base_url}/mcp")),
            ai_coordination: Some(format!("{base_url}/ai")),
            service_mesh: Some(format!("{base_url}/mesh")),
            custom: Vec::new(), // Changed from HashMap to Vec<(String, String)>
        }
    }

    /// Handle a primal request
    async fn handle_primal_request(
        &self,
        request: PrimalRequest,
    ) -> UniversalResult<PrimalResponse> {
        info!("Handling primal request: {}", request.operation);

        let response_payload = match request.operation.as_str() {
            "ai_inference" => self.handle_ai_inference_request(request.payload).await?,
            "context_analysis" => {
                self.handle_context_analysis_request(request.payload)
                    .await?
            }
            "session_create" => self.create_session(request.payload).await?,
            "session_get" => self.get_session(request.payload).await?,
            "session_update" => self.update_session(request.payload).await?,
            "session_delete" => self.delete_session(request.payload).await?,
            "session_list" => self.list_user_sessions(request.payload).await?,
            "health_check" => {
                let healthy = self.perform_health_check().await?;
                serde_json::json!({
                    "healthy": healthy,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                })
            }
            _ => {
                return Err(PrimalError::OperationNotSupported(format!(
                    "Unknown operation: {}",
                    request.operation
                )));
            }
        };

        Ok(PrimalResponse {
            response_id: Uuid::new_v4(),
            request_id: request.request_id,
            success: true,
            data: Some(serde_json::Value::Object(serde_json::Map::new())),
            payload: response_payload,
            duration: Some("100ms".to_string()),
            processing_time_ms: Some(100),
            error_message: None,
            status: ResponseStatus::Success,
            metadata: HashMap::new(),
            error: None,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Initialize the primal
    async fn initialize(&mut self, _config: serde_json::Value) -> UniversalResult<()> {
        self.initialize_ecosystem()
            .await
            .map_err(|e| PrimalError::Internal(e.to_string()))?;
        Ok(())
    }

    /// Shutdown the primal
    async fn shutdown(&mut self) -> UniversalResult<()> {
        self.shutdown_ecosystem()
            .await
            .map_err(|e| PrimalError::Internal(e.to_string()))?;
        Ok(())
    }

    /// Validate if primal can serve the context
    fn can_serve_context(&self, context: &PrimalContext) -> bool {
        Self::can_serve_context(self, context)
    }

    /// Get dynamic port information
    fn dynamic_port_info(&self) -> Option<DynamicPortInfo> {
        Self::dynamic_port_info(self)
    }

    /// Register with service mesh (capability-based)
    async fn register_with_service_mesh(
        &mut self,
        service_mesh_endpoint: &str,
    ) -> UniversalResult<String> {
        // Delegate to inherent implementation (`primal_provider::ecosystem_integration`) so the
        // universal trait stays aligned with the real registration id and logging.
        Self::register_with_service_mesh(self, service_mesh_endpoint).await
    }

    /// Deregister from service mesh
    async fn deregister_from_service_mesh(&mut self) -> UniversalResult<()> {
        Self::deregister_from_service_mesh(self).await
    }

    /// Get service mesh status
    fn get_service_mesh_status(&self) -> ServiceMeshStatus {
        Self::get_service_mesh_status(self)
    }

    /// Handle ecosystem request
    async fn handle_ecosystem_request(
        &self,
        _request: EcosystemRequest,
    ) -> UniversalResult<EcosystemResponse> {
        // FUTURE: [Ecosystem-Integration] Implement ecosystem request handling
        // Tracking: Planned for v0.2.0 - ecosystem integration work
        // This should:
        // 1. Parse the ecosystem request (capability queries, service discovery, etc.)
        // 2. Route to appropriate handler based on request type
        // 3. Use ecosystem_manager for capability discovery and coordination
        // 4. Return proper ecosystem response with results
        Ok(EcosystemResponse {
            request_id: uuid::Uuid::new_v4(),
            response_id: uuid::Uuid::new_v4(),
            status: ResponseStatus::Success,
            success: true,
            payload: serde_json::json!({}),
            error_message: None,
            timestamp: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        })
    }

    /// Report health to ecosystem registry
    async fn report_health(&self, _health: PrimalHealth) -> UniversalResult<()> {
        // FUTURE: [Ecosystem-Integration] Implement health reporting to ecosystem registry
        // Tracking: Planned for v0.2.0 - ecosystem integration work
        // This should:
        // 1. Convert PrimalHealth to ecosystem health format
        // 2. Report to ecosystem_manager or health registry
        // 3. Handle reporting failures gracefully
        Ok(())
    }

    /// Update system capabilities
    async fn update_capabilities(
        &self,
        _capabilities: Vec<PrimalCapability>,
    ) -> UniversalResult<()> {
        // FUTURE: [Ecosystem-Integration] Implement capability updates in ecosystem registry
        // Tracking: Planned for v0.2.0 - ecosystem integration work
        // This should:
        // 1. Update local capability list
        // 2. Notify ecosystem_manager of capability changes
        // 3. Update service mesh registration if needed
        // Tracked in: ecosystem integration work
        Ok(())
    }
}
