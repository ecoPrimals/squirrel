use crate::transport::{tcp, websocket, stdio, memory};
use crate::transport;
use super::error::TransportError;
use std::sync::Arc;

/// Convert from old TransportConfig to new TcpTransportConfig
pub fn convert_to_new_tcp_config(
    old_config: &super::TransportConfig
) -> tcp::TcpTransportConfig {
    let mut config = tcp::TcpTransportConfig::default();
    
    // Map address settings
    if let Some(addr) = &old_config.remote_address {
        config.remote_address = addr.clone();
    }
    
    if let Some(addr) = &old_config.local_bind_address {
        config.local_bind_address = Some(addr.clone());
    }
    
    // Map timeout settings
    config.connection_timeout = old_config.connection_timeout_ms;
    
    // Map reconnection settings
    config.max_reconnect_attempts = old_config.retry_count;
    config.reconnect_delay_ms = old_config.retry_delay_ms;
    
    // Map keep-alive settings
    if let Some(keep_alive) = old_config.keep_alive_interval_ms {
        config.keep_alive_interval = Some(keep_alive);
    }
    
    // Map encryption settings
    if old_config.encryption_enabled {
        // Set the encryption format directly without using an Option<String>
        config.encryption = match old_config.encryption_format {
            crate::types::EncryptionFormat::Aes256Gcm => crate::types::EncryptionFormat::Aes256Gcm,
            crate::types::EncryptionFormat::ChaCha20Poly1305 => crate::types::EncryptionFormat::ChaCha20Poly1305,
            _ => crate::types::EncryptionFormat::None,
        };
    }
    
    // Map message size limit
    config.max_message_size = old_config.max_message_size;
    
    // Note: TcpTransportConfig doesn't have read_buffer_size or write_buffer_size fields
    // so we don't map those from the old config
    
    config
}

/// Convert from old TransportConfig to new WebSocketConfig
pub fn convert_to_new_websocket_config(
    old_config: &super::TransportConfig
) -> websocket::WebSocketConfig {
    let mut config = websocket::WebSocketConfig::default();
    
    // Map address settings
    if let Some(addr) = &old_config.remote_address {
        // Convert TCP addr to WS URL
        let addr_str = addr.clone();
        if !addr_str.starts_with("ws://") && !addr_str.starts_with("wss://") {
            config.url = format!("ws://{}", addr_str);
        } else {
            config.url = addr_str;
        }
    }
    
    // Map timeout settings
    config.connection_timeout = old_config.connection_timeout_ms;
    
    // Map reconnection settings
    config.max_reconnect_attempts = old_config.retry_count;
    config.reconnect_delay_ms = old_config.retry_delay_ms;
    
    // Map keep-alive settings
    if let Some(keep_alive) = old_config.keep_alive_interval_ms {
        config.ping_interval = Some(keep_alive);
    }
    
    // Map encryption settings - WebSocketConfig uses TLS instead
    config.encryption = if old_config.encryption_enabled {
        match old_config.encryption_format {
            crate::types::EncryptionFormat::Aes256Gcm => crate::types::EncryptionFormat::Aes256Gcm,
            crate::types::EncryptionFormat::ChaCha20Poly1305 => crate::types::EncryptionFormat::ChaCha20Poly1305,
            _ => crate::types::EncryptionFormat::None,
        }
    } else {
        crate::types::EncryptionFormat::None
    };
    
    // Map message size limit
    config.max_message_size = old_config.max_message_size;
    
    config
}

/// Create a new TCP transport from the old transport API
pub fn create_tcp_transport_from_old(
    old_transport: &super::Transport
) -> Result<Arc<dyn transport::Transport>, TransportError> {
    let config = convert_to_new_tcp_config(&old_transport.config);
    let new_transport = Arc::new(tcp::TcpTransport::new(config));
    
    Ok(new_transport)
}

/// Create a WebSocket transport from the old transport API
pub fn create_websocket_transport_from_old(
    old_transport: &super::Transport
) -> Result<Arc<dyn transport::Transport>, TransportError> {
    let config = convert_to_new_websocket_config(&old_transport.config);
    let new_transport = Arc::new(websocket::WebSocketTransport::new(config));
    
    Ok(new_transport)
}

/// Creates a new standard input/output transport that sends and receives data
/// through standard input/output streams.
pub fn create_stdio_transport() -> Arc<dyn transport::Transport> {
    let config = stdio::StdioConfig::default();
    Arc::new(stdio::StdioTransport::new(config))
}

/// Creates a memory transport pair that can be used for in-process communication
/// 
/// WARNING: This function returns a pair of Arc<dyn Transport> instances, but there is a known
/// issue with the Transport trait: the receive_message method requires a mutable reference,
/// which cannot be easily obtained through an Arc. This means that trying to call receive_message
/// on the returned transports will fail at compile time.
/// 
/// In a future MCP version, the Transport trait should be updated to use interior mutability
/// so that all methods can operate on &self references, making it compatible with Arc wrapping.
pub fn create_memory_transport_pair() -> (
    Arc<dyn transport::Transport>,
    Arc<dyn transport::Transport>
) {
    let (transport1, transport2) = memory::MemoryChannel::create_pair();
    
    (
        Arc::new(transport1) as Arc<dyn transport::Transport>,
        Arc::new(transport2) as Arc<dyn transport::Transport>
    )
}

/// Creates a single memory transport for testing purposes
/// 
/// WARNING: This function returns a pair of Arc<dyn Transport> instances, but there is a known
/// issue with the Transport trait: the receive_message method requires a mutable reference,
/// which cannot be easily obtained through an Arc. This means that trying to call receive_message
/// on the returned transports will fail at compile time.
/// 
/// In a future MCP version, the Transport trait should be updated to use interior mutability
/// so that all methods can operate on &self references, making it compatible with Arc wrapping.
pub fn create_memory_transport() -> (
    Arc<dyn transport::Transport>,
    Arc<dyn transport::Transport>
) {
    // Use the create_pair function and then wrap in Arc
    // Note: This will compile but the receive_message method cannot be called through the Arc
    let (transport1, transport2) = memory::MemoryChannel::create_pair();
    
    (
        Arc::new(transport1) as Arc<dyn transport::Transport>,
        Arc::new(transport2) as Arc<dyn transport::Transport>
    )
}

/// Gets the remote address from an old transport
pub fn get_remote_address_from_old(
    old_transport: &super::Transport
) -> Result<String, TransportError> {
    if let Some(addr) = &old_transport.config.remote_address {
        Ok(addr.clone())
    } else {
        Err(TransportError::ConnectionFailed(
            "No remote address configured".to_string()
        ))
    }
}

/// Checks if the compatibility mode between old and new transport implementations is available
///
/// This function determines whether the system can support compatibility bridges between
/// the legacy transport system and the new transport implementation. Currently, this
/// always returns true as the compatibility layer is a core feature.
///
/// # Returns
///
/// * `true` - Compatibility mode is available
/// * `false` - Compatibility mode is not available
pub fn is_compat_available() -> bool {
    true
} 