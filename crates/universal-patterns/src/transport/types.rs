// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Core types and enums for universal transport abstraction
#![allow(
    dead_code,
    reason = "Transport and discovery types used via serde/runtime wiring"
)]

use bytes::Bytes;
use std::path::PathBuf;

/// IPC endpoint discovered at runtime
///
/// Represents different types of IPC endpoints that can be discovered
/// dynamically without compile-time platform knowledge.
///
/// ## Isomorphic Discovery
///
/// This enum enables clients to discover the actual transport being used
/// by a server, whether it's Unix sockets (optimal) or TCP fallback.
#[derive(Debug, Clone)]
pub enum IpcEndpoint {
    /// Unix domain socket (path or abstract)
    #[cfg(unix)]
    UnixSocket(PathBuf),

    /// TCP local address (fallback)
    TcpLocal(std::net::SocketAddr),

    /// Named pipe (Windows)
    #[cfg(windows)]
    NamedPipe(String),
}

/// Transport type enumeration
///
/// Used for explicit transport selection or preference specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TransportType {
    /// Unix domain socket (abstract namespace on Linux)
    UnixAbstract,

    /// Unix domain socket (filesystem)
    UnixFilesystem,

    /// Named pipe (Windows)
    NamedPipe,

    /// TCP connection
    Tcp,

    /// In-process channel
    InProcess,
}

/// Configuration for transport connection
///
/// Provides hints for transport selection and connection behavior.
#[derive(Debug, Clone)]
pub struct TransportConfig {
    /// Preferred transport type (None = automatic)
    pub preferred_transport: Option<TransportType>,

    /// Enable automatic fallback on connection failure
    pub enable_fallback: bool,

    /// Connection timeout in milliseconds
    pub timeout_ms: u64,

    /// Base directory for filesystem sockets
    pub socket_base_dir: Option<PathBuf>,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            preferred_transport: None,
            enable_fallback: true,
            timeout_ms: 5000,
            socket_base_dir: None,
        }
    }
}

/// Configuration for server listener
///
/// Provides configuration options for binding server sockets.
#[derive(Debug, Clone)]
pub struct ListenerConfig {
    /// Preferred transport type (None = automatic)
    pub preferred_transport: Option<TransportType>,

    /// Enable automatic fallback on bind failure
    pub enable_fallback: bool,

    /// Base directory for filesystem sockets
    pub socket_base_dir: Option<PathBuf>,

    /// Backlog size for accept queue
    pub backlog: Option<u32>,

    /// Unix socket permissions (octal, e.g., 0o666)
    #[cfg(unix)]
    pub unix_permissions: Option<u32>,
}

impl Default for ListenerConfig {
    fn default() -> Self {
        Self {
            preferred_transport: None,
            enable_fallback: true,
            socket_base_dir: None,
            backlog: Some(128),
            #[cfg(unix)]
            unix_permissions: Some(0o666),
        }
    }
}

/// Remote address information
///
/// Represents the remote peer address for an accepted connection.
#[derive(Debug, Clone)]
pub enum RemoteAddr {
    /// Unix socket (path or abstract)
    #[cfg(unix)]
    Unix(Option<std::os::unix::net::SocketAddr>),

    /// Named pipe (Windows)
    #[cfg(windows)]
    NamedPipe(String),

    /// TCP address
    Tcp(std::net::SocketAddr),

    /// In-process
    InProcess,
}

/// In-process transport for testing and embedded scenarios
///
/// Provides zero-overhead communication within the same process using
/// in-memory channels. This is useful for testing, embedded systems, and
/// scenarios where IPC overhead is not desired.
#[derive(Debug)]
pub struct InProcessTransport {
    /// Sender half of the channel
    sender: tokio::sync::mpsc::UnboundedSender<Bytes>,
    /// Receiver half of the channel
    receiver: tokio::sync::mpsc::UnboundedReceiver<Bytes>,
    /// Buffer for reading data
    read_buffer: std::sync::Mutex<Vec<u8>>,
}

