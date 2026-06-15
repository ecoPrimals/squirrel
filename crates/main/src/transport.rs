// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Transport abstraction — sourDough `TransportEndpoint` wire-compatible.
//!
//! Squirrel implements its own transport types (primal self-knowledge only,
//! no cross-primal deps). Wire format is identical to sourDough's canonical
//! standard so launchers/Tower Atomic can inject endpoints as JSON.
//!
//! # Usage
//!
//! ```ignore
//! use squirrel::transport::{TransportEndpoint, connect_transport};
//!
//! let ep = TransportEndpoint::uds("/run/user/1000/biomeos/beardog.sock");
//! let stream = connect_transport(&ep).await?;
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// Structured transport endpoint — wire-compatible with sourDough `TransportEndpoint`.
///
/// ```json
/// { "transport": "uds", "path": "/run/membrane/beardog.sock" }
/// { "transport": "tcp", "host": "192.168.1.144", "port": 7700 }
/// { "transport": "mesh_relay", "peer_id": "strand-gate", "capability": "security" }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(tag = "transport")]
pub enum TransportEndpoint {
    /// Unix Domain Socket — local primal on same host.
    #[serde(rename = "uds")]
    Uds {
        /// Filesystem path to the socket.
        path: String,
    },

    /// TCP — direct network connection.
    #[serde(rename = "tcp")]
    Tcp {
        /// Host address.
        host: String,
        /// TCP port.
        port: u16,
    },

    /// Mesh relay — reachable via Songbird's mesh network.
    #[serde(rename = "mesh_relay")]
    MeshRelay {
        /// Mesh peer identifier.
        peer_id: String,
        /// Capability being resolved.
        capability: String,
    },
}

impl TransportEndpoint {
    /// Construct a UDS endpoint.
    #[must_use]
    pub fn uds(path: impl Into<String>) -> Self {
        Self::Uds { path: path.into() }
    }

    /// Construct a TCP endpoint.
    #[must_use]
    pub fn tcp(host: impl Into<String>, port: u16) -> Self {
        Self::Tcp {
            host: host.into(),
            port,
        }
    }

    /// Whether this endpoint is local (same-host, no network hop).
    #[must_use]
    pub fn is_local(&self) -> bool {
        match self {
            Self::Uds { .. } => true,
            Self::Tcp { host, .. } => host == "127.0.0.1" || host == "::1" || host == "localhost",
            Self::MeshRelay { .. } => false,
        }
    }

    /// Build from a URI-style string (convenience for endpoint env vars).
    ///
    /// Supports: `/path/to/socket` (UDS), `host:port` (TCP), or JSON.
    #[must_use]
    pub fn from_uri(uri: &str) -> Option<Self> {
        if uri.starts_with('{') {
            serde_json::from_str(uri).ok()
        } else if uri.starts_with('/') || uri.starts_with('@') {
            Some(Self::Uds {
                path: uri.to_owned(),
            })
        } else if let Some((host, port_str)) = uri.rsplit_once(':') {
            let port = port_str.parse::<u16>().ok()?;
            Some(Self::Tcp {
                host: host.to_owned(),
                port,
            })
        } else {
            None
        }
    }
}

impl fmt::Display for TransportEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Uds { path } => write!(f, "unix://{path}"),
            Self::Tcp { host, port } => write!(f, "tcp://{host}:{port}"),
            Self::MeshRelay {
                peer_id,
                capability,
            } => write!(f, "mesh://{peer_id}/{capability}"),
        }
    }
}

// ---------------------------------------------------------------------------
// TransportStream — type-erased connected stream
// ---------------------------------------------------------------------------

/// Transport-agnostic connected stream (implements `AsyncRead + AsyncWrite`).
#[derive(Debug)]
pub enum TransportStream {
    /// Connected Unix domain socket.
    Unix(tokio::net::UnixStream),
    /// Connected TCP stream.
    Tcp(tokio::net::TcpStream),
}

