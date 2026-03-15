// SPDX-License-Identifier: AGPL-3.0-only
// ORC-Notice: AI coordination mechanics licensed under ORC
// Copyright (C) 2026 DataScienceBioLab

//! Protocol Negotiation
//!
//! Enables automatic protocol selection between JSON-RPC and tarpc
//! at connection time. Supports gradual rollout and backward compatibility.
//!
//! ## Protocol
//!
//! ```text
//! Client → Server: "PROTOCOLS: jsonrpc,tarpc\n"
//! Server → Client: "PROTOCOL: tarpc\n" or "PROTOCOL: jsonrpc\n"
//! [Connection proceeds with selected protocol]
//! ```
//!
//! ## Backward Compatibility
//!
//! If the client doesn't send a protocol negotiation request, the server
//! assumes JSON-RPC (default protocol).

use super::protocol::IpcProtocol;
use anyhow::{Context, Result};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tracing::{debug, info, warn};

/// Protocol negotiation request from client
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolRequest {
    /// Protocols supported by the client (in order of preference)
    pub supported: Vec<IpcProtocol>,
}

impl ProtocolRequest {
    /// Create a new protocol request
    pub fn new(supported: Vec<IpcProtocol>) -> Self {
        Self { supported }
    }

    /// Create a request with all supported protocols
    pub fn all_supported() -> Self {
        Self {
            supported: IpcProtocol::supported(),
        }
    }

    /// Serialize to wire format
    ///
    /// Format: "PROTOCOLS: jsonrpc,tarpc\n"
    pub fn to_wire(&self) -> String {
        let protocols: Vec<String> = self
            .supported
            .iter()
            .map(|p| p.negotiation_name().to_string())
            .collect();
        format!("PROTOCOLS: {}\n", protocols.join(","))
    }

    /// Parse from wire format
    pub fn from_wire(line: &str) -> Result<Self> {
        let line = line.trim();

        let protocols_str = line
            .strip_prefix("PROTOCOLS: ")
            .ok_or_else(|| anyhow::anyhow!("Invalid protocol request: {line}"))?;

        let mut supported = Vec::new();

        for proto_name in protocols_str.split(',') {
            if let Some(proto) = IpcProtocol::from_str(proto_name.trim()) {
                supported.push(proto);
            }
        }

        if supported.is_empty() {
            anyhow::bail!("No valid protocols in request");
        }

        Ok(Self { supported })
    }
}

/// Protocol negotiation response from server
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolResponse {
    /// Selected protocol
    pub selected: IpcProtocol,
}

impl ProtocolResponse {
    /// Create a new protocol response
    pub fn new(selected: IpcProtocol) -> Self {
        Self { selected }
    }

    /// Serialize to wire format
    ///
    /// Format: "PROTOCOL: tarpc\n"
    pub fn to_wire(&self) -> String {
        format!("PROTOCOL: {}\n", self.selected.negotiation_name())
    }

    /// Parse from wire format
    pub fn from_wire(line: &str) -> Result<Self> {
        let line = line.trim();

        let proto_name = line
            .strip_prefix("PROTOCOL: ")
            .ok_or_else(|| anyhow::anyhow!("Invalid protocol response: {line}"))?;

        let selected = IpcProtocol::from_str(proto_name)
            .ok_or_else(|| anyhow::anyhow!("Unknown protocol: {proto_name}"))?;

        Ok(Self { selected })
    }
}

/// Negotiate protocol from client side
///
/// Sends supported protocols to server and waits for response.
///
/// # Arguments
///
/// * `transport` - The transport to negotiate over
/// * `supported` - Protocols supported by client (in preference order)
///
/// # Returns
///
/// Selected protocol
pub async fn negotiate_client<T>(
    transport: &mut T,
    supported: Vec<IpcProtocol>,
) -> Result<IpcProtocol>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    // Send protocol request
    let request = ProtocolRequest::new(supported);
    let request_line = request.to_wire();

    debug!("Client sending protocol request: {:?}", request);
    transport
        .write_all(request_line.as_bytes())
        .await
        .context("Failed to send protocol request")?;
    transport
        .flush()
        .await
        .context("Failed to flush protocol request")?;

    // Read response
    let mut reader = BufReader::new(transport);
    let mut response_line = String::new();
    reader
        .read_line(&mut response_line)
        .await
        .context("Failed to read protocol response")?;

    let response =
        ProtocolResponse::from_wire(&response_line).context("Failed to parse protocol response")?;

    info!("Client negotiated protocol: {}", response.selected);
    Ok(response.selected)
}

