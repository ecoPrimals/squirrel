// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Tool domain JSON-RPC handlers: `tool.execute`, `tool.list`.

use super::jsonrpc_server::{JsonRpcError, JsonRpcServer, error_codes};
use serde_json::Value;
use tracing::{debug, info};

impl JsonRpcServer {
    /// Handle `tool.execute` — execute a tool locally or forward to a remote primal.
    ///
    /// Checks the announced-primal registry first. If the tool was registered
    /// by a remote primal via `capability.announce`, the request is forwarded
    /// to that primal's Unix socket. Otherwise, local execution proceeds.
    pub(crate) async fn handle_execute_tool(
        &self,
        params: Option<Value>,
    ) -> Result<Value, JsonRpcError> {
        info!("execute_tool request");

        let tool_params = params.ok_or_else(|| JsonRpcError {
            code: error_codes::INVALID_PARAMS,
            message: "Missing parameters for execute_tool".to_string(),
            data: None,
        })?;

        let tool_name = tool_params
            .get("tool")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JsonRpcError {
                code: error_codes::INVALID_PARAMS,
                message: "Missing 'tool' parameter".to_string(),
                data: None,
            })?;

        let args = tool_params
            .get("args")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));

        let remote = {
            let registry = self.announced_tools.read().await;
            registry
                .get(tool_name)
                .map(|a| std::sync::Arc::clone(&a.socket_path))
        };

        if let Some(socket_path) = remote {
            info!(
                "Forwarding tool '{}' to remote primal at {}",
                tool_name, socket_path
            );
            return self
                .forward_tool_to_remote(tool_name, &args, socket_path.as_ref())
                .await;
        }

        let executor = crate::tool::ToolExecutor::new();

        // Prefer local built-ins over spring routing (springs may advertise overlapping names).
        if !executor.available_tools.contains_key(tool_name) {
            // Check spring tool routing table (mcp.tools.list discovery)
            let spring_discovery = super::spring_tools::SpringToolDiscovery::new();
            let routing_table = spring_discovery.build_routing_table().await;
            if let Some(socket_path) = routing_table.get(tool_name) {
                info!("Routing tool '{}' to spring at {}", tool_name, socket_path);
                return self
                    .forward_tool_to_remote(tool_name, &args, socket_path.as_ref())
                    .await;
            }
        }

        info!("Executing local tool: {tool_name}");
        let args_str = serde_json::to_string(&args).map_err(|e| JsonRpcError {
            code: error_codes::INVALID_PARAMS,
            message: format!("Invalid tool args: {e}"),
            data: None,
        })?;

        match executor.execute_tool(tool_name, &args_str).await {
            Ok(result) => {
                let response = serde_json::json!({
                    "tool": result.tool_name,
                    "success": result.success,
                    "output": result.output,
                    "error": result.error,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                });
                Ok(response)
            }
            Err(e) => Err(JsonRpcError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Tool execution failed: {e}"),
                data: Some(serde_json::json!({ "tool": tool_name })),
            }),
        }
    }

    /// Forward a `tool.execute` call to a remote primal via Unix socket.
    pub(crate) async fn forward_tool_to_remote(
        &self,
        tool_name: &str,
        args: &Value,
        socket_path: &str,
    ) -> Result<Value, JsonRpcError> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        use tokio::net::UnixStream;

        let stream = UnixStream::connect(socket_path)
            .await
            .map_err(|e| JsonRpcError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Failed to connect to remote primal at {socket_path}: {e}"),
                data: Some(serde_json::json!({ "tool": tool_name, "socket": socket_path })),
            })?;

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "tool.execute",
            "params": { "tool": tool_name, "args": args },
            "id": 1
        });

        let mut request_line = serde_json::to_string(&request).map_err(|e| JsonRpcError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Serialization error: {e}"),
            data: None,
        })?;
        request_line.push('\n');

        let (reader, mut writer) = tokio::io::split(stream);
        writer
            .write_all(request_line.as_bytes())
            .await
            .map_err(|e| JsonRpcError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Failed to write to remote primal: {e}"),
                data: None,
            })?;

        writer.flush().await.map_err(|e| JsonRpcError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Failed to flush to remote primal: {e}"),
            data: None,
        })?;

        let mut buf_reader = BufReader::new(reader);
        let mut response_line = String::new();
        buf_reader
            .read_line(&mut response_line)
            .await
            .map_err(|e| JsonRpcError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Failed to read from remote primal: {e}"),
                data: None,
            })?;

        let response: Value =
            serde_json::from_str(response_line.trim()).map_err(|e| JsonRpcError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Invalid response from remote primal: {e}"),
                data: None,
            })?;

        universal_patterns::extract_rpc_result(&response).map_err(|rpc_err| JsonRpcError {
            code: rpc_err.code,
            message: rpc_err.message,
            data: rpc_err.data,
        })
    }

    /// Handle `tool.list` — list all available tools.
    ///
    /// Returns local built-ins + tools announced by remote primals +
    /// tools discovered from domain springs via `mcp.tools.list`.
    /// Tool definitions enriched with JSON Schema from capability_registry.toml.
    pub(crate) async fn handle_list_tools(&self) -> Result<Value, JsonRpcError> {
        debug!("tool.list request");

        let executor = crate::tool::ToolExecutor::new();
        let mut entries: Vec<super::types::ToolListEntry> = executor
            .list_tools()
            .iter()
            .map(|t| {
                let schema = self
                    .capability_registry
                    .find(&t.name)
                    .and_then(|c| c.input_schema.clone());
                super::types::ToolListEntry {
                    name: t.name.to_string(),
                    description: t.description.clone(),
                    domain: t.domain.to_string(),
                    source: super::types::ToolSource::Builtin,
                    input_schema: schema,
                }
            })
            .collect();

        let mut seen: std::collections::HashSet<String> =
            entries.iter().map(|e| e.name.clone()).collect();

        for cap in &self.capability_registry.capabilities {
            if seen.insert(cap.method.clone()) {
                entries.push(super::types::ToolListEntry {
                    name: cap.method.clone(),
                    description: cap.description.clone(),
                    domain: cap.domain.clone(),
                    source: super::types::ToolSource::Builtin,
                    input_schema: cap.input_schema.clone(),
                });
            }
        }

        let announced = self.announced_tools.read().await;
        for (tool_name, announced_primal) in announced.iter() {
            if seen.insert(tool_name.as_ref().to_string()) {
                let domain = tool_name
                    .split('.')
                    .next()
                    .unwrap_or("external")
                    .to_string();
                entries.push(super::types::ToolListEntry {
                    name: tool_name.as_ref().to_string(),
                    description: format!("Remote tool from {}", announced_primal.primal),
                    domain,
                    source: super::types::ToolSource::Remote {
                        primal: announced_primal.primal.as_ref().to_string(),
                    },
                    input_schema: None,
                });
            }
        }

        // Discover tools from domain springs via mcp.tools.list
        let spring_discovery = super::spring_tools::SpringToolDiscovery::new();
        let spring_tools = spring_discovery.discover_spring_tools().await;
        for (tool_def, _socket_path) in &spring_tools {
            if seen.insert(tool_def.name.clone()) {
                entries.push(super::types::ToolListEntry {
                    name: tool_def.name.clone(),
                    description: tool_def.description.clone(),
                    domain: tool_def.domain.clone(),
                    source: super::types::ToolSource::Remote {
                        primal: format!("{}-spring", tool_def.domain),
                    },
                    input_schema: tool_def.input_schema.clone(),
                });
            }
        }

        let total = entries.len();
        let response = super::types::ToolListResponse {
            tools: entries,
            total,
        };

        serde_json::to_value(response).map_err(|e| JsonRpcError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Serialization error: {e}"),
            data: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::rpc::JsonRpcServer;
    use crate::rpc::jsonrpc_server::error_codes;

    #[tokio::test]
    async fn execute_tool_rejects_missing_params() {
        let server = JsonRpcServer::new("/tmp/sq-tool-missing-params.sock".to_string());
        let err = server.handle_execute_tool(None).await.unwrap_err();
        assert_eq!(err.code, error_codes::INVALID_PARAMS);
    }

    #[tokio::test]
    async fn execute_tool_rejects_missing_tool_name() {
        let server = JsonRpcServer::new("/tmp/sq-tool-missing-tool.sock".to_string());
        let err = server
            .handle_execute_tool(Some(serde_json::json!({ "args": {} })))
            .await
            .unwrap_err();
        assert_eq!(err.code, error_codes::INVALID_PARAMS);
    }

    #[tokio::test]
    async fn execute_tool_local_system_health_builds_response() {
        let server = JsonRpcServer::new("/tmp/sq-tool-health.sock".to_string());
        let params = Some(serde_json::json!({
            "tool": "system.health",
            "args": {}
        }));
        let v = server.handle_execute_tool(params).await.unwrap();
        assert_eq!(v.get("success"), Some(&serde_json::json!(true)));
        assert!(v.get("output").and_then(|o| o.as_str()).is_some());
    }

    #[tokio::test]
    async fn execute_tool_unknown_tool_returns_failure_in_payload() {
        let server = JsonRpcServer::new("/tmp/sq-tool-unknown.sock".to_string());
        let params = Some(serde_json::json!({
            "tool": "not.a.registered.tool",
            "args": {}
        }));
        let v = server.handle_execute_tool(params).await.unwrap();
        assert_eq!(v.get("success"), Some(&serde_json::json!(false)));
    }

    #[tokio::test]
    async fn list_tools_returns_non_empty() {
        let server = JsonRpcServer::new("/tmp/sq-tool-list.sock".to_string());
        let v = server.handle_list_tools().await.unwrap();
        let tools = v.get("tools").and_then(|t| t.as_array());
        assert!(tools.is_some_and(|a| !a.is_empty()));
        assert!(v.get("total").and_then(|n| n.as_u64()).unwrap_or(0) > 0);
    }

    #[tokio::test]
    async fn execute_tool_omitted_args_defaults_to_empty_object() {
        let server = JsonRpcServer::new("/tmp/sq-tool-no-args.sock".to_string());
        let params = Some(serde_json::json!({
            "tool": "system.health"
        }));
        let v = server.handle_execute_tool(params).await.unwrap();
        assert_eq!(v.get("success"), Some(&serde_json::json!(true)));
    }

    #[tokio::test]
    async fn forward_tool_to_remote_missing_socket_errors() {
        let server = JsonRpcServer::new("/tmp/sq-tool-fwd.sock".to_string());
        let err = server
            .forward_tool_to_remote(
                "any.tool",
                &serde_json::json!({}),
                "/nonexistent/path/does-not-exist.sock",
            )
            .await
            .unwrap_err();
        assert_eq!(err.code, error_codes::INTERNAL_ERROR);
        assert!(err.message.contains("Failed to connect"));
    }
}
