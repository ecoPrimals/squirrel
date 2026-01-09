//! tarpc Client Implementation
//!
//! High-performance RPC client for connecting to remote Squirrel instances.
//!
//! The `#[tarpc::service]` macro generates a client type automatically.
//! This module provides helper functions for connection management.

use super::tarpc_service::*;
use crate::error::PrimalError;
use std::net::SocketAddr;
use tarpc::client;
use tokio::net::TcpStream;
use tokio_serde::formats::Bincode;
use tracing::info;

/// Connect to a remote Squirrel instance via tarpc
///
/// Returns the generated client type that implements the SquirrelRpc trait
pub async fn connect(addr: SocketAddr) -> Result<SquirrelRpcClient, PrimalError> {
    info!("🔌 Connecting to Squirrel instance at {}", addr);

    let stream = TcpStream::connect(addr)
        .await
        .map_err(|e| PrimalError::NetworkError(format!("Failed to connect: {}", e)))?;

    let transport = tarpc::serde_transport::new(stream, Bincode::default());

    let client = SquirrelRpcClient::new(client::Config::default(), transport).spawn();

    info!("✅ Connected to Squirrel instance at {}", addr);

    Ok(client)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_types() {
        // Test that types are properly defined
        let request = TarpcQueryRequest {
            prompt: "test".to_string(),
            provider: None,
            model: None,
            max_tokens: None,
            temperature: None,
        };

        assert_eq!(request.prompt, "test");
    }
}
