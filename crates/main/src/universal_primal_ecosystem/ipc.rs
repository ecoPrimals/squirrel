// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Capability request transport: Unix sockets and HTTP delegation hooks.

use crate::error::PrimalError;
use crate::universal::{PrimalRequest, PrimalResponse, UniversalResult};

use super::UniversalPrimalEcosystem;
use super::types::DiscoveredService;

impl UniversalPrimalEcosystem {
    /// Send capability-based request with comprehensive resilience and observability
    pub(crate) async fn send_capability_request(
        &self,
        service: &DiscoveredService,
        request: PrimalRequest,
    ) -> UniversalResult<PrimalResponse> {
        // Modern TRUE PRIMAL implementation: JSON-RPC over Unix sockets
        tracing::debug!(
            "Sending capability request to service {} at {}",
            service.service_id,
            service.endpoint
        );

        // Parse endpoint to determine transport
        if service.endpoint.starts_with("unix://") {
            // Unix socket communication (TRUE PRIMAL pattern)
            self.send_unix_socket_request(service, request).await
        } else if service.endpoint.starts_with("http://")
            || service.endpoint.starts_with("https://")
        {
            // HTTP requests must be delegated via service mesh (concentrated gap strategy)
            self.delegate_to_http_proxy(service, request).await
        } else {
            Err(PrimalError::InvalidEndpoint(format!(
                "Unknown endpoint protocol: {}. Expected unix:// or http(s)://",
                service.endpoint
            )))
        }
    }

    /// Send request via Unix socket (TRUE PRIMAL pattern)
    async fn send_unix_socket_request(
        &self,
        service: &DiscoveredService,
        request: PrimalRequest,
    ) -> UniversalResult<PrimalResponse> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::UnixStream;

        let socket_path = service
            .endpoint
            .strip_prefix("unix://")
            .ok_or_else(|| PrimalError::InvalidEndpoint(service.endpoint.clone()))?;

        // Connect to Unix socket
        let mut stream = UnixStream::connect(socket_path).await.map_err(|e| {
            PrimalError::NetworkError(format!(
                "Failed to connect to Unix socket {socket_path}: {e}"
            ))
        })?;

        // Serialize request as JSON-RPC 2.0
        let json_rpc_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": request.request_id.to_string(),
            "method": request.operation,
            "params": request.payload,
        });

        let request_bytes = serde_json::to_vec(&json_rpc_request).map_err(|e| {
            PrimalError::SerializationError(format!("Failed to serialize request: {e}"))
        })?;

        // Send request
        stream
            .write_all(&request_bytes)
            .await
            .map_err(|e| PrimalError::NetworkError(format!("Failed to write to socket: {e}")))?;

        stream
            .write_all(b"\n")
            .await
            .map_err(|e| PrimalError::NetworkError(format!("Failed to write delimiter: {e}")))?;

        // Read response
        let mut response_bytes = Vec::new();
        stream
            .read_to_end(&mut response_bytes)
            .await
            .map_err(|e| PrimalError::NetworkError(format!("Failed to read from socket: {e}")))?;

        // Deserialize JSON-RPC response
        let json_rpc_response: serde_json::Value = serde_json::from_slice(&response_bytes)
            .map_err(|e| {
                PrimalError::SerializationError(format!("Failed to deserialize response: {e}"))
            })?;

        let result = universal_patterns::extract_rpc_result(&json_rpc_response)
            .map_err(|rpc_err| PrimalError::RemoteError(rpc_err.to_string()))?;
        let result = &result;

        // Convert to PrimalResponse
        Ok(PrimalResponse {
            request_id: request.request_id,
            response_id: uuid::Uuid::new_v4(),
            status: crate::universal::ResponseStatus::Success,
            success: true,
            data: Some(result.clone()),
            payload: result.clone(),
            timestamp: chrono::Utc::now(),
            processing_time_ms: None,
            duration: None,
            error: None,
            error_message: None,
            metadata: std::collections::HashMap::new(),
        })
    }

    /// Delegate HTTP request via service mesh (concentrated gap strategy).
    ///
    /// Squirrel discovers the `http.proxy` capability at runtime rather
    /// than hardcoding which primal provides it.
    async fn delegate_to_http_proxy(
        &self,
        service: &DiscoveredService,
        _request: PrimalRequest,
    ) -> UniversalResult<PrimalResponse> {
        tracing::warn!(
            "HTTP request needed for {}. Delegating via 'http.proxy' capability discovery.",
            service.service_id
        );

        Err(PrimalError::NotImplemented(
            "HTTP delegation via capability discovery not yet implemented. \
             TRUE PRIMAL pattern: discover 'http.proxy' capability and delegate."
                .to_string(),
        ))
    }
}