impl InProcessTransport {
    /// Create a new in-process transport instance
    ///
    /// Creates a pair of connected channels for bidirectional communication.
    pub(crate) fn new() -> Self {
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        Self {
            sender,
            receiver,
            read_buffer: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Create a connected pair of transports
    ///
    /// Returns (client_transport, server_transport) for testing scenarios.
    pub fn pair() -> (Self, Self) {
        let (tx1, rx1) = tokio::sync::mpsc::unbounded_channel();
        let (tx2, rx2) = tokio::sync::mpsc::unbounded_channel();

        (
            Self {
                sender: tx1,
                receiver: rx2,
                read_buffer: std::sync::Mutex::new(Vec::new()),
            },
            Self {
                sender: tx2,
                receiver: rx1,
                read_buffer: std::sync::Mutex::new(Vec::new()),
            },
        )
    }

    /// Send data through the channel
    pub(crate) fn send(&self, data: impl Into<Bytes>) -> Result<(), std::io::Error> {
        self.sender.send(data.into()).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "In-process transport channel closed",
            )
        })
    }

    /// Try to receive data from the channel (non-blocking)
    pub(crate) fn try_recv(&mut self) -> Result<Option<Bytes>, std::io::Error> {
        match self.receiver.try_recv() {
            Ok(data) => Ok(Some(data)),
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => Ok(None),
            Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "In-process transport channel closed",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_transport_config_default() {
        let config = TransportConfig::default();
        assert_eq!(config.preferred_transport, None);
        assert!(config.enable_fallback);
        assert_eq!(config.timeout_ms, 5000);
        assert_eq!(config.socket_base_dir, None);
    }

    #[test]
    fn test_transport_config_custom_values() {
        let socket_dir = PathBuf::from("/tmp/sockets");
        let config = TransportConfig {
            preferred_transport: Some(TransportType::UnixFilesystem),
            enable_fallback: false,
            timeout_ms: 10000,
            socket_base_dir: Some(socket_dir.clone()),
        };
        assert_eq!(
            config.preferred_transport,
            Some(TransportType::UnixFilesystem)
        );
        assert!(!config.enable_fallback);
        assert_eq!(config.timeout_ms, 10000);
        assert_eq!(config.socket_base_dir, Some(socket_dir));
    }

    #[test]
    fn test_transport_type_variants() {
        // Test all enum variants exist and can be compared
        assert_eq!(TransportType::UnixAbstract, TransportType::UnixAbstract);
        assert_eq!(TransportType::UnixFilesystem, TransportType::UnixFilesystem);
        assert_eq!(TransportType::NamedPipe, TransportType::NamedPipe);
        assert_eq!(TransportType::Tcp, TransportType::Tcp);
        assert_eq!(TransportType::InProcess, TransportType::InProcess);

        // Test variants are different from each other
        assert_ne!(TransportType::UnixAbstract, TransportType::UnixFilesystem);
        assert_ne!(TransportType::Tcp, TransportType::InProcess);
    }

    #[test]
    fn test_transport_type_debug() {
        // Test that all variants can be formatted for debugging
        let types = vec![
            TransportType::UnixAbstract,
            TransportType::UnixFilesystem,
            TransportType::NamedPipe,
            TransportType::Tcp,
            TransportType::InProcess,
        ];

        for transport_type in types {
            let debug_str = format!("{:?}", transport_type);
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_in_process_transport_pair_creation() {
        let (client, server) = InProcessTransport::pair();

        // Both transports should be created successfully
        // We can verify by checking they can send/receive
        assert!(client.send(vec![1, 2, 3]).is_ok());
        assert!(server.send(vec![4, 5, 6]).is_ok());
    }

    #[test]
    fn test_in_process_transport_message_passing() {
        let (mut client, mut server) = InProcessTransport::pair();

        // Send message from client to server
        let message = vec![1, 2, 3, 4, 5];
        assert!(client.send(message.clone()).is_ok());

        // Receive message on server side
        let received = server.try_recv().expect("should succeed");
        assert_eq!(received.as_deref(), Some(message.as_slice()));

        // Send message from server to client
        let response = vec![6, 7, 8];
        assert!(server.send(response.clone()).is_ok());

        // Receive response on client side
        let received_response = client.try_recv().expect("should succeed");
        assert_eq!(received_response.as_deref(), Some(response.as_slice()));
    }

    #[test]
    fn test_in_process_transport_empty_channel() {
        let (mut client, _server) = InProcessTransport::pair();

        // Try to receive when channel is empty
        let result = client.try_recv().expect("should succeed");
        assert_eq!(result, None);
    }

    #[test]
    fn test_in_process_transport_multiple_messages() {
        let (sender, mut receiver) = InProcessTransport::pair();

        // Send multiple messages
        for i in 0..5 {
            let message = vec![i, i + 1, i + 2];
            assert!(sender.send(message).is_ok());
        }

        // Receive all messages in order
        for i in 0..5 {
            let msg = receiver.try_recv().expect("should succeed");
            assert_eq!(msg.as_deref(), Some([i, i + 1, i + 2].as_slice()));
        }

        // Channel should be empty now
        assert_eq!(receiver.try_recv().expect("should succeed"), None);
    }

    #[test]
    fn test_in_process_transport_large_message() {
        let (client, mut server) = InProcessTransport::pair();

        // Send a large message
        let large_message: Vec<u8> = (0..10000)
            .map(|i| u8::try_from(i % 256).expect("mod 256 fits u8"))
            .collect();
        assert!(client.send(large_message.clone()).is_ok());

        // Receive the large message
        let msg = server.try_recv().expect("should succeed");
        assert_eq!(msg.as_deref(), Some(large_message.as_slice()));
    }

    #[test]
    fn test_in_process_transport_error_display() {
        let (client, mut receiver) = InProcessTransport::pair();

        // Drop the sender to simulate broken pipe
        drop(client);

        // Try to receive from closed channel
        let result = receiver.try_recv();
        assert!(result.is_err());

        // Verify error kind and message
        let error = result.unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::BrokenPipe);
        assert!(
            error
                .to_string()
                .contains("In-process transport channel closed")
        );
    }

    #[test]
    fn test_in_process_transport_send_after_receiver_dropped() {
        let (sender, receiver) = InProcessTransport::pair();

        // Drop the receiver
        drop(receiver);

        // Try to send after receiver is dropped
        let result = sender.send(vec![1, 2, 3]);
        assert!(result.is_err());

        // Verify error kind and message
        let error = result.unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::BrokenPipe);
        assert!(
            error
                .to_string()
                .contains("In-process transport channel closed")
        );
    }

    #[test]
    fn test_listener_config_default() {
        let config = ListenerConfig::default();
        assert_eq!(config.preferred_transport, None);
        assert!(config.enable_fallback);
        assert_eq!(config.socket_base_dir, None);
        assert_eq!(config.backlog, Some(128));
        #[cfg(unix)]
        assert_eq!(config.unix_permissions, Some(0o666));
    }

    #[test]
    fn test_listener_config_custom_values() {
        let socket_dir = PathBuf::from("/var/run/sockets");
        let config = ListenerConfig {
            preferred_transport: Some(TransportType::UnixAbstract),
            enable_fallback: false,
            socket_base_dir: Some(socket_dir.clone()),
            backlog: Some(256),
            #[cfg(unix)]
            unix_permissions: Some(0o600),
        };

        assert_eq!(
            config.preferred_transport,
            Some(TransportType::UnixAbstract)
        );
        assert!(!config.enable_fallback);
        assert_eq!(config.socket_base_dir, Some(socket_dir));
        assert_eq!(config.backlog, Some(256));
        #[cfg(unix)]
        assert_eq!(config.unix_permissions, Some(0o600));
    }
}
