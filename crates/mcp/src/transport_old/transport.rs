/// Transport implementation for the MCP protocol
///
/// # DEPRECATION NOTICE
///
/// This struct is deprecated and will be removed in a future release.
/// Please migrate to the new Transport trait implementations in the
/// `transport` module:
///
/// - For TCP connections, use `transport::tcp::TcpTransport`
/// - For WebSocket connections, use `transport::websocket::WebSocketTransport`
/// - For in-memory testing, use `transport::memory::MemoryTransport`
/// - For stdio based communication, use `transport::stdio::StdioTransport`
///
/// See the `docs/migration/TRANSPORT_MIGRATION_GUIDE.md` for detailed migration instructions.
///
/// To help with migration, you can use the compatibility layer functions:
/// ```
/// use mcp::transport_old::compat;
///
/// // Convert old config to new config
/// let new_config = compat::convert_to_new_tcp_config(&old_config);
///
/// // Create new transport from old transport
/// let new_transport = compat::create_new_tcp_transport(&old_transport)?;
/// ```
#[deprecated(
    since = "0.2.0",
    note = "Use the new transport module instead. Will be removed in a future release."
)]
pub struct Transport {
    /// Configuration for this transport
    pub config: TransportConfig,
}

impl Transport {
    /// Create a new transport with the given configuration
    pub fn new(config: TransportConfig) -> Self {
        Self { config }
    }
    
    /// Connect to the remote endpoint
    pub async fn connect(&self) -> Result<(), super::error::TransportError> {
        // This is a stub implementation - it's deprecated
        Ok(())
    }
    
    /// Send a message through the transport
    pub async fn send_message(&self, _message: crate::message::Message) -> Result<(), super::error::TransportError> {
        // This is a stub implementation - it's deprecated
        Ok(())
    }
    
    /// Receive a message from the transport
    pub async fn receive_message(&self) -> Result<crate::message::Message, super::error::TransportError> {
        // This is a stub implementation - it's deprecated
        Ok(crate::message::MessageBuilder::new()
            .with_message_type("response")
            .with_payload(serde_json::json!({"status": "ok"}))
            .build())
    }
    
    /// Check if the transport is connected
    pub async fn is_connected(&self) -> Result<bool, super::error::TransportError> {
        // This is a stub implementation - it's deprecated
        Ok(true)
    }
    
    /// Disconnect the transport
    pub async fn disconnect(&self) -> Result<(), super::error::TransportError> {
        // This is a stub implementation - it's deprecated
        Ok(())
    }
}

/// Configuration for the Transport
///
/// # DEPRECATION NOTICE
///
/// This struct is deprecated and will be removed in a future release.
/// Use the specific transport configuration types in the new `transport` module:
///
/// - `transport::tcp::TcpTransportConfig`
/// - `transport::websocket::WebSocketTransportConfig`
/// - `transport::stdio::StdioTransportConfig`
///
/// See the `docs/migration/TRANSPORT_MIGRATION_GUIDE.md` for detailed migration instructions.
#[deprecated(
    since = "0.2.0",
    note = "Use the new transport module instead. Will be removed in a future release."
)]
pub struct TransportConfig {
    /// Remote address to connect to
    pub remote_address: Option<String>,
    
    /// Local address to bind to
    pub local_bind_address: Option<String>,
    
    /// Connection timeout in milliseconds
    pub connection_timeout_ms: u64,
    
    /// Number of retry attempts
    pub retry_count: u32,
    
    /// Delay between retry attempts in milliseconds
    pub retry_delay_ms: u64,
    
    /// Maximum message size in bytes
    pub max_message_size: usize,
    
    /// Buffer size for the transport
    pub buffer_size: Option<usize>,
    
    /// Whether encryption is enabled
    pub encryption_enabled: bool,
    
    /// Encryption format to use
    pub encryption_format: crate::types::EncryptionFormat,
    
    /// Keep alive interval in milliseconds
    pub keep_alive_interval_ms: Option<u64>,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            remote_address: None,
            local_bind_address: None,
            connection_timeout_ms: 30000,
            retry_count: 3,
            retry_delay_ms: 1000,
            max_message_size: 10 * 1024 * 1024, // 10MB
            buffer_size: Some(1024),
            encryption_enabled: false,
            encryption_format: crate::types::EncryptionFormat::None,
            keep_alive_interval_ms: Some(60000),
        }
    }
}

/// State of the Transport
///
/// # DEPRECATION NOTICE
///
/// This enum is deprecated and will be removed in a future release.
/// The new `Transport` trait implementations use their own state enums
/// or provide methods like `is_connected()` to check the connection status.
///
/// See the `docs/migration/TRANSPORT_MIGRATION_GUIDE.md` for detailed migration instructions.
#[deprecated(
    since = "0.2.0",
    note = "Use the new transport module instead. Will be removed in a future release."
)]
pub enum TransportState {
    /// Not connected
    Disconnected,
    
    /// In the process of connecting
    Connecting,
    
    /// Connected and ready to send/receive
    Connected,
    
    /// In the process of disconnecting
    Disconnecting,
    
    /// Connection has failed
    Failed(String),
} 