#[cfg(all(test, unix))]
#[expect(
    clippy::expect_used,
    reason = "Invariant or startup failure: expect after validation"
)]
mod ipc_tests {
    use crate::error::PrimalError;
    use crate::universal::PrimalRequest;
    use crate::universal_primal_ecosystem::{
        DiscoveredService, ServiceHealth, UniversalPrimalEcosystem,
    };

    fn sample_service(endpoint: &str) -> DiscoveredService {
        DiscoveredService {
            service_id: "svc".to_string(),
            instance_id: "i1".to_string(),
            endpoint: endpoint.to_string(),
            capabilities: vec!["test".to_string()],
            health: ServiceHealth::Healthy,
            discovered_at: chrono::Utc::now(),
            last_health_check: None,
        }
    }

    #[tokio::test]
    async fn send_capability_rejects_unknown_endpoint_scheme() {
        let eco = UniversalPrimalEcosystem::new(crate::universal::PrimalContext::default());
        let svc = sample_service("ftp://noop");
        let req = PrimalRequest::new(
            "a",
            "cap",
            "op",
            serde_json::json!({}),
            crate::universal::PrimalContext::default(),
        );
        let err = eco
            .send_capability_request(&svc, req)
            .await
            .expect_err("expected bad endpoint");
        assert!(matches!(err, PrimalError::InvalidEndpoint(_)));
    }

    #[tokio::test]
    async fn send_capability_http_returns_not_implemented() {
        let eco = UniversalPrimalEcosystem::new(crate::universal::PrimalContext::default());
        let svc = sample_service("https://example.com");
        let req = PrimalRequest::new(
            "a",
            "cap",
            "op",
            serde_json::json!({}),
            crate::universal::PrimalContext::default(),
        );
        let err = eco
            .send_capability_request(&svc, req)
            .await
            .expect_err("expected not implemented");
        assert!(matches!(err, PrimalError::NotImplemented(_)));
    }

    #[tokio::test]
    async fn unix_socket_jsonrpc_roundtrip() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("ipc.sock");
        let listener = tokio::net::UnixListener::bind(&sock).expect("bind");

        let req_id = uuid::Uuid::new_v4();
        let id_str = req_id.to_string();

        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("accept");
            use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
            let mut reader = BufReader::new(&mut stream);
            let mut buf = Vec::new();
            reader.read_until(b'\n', &mut buf).await.expect("read");
            let resp = serde_json::json!({
                "jsonrpc": "2.0",
                "id": id_str,
                "result": {"ok": true, "message": "pong"}
            });
            let mut line = serde_json::to_string(&resp).expect("ipc response json");
            line.push('\n');
            stream.write_all(line.as_bytes()).await.ok();
            drop(stream);
        });

        let eco = UniversalPrimalEcosystem::new(crate::universal::PrimalContext::default());
        let endpoint = format!("unix://{}", sock.to_string_lossy());
        let svc = sample_service(&endpoint);
        let req = PrimalRequest::new(
            "src",
            "cap",
            "ping",
            serde_json::json!({"x": 1}),
            crate::universal::PrimalContext::default(),
        );
        let mut req = req;
        req.request_id = req_id;

        let out = eco
            .send_capability_request(&svc, req)
            .await
            .expect("unix ok");
        assert!(out.success);
        server.await.ok();
    }
}