/// Negotiate protocol from server side
///
/// Reads client's supported protocols and selects the best one.
///
/// # Arguments
///
/// * `transport` - The transport to negotiate over
/// * `server_supported` - Protocols supported by server (in preference order)
///
/// # Returns
///
/// Selected protocol, or None if no negotiation request received (use default)
pub async fn negotiate_server<T>(
    transport: &mut T,
    server_supported: Vec<IpcProtocol>,
) -> Result<Option<IpcProtocol>>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    // Peek at first line to see if it's a protocol negotiation request
    let mut reader = BufReader::new(transport);
    let mut first_line = String::new();

    // Try to read first line (non-blocking peek would be better, but this works)
    match tokio::time::timeout(
        std::time::Duration::from_millis(100),
        reader.read_line(&mut first_line),
    )
    .await
    {
        Ok(Ok(0)) => {
            // EOF - no negotiation
            Ok(None)
        }
        Ok(Ok(_)) => {
            // Got a line, check if it's a protocol request
            if first_line.trim().starts_with("PROTOCOLS: ") {
                // Parse request
                let request = ProtocolRequest::from_wire(&first_line)
                    .context("Failed to parse protocol request")?;

                // Select best protocol using preference matching
                let selected = select_protocol(&request.supported, &server_supported);

                // Send response
                let response = ProtocolResponse::new(selected);
                let response_line = response.to_wire();

                reader
                    .get_mut()
                    .write_all(response_line.as_bytes())
                    .await
                    .context("Failed to send protocol response")?;
                reader
                    .get_mut()
                    .flush()
                    .await
                    .context("Failed to flush protocol response")?;

                info!("Server negotiated protocol: {}", selected);
                Ok(Some(selected))
            } else {
                // Not a protocol request - this is a regular RPC request
                // Assume JSON-RPC (default) and let the handler deal with it
                warn!("No protocol negotiation, assuming JSON-RPC");
                Ok(None)
            }
        }
        Ok(Err(e)) => {
            warn!("Error reading protocol negotiation: {}", e);
            Ok(None)
        }
        Err(_) => {
            // Timeout - no negotiation request
            Ok(None)
        }
    }
}

/// Select best protocol from client's supported list
///
/// Chooses the first protocol from the client's list that the server also supports.
///
/// # Arguments
///
/// * `client_supported` - Client's supported protocols (in preference order)
/// * `server_supported` - Server's supported protocols (in preference order)
///
/// # Returns
///
/// Selected protocol
pub fn select_protocol(
    client_supported: &[IpcProtocol],
    server_supported: &[IpcProtocol],
) -> IpcProtocol {
    // Find first client protocol that server supports
    for client_proto in client_supported {
        if server_supported.contains(client_proto) {
            return *client_proto;
        }
    }

    // Fallback to JSON-RPC (always supported)
    IpcProtocol::JsonRpc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_request_wire_format() {
        let request = ProtocolRequest::new(vec![IpcProtocol::JsonRpc]);
        assert_eq!(request.to_wire(), "PROTOCOLS: jsonrpc\n");

        #[cfg(feature = "tarpc-rpc")]
        {
            let request = ProtocolRequest::new(vec![IpcProtocol::Tarpc, IpcProtocol::JsonRpc]);
            assert_eq!(request.to_wire(), "PROTOCOLS: tarpc,jsonrpc\n");
        }
    }

    #[test]
    fn test_protocol_request_parse() {
        let request = ProtocolRequest::from_wire("PROTOCOLS: jsonrpc\n").unwrap();
        assert_eq!(request.supported, vec![IpcProtocol::JsonRpc]);

        #[cfg(feature = "tarpc-rpc")]
        {
            let request = ProtocolRequest::from_wire("PROTOCOLS: tarpc,jsonrpc\n").unwrap();
            assert_eq!(
                request.supported,
                vec![IpcProtocol::Tarpc, IpcProtocol::JsonRpc]
            );
        }
    }

    #[test]
    fn test_protocol_response_wire_format() {
        let response = ProtocolResponse::new(IpcProtocol::JsonRpc);
        assert_eq!(response.to_wire(), "PROTOCOL: jsonrpc\n");

        #[cfg(feature = "tarpc-rpc")]
        {
            let response = ProtocolResponse::new(IpcProtocol::Tarpc);
            assert_eq!(response.to_wire(), "PROTOCOL: tarpc\n");
        }
    }

    #[test]
    fn test_protocol_response_parse() {
        let response = ProtocolResponse::from_wire("PROTOCOL: jsonrpc\n").unwrap();
        assert_eq!(response.selected, IpcProtocol::JsonRpc);

        #[cfg(feature = "tarpc-rpc")]
        {
            let response = ProtocolResponse::from_wire("PROTOCOL: tarpc\n").unwrap();
            assert_eq!(response.selected, IpcProtocol::Tarpc);
        }
    }

    #[test]
    fn test_select_protocol() {
        let client = vec![IpcProtocol::JsonRpc];
        let server = vec![IpcProtocol::JsonRpc];
        assert_eq!(select_protocol(&client, &server), IpcProtocol::JsonRpc);

        #[cfg(feature = "tarpc-rpc")]
        {
            let client = vec![IpcProtocol::Tarpc, IpcProtocol::JsonRpc];
            let server = vec![IpcProtocol::Tarpc, IpcProtocol::JsonRpc];
            assert_eq!(select_protocol(&client, &server), IpcProtocol::Tarpc);

            // Client prefers tarpc, server only supports JSON-RPC
            let client = vec![IpcProtocol::Tarpc, IpcProtocol::JsonRpc];
            let server = vec![IpcProtocol::JsonRpc];
            assert_eq!(select_protocol(&client, &server), IpcProtocol::JsonRpc);

            // No common protocol (fallback to JSON-RPC)
            let client = vec![IpcProtocol::Tarpc];
            let server = vec![IpcProtocol::JsonRpc];
            assert_eq!(select_protocol(&client, &server), IpcProtocol::JsonRpc);
        }
    }

    #[test]
    fn test_all_supported_request() {
        let request = ProtocolRequest::all_supported();
        assert!(!request.supported.is_empty());
        assert!(request.supported.contains(&IpcProtocol::JsonRpc));

        #[cfg(feature = "tarpc-rpc")]
        assert!(request.supported.contains(&IpcProtocol::Tarpc));
    }
}