impl AsyncRead for TransportStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        match self.get_mut() {
            Self::Unix(s) => Pin::new(s).poll_read(cx, buf),
            Self::Tcp(s) => Pin::new(s).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for TransportStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match self.get_mut() {
            Self::Unix(s) => Pin::new(s).poll_write(cx, buf),
            Self::Tcp(s) => Pin::new(s).poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.get_mut() {
            Self::Unix(s) => Pin::new(s).poll_flush(cx),
            Self::Tcp(s) => Pin::new(s).poll_flush(cx),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.get_mut() {
            Self::Unix(s) => Pin::new(s).poll_shutdown(cx),
            Self::Tcp(s) => Pin::new(s).poll_shutdown(cx),
        }
    }
}

// ---------------------------------------------------------------------------
// connect_transport — connect to a resolved endpoint
// ---------------------------------------------------------------------------

/// Connect to a service via its resolved [`TransportEndpoint`].
///
/// Returns a [`TransportStream`] ready for JSON-RPC framing.
///
/// # Errors
///
/// Returns `io::Error` on connection failure or timeout.
/// `MeshRelay` endpoints are not directly connectable (require Songbird routing).
pub async fn connect_transport(endpoint: &TransportEndpoint) -> io::Result<TransportStream> {
    connect_transport_with_timeout(endpoint, DEFAULT_CONNECT_TIMEOUT).await
}

/// Connect with an explicit timeout.
///
/// # Errors
///
/// Returns `io::Error` on connection failure, timeout, or unsupported endpoint type.
pub async fn connect_transport_with_timeout(
    endpoint: &TransportEndpoint,
    timeout: Duration,
) -> io::Result<TransportStream> {
    match endpoint {
        TransportEndpoint::Uds { path } => {
            let stream = tokio::time::timeout(timeout, tokio::net::UnixStream::connect(path)).await;
            match stream {
                Ok(Ok(s)) => Ok(TransportStream::Unix(s)),
                Ok(Err(e)) => Err(e),
                Err(_) => Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    format!("UDS connect timeout: {path}"),
                )),
            }
        }
        TransportEndpoint::Tcp { host, port } => {
            let addr = format!("{host}:{port}");
            let stream = tokio::time::timeout(timeout, tokio::net::TcpStream::connect(&addr)).await;
            match stream {
                Ok(Ok(s)) => Ok(TransportStream::Tcp(s)),
                Ok(Err(e)) => Err(e),
                Err(_) => Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    format!("TCP connect timeout: {addr}"),
                )),
            }
        }
        TransportEndpoint::MeshRelay {
            peer_id,
            capability,
        } => Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!(
                "mesh_relay not directly connectable (peer={peer_id}, cap={capability}) — route via Songbird"
            ),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde_uds() {
        let ep = TransportEndpoint::uds("/run/user/1000/biomeos/squirrel.sock");
        let json = serde_json::to_string(&ep).expect("serialize");
        assert_eq!(
            json,
            r#"{"transport":"uds","path":"/run/user/1000/biomeos/squirrel.sock"}"#
        );
        let decoded: TransportEndpoint = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded, ep);
    }

    #[test]
    fn test_serde_tcp() {
        let ep = TransportEndpoint::tcp("192.168.1.173", 7700);
        let json = serde_json::to_string(&ep).expect("serialize");
        assert_eq!(
            json,
            r#"{"transport":"tcp","host":"192.168.1.173","port":7700}"#
        );
        let decoded: TransportEndpoint = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded, ep);
    }

    #[test]
    fn test_serde_mesh_relay() {
        let json = r#"{"transport":"mesh_relay","peer_id":"strand-gate","capability":"security"}"#;
        let ep: TransportEndpoint = serde_json::from_str(json).expect("deserialize");
        assert!(matches!(ep, TransportEndpoint::MeshRelay { .. }));
        assert!(!ep.is_local());
    }

    #[test]
    fn test_is_local() {
        assert!(TransportEndpoint::uds("/tmp/test.sock").is_local());
        assert!(TransportEndpoint::tcp("127.0.0.1", 8080).is_local());
        assert!(TransportEndpoint::tcp("localhost", 8080).is_local());
        assert!(!TransportEndpoint::tcp("192.168.1.1", 8080).is_local());
    }

    #[test]
    fn test_from_uri_json() {
        let ep = TransportEndpoint::from_uri(r#"{"transport":"uds","path":"/tmp/test.sock"}"#);
        assert_eq!(ep, Some(TransportEndpoint::uds("/tmp/test.sock")));
    }

    #[test]
    fn test_from_uri_path() {
        let ep = TransportEndpoint::from_uri("/run/user/1000/biomeos/squirrel.sock");
        assert_eq!(
            ep,
            Some(TransportEndpoint::uds(
                "/run/user/1000/biomeos/squirrel.sock"
            ))
        );
    }

    #[test]
    fn test_from_uri_host_port() {
        let ep = TransportEndpoint::from_uri("192.168.1.173:7700");
        assert_eq!(ep, Some(TransportEndpoint::tcp("192.168.1.173", 7700)));
    }

    #[test]
    fn test_display() {
        assert_eq!(
            TransportEndpoint::uds("/tmp/test.sock").to_string(),
            "unix:///tmp/test.sock"
        );
        assert_eq!(
            TransportEndpoint::tcp("10.0.0.1", 9100).to_string(),
            "tcp://10.0.0.1:9100"
        );
    }

    #[tokio::test]
    async fn test_connect_mesh_relay_unsupported() {
        let ep = TransportEndpoint::MeshRelay {
            peer_id: "test".into(),
            capability: "security".into(),
        };
        let result = connect_transport(&ep).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::Unsupported);
    }

    #[tokio::test]
    async fn test_connect_uds_nonexistent() {
        let ep = TransportEndpoint::uds("/tmp/nonexistent-squirrel-test-12345.sock");
        let result = connect_transport(&ep).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_connect_tcp_refused() {
        let ep = TransportEndpoint::tcp("127.0.0.1", 1);
        let result = connect_transport(&ep).await;
        assert!(result.is_err());
    }
}
