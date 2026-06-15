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

        // riboCipher preamble (Wave 113 outbound compliance)
        universal_patterns::transport::ribocipher::write_ndjson_preamble(&mut stream)
            .await
            .map_err(|e| {
                PrimalError::NetworkError(format!("Failed to write riboCipher preamble: {e}"))
            })?;

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

        Ok(PrimalResponse {
            request_id: request.request_id,
            response_id: uuid::Uuid::new_v4(),
            status: crate::universal::ResponseStatus::Success,
            success: true,
            data: Some(result.clone()),
            payload: result,
            timestamp: chrono::Utc::now(),
            processing_time_ms: None,
            duration: None,
            error: None,
            error_message: None,
            metadata: std::collections::HashMap::new(),
        })
    }

    /// Delegate JSON-RPC request to an HTTP endpoint using raw TCP.
    ///
    /// Implements minimal HTTP/1.1 POST without external dependencies (uniBin
    /// compliant). The request body is a JSON-RPC 2.0 envelope identical to the
    /// Unix socket path. The endpoint URL is parsed to extract host:port and the
    /// path component.
    async fn delegate_to_http_proxy(
        &self,
        service: &DiscoveredService,
        request: PrimalRequest,
    ) -> UniversalResult<PrimalResponse> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let url = url::Url::parse(&service.endpoint).map_err(|e| {
            PrimalError::InvalidEndpoint(format!("Bad HTTP URL {}: {e}", service.endpoint))
        })?;

        let host = url
            .host_str()
            .ok_or_else(|| PrimalError::InvalidEndpoint("Missing host".into()))?;
        let port = url
            .port()
            .unwrap_or_else(|| if url.scheme() == "https" { 443 } else { 80 });
        let path = if url.path().is_empty() {
            "/"
        } else {
            url.path()
        };

        let json_rpc_body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": request.request_id.to_string(),
            "method": request.operation,
            "params": request.payload,
        });
        let body_bytes = serde_json::to_vec(&json_rpc_body)
            .map_err(|e| PrimalError::SerializationError(format!("Serialize request: {e}")))?;

        let http_request = format!(
            "POST {path} HTTP/1.1\r\n\
             Host: {host}\r\n\
             Content-Type: application/json\r\n\
             Content-Length: {len}\r\n\
             Connection: close\r\n\
             \r\n",
            len = body_bytes.len(),
        );

        let addr = format!("{host}:{port}");
        let mut stream = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            tokio::net::TcpStream::connect(&addr),
        )
        .await
        .map_err(|_| PrimalError::NetworkError(format!("Timeout connecting to {addr}")))?
        .map_err(|e| PrimalError::NetworkError(format!("TCP connect to {addr}: {e}")))?;

        stream
            .write_all(http_request.as_bytes())
            .await
            .map_err(|e| PrimalError::NetworkError(format!("Write HTTP headers: {e}")))?;
        stream
            .write_all(&body_bytes)
            .await
            .map_err(|e| PrimalError::NetworkError(format!("Write HTTP body: {e}")))?;

        let mut response_buf = Vec::new();
        stream
            .read_to_end(&mut response_buf)
            .await
            .map_err(|e| PrimalError::NetworkError(format!("Read HTTP response: {e}")))?;

        let response_str = String::from_utf8_lossy(&response_buf);
        let body_start = response_str.find("\r\n\r\n").map_or(0, |i| i + 4);
        let body = &response_buf[body_start..];

        let json_rpc_response: serde_json::Value = serde_json::from_slice(body).map_err(|e| {
            PrimalError::SerializationError(format!("Deserialize HTTP JSON-RPC response: {e}"))
        })?;

        let result = universal_patterns::extract_rpc_result(&json_rpc_response)
            .map_err(|rpc_err| PrimalError::RemoteError(rpc_err.to_string()))?;

        Ok(PrimalResponse {
            request_id: request.request_id,
            response_id: uuid::Uuid::new_v4(),
            status: crate::universal::ResponseStatus::Success,
            success: true,
            data: Some(result.clone()),
            payload: result,
            timestamp: chrono::Utc::now(),
            processing_time_ms: None,
            duration: None,
            error: None,
            error_message: None,
            metadata: std::collections::HashMap::new(),
        })
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
    async fn send_capability_http_attempts_connection() {
        let eco = UniversalPrimalEcosystem::new(crate::universal::PrimalContext::default());
        let svc = sample_service("http://127.0.0.1:1");
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
            .expect_err("expected network error on unreachable port");
        assert!(matches!(err, PrimalError::NetworkError(_)));
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
