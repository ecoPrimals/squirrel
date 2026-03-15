// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Comprehensive tests for MCP transport layer core types and functionality
//!
//! This module provides thorough testing of transport types, metadata,
//! compression/encryption formats, and the SimpleTransport implementation.

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::protocol::MCPMessage;
    use std::collections::HashMap;

    // ========== TransportMetadata Tests ==========

    #[test]
    fn test_transport_metadata_creation() {
        use std::net::{IpAddr, Ipv4Addr, SocketAddr};

        let now = chrono::Utc::now();
        let mut additional = HashMap::new();
        additional.insert("key".to_string(), "value".to_string());

        let remote_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 8080);
        let local_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3000);

        let metadata = types::TransportMetadata {
            connection_id: "conn-123".to_string(),
            remote_address: Some(remote_addr),
            local_address: Some(local_addr),
            connected_at: now,
            last_activity: now,
            encryption_format: Some(crate::types::EncryptionFormat::Aes256),
            compression_format: Some(crate::types::CompressionFormat::Gzip),
            additional_info: additional.clone(),
        };

        assert_eq!(metadata.connection_id, "conn-123");
        assert_eq!(metadata.remote_address, Some(remote_addr));
        assert_eq!(metadata.local_address, Some(local_addr));
        assert_eq!(
            metadata.encryption_format,
            Some(crate::types::EncryptionFormat::Aes256)
        );
        assert_eq!(
            metadata.compression_format,
            Some(crate::types::CompressionFormat::Gzip)
        );
        assert_eq!(
            metadata.additional_info.get("key"),
            Some(&"value".to_string())
        );
    }

    #[test]
    fn test_transport_metadata_minimal() {
        let metadata = types::TransportMetadata {
            connection_id: "simple-conn".to_string(),
            remote_address: None,
            local_address: None,
            connected_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            encryption_format: None,
            compression_format: None,
            additional_info: HashMap::new(),
        };

        assert_eq!(metadata.connection_id, "simple-conn");
        assert!(metadata.remote_address.is_none());
        assert!(metadata.local_address.is_none());
        assert!(metadata.encryption_format.is_none());
        assert!(metadata.compression_format.is_none());
        assert!(metadata.additional_info.is_empty());
    }

    #[test]
    fn test_transport_metadata_with_additional_info() {
        let mut additional = HashMap::new();
        additional.insert("protocol".to_string(), "websocket".to_string());
        additional.insert("version".to_string(), "1.0".to_string());
        additional.insert("endpoint".to_string(), "/ws".to_string());

        let metadata = types::TransportMetadata {
            connection_id: "ws-conn".to_string(),
            remote_address: None,
            local_address: None,
            connected_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            encryption_format: None,
            compression_format: None,
            additional_info: additional,
        };

        assert_eq!(metadata.additional_info.len(), 3);
        assert_eq!(
            metadata.additional_info.get("protocol"),
            Some(&"websocket".to_string())
        );
        assert_eq!(
            metadata.additional_info.get("version"),
            Some(&"1.0".to_string())
        );
        assert_eq!(
            metadata.additional_info.get("endpoint"),
            Some(&"/ws".to_string())
        );
    }

    #[test]
    fn test_transport_metadata_activity_tracking() {
        let connected_time = chrono::Utc::now();
        let activity_time = connected_time + chrono::Duration::seconds(30);

        let metadata = types::TransportMetadata {
            connection_id: "tracked-conn".to_string(),
            remote_address: None,
            local_address: None,
            connected_at: connected_time,
            last_activity: activity_time,
            encryption_format: None,
            compression_format: None,
            additional_info: HashMap::new(),
        };

        assert!(metadata.last_activity > metadata.connected_at);
        let duration = metadata.last_activity - metadata.connected_at;
        assert_eq!(duration.num_seconds(), 30);
    }

    // ========== EncryptionFormat Tests ==========

    #[test]
    fn test_encryption_format_none() {
        let format = crate::types::EncryptionFormat::None;
        assert_eq!(format, crate::types::EncryptionFormat::None);
    }

    #[test]
    fn test_encryption_format_aes128() {
        let format = crate::types::EncryptionFormat::Aes128;
        assert_eq!(format, crate::types::EncryptionFormat::Aes128);
    }

    #[test]
    fn test_encryption_format_aes256() {
        let format = crate::types::EncryptionFormat::Aes256;
        assert_eq!(format, crate::types::EncryptionFormat::Aes256);
    }

    #[test]
    fn test_encryption_format_chacha20() {
        let format = crate::types::EncryptionFormat::ChaCha20;
        assert_eq!(format, crate::types::EncryptionFormat::ChaCha20);
    }

    #[test]
    fn test_encryption_format_comparison() {
        let none = crate::types::EncryptionFormat::None;
        let aes128 = crate::types::EncryptionFormat::Aes128;
        let aes256 = crate::types::EncryptionFormat::Aes256;
        let chacha = crate::types::EncryptionFormat::ChaCha20;

        assert_ne!(none, aes128);
        assert_ne!(aes128, aes256);
        assert_ne!(aes256, chacha);
        assert_eq!(aes256, crate::types::EncryptionFormat::Aes256);
    }

    // ========== CompressionFormat Tests ==========

    #[test]
    fn test_compression_format_none() {
        let format = crate::types::CompressionFormat::None;
        assert_eq!(format, crate::types::CompressionFormat::None);
    }

    #[test]
    fn test_compression_format_gzip() {
        let format = crate::types::CompressionFormat::Gzip;
        assert_eq!(format, crate::types::CompressionFormat::Gzip);
    }

    #[test]
    fn test_compression_format_zstd() {
        let format = crate::types::CompressionFormat::Zstd;
        assert_eq!(format, crate::types::CompressionFormat::Zstd);
    }

    #[test]
    fn test_compression_format_lz4() {
        let format = crate::types::CompressionFormat::Lz4;
        assert_eq!(format, crate::types::CompressionFormat::Lz4);
    }

    #[test]
    fn test_compression_format_comparison() {
        let none = crate::types::CompressionFormat::None;
        let gzip = crate::types::CompressionFormat::Gzip;
        let zstd = crate::types::CompressionFormat::Zstd;
        let lz4 = crate::types::CompressionFormat::Lz4;

        assert_ne!(none, gzip);
        assert_ne!(gzip, zstd);
        assert_ne!(zstd, lz4);
        assert_eq!(gzip, crate::types::CompressionFormat::Gzip);
    }

    // ========== SimpleTransport Tests ==========

    #[tokio::test]
    async fn test_simple_transport_send_message() {
        let transport = SimpleTransport;
        let message = MCPMessage::default();

        let result = transport.send_message(message).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_simple_transport_receive_message() {
        let transport = SimpleTransport;

        let result = transport.receive_message().await;
        assert!(result.is_ok());

        let message = result.unwrap();
        // SimpleTransport returns default message
        assert!(message.id.0.len() > 0); // Should have an ID
    }

    #[tokio::test]
    async fn test_simple_transport_connect() {
        let mut transport = SimpleTransport;

        let result = transport.connect().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_simple_transport_disconnect() {
        let transport = SimpleTransport;

        let result = transport.disconnect().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_simple_transport_is_connected() {
        let transport = SimpleTransport;

        let is_connected = transport.is_connected().await;
        assert_eq!(is_connected, true);
    }

    #[tokio::test]
    async fn test_simple_transport_get_metadata() {
        let transport = SimpleTransport;

        let metadata = transport.get_metadata().await;

        assert_eq!(metadata.connection_id, "simple");
        assert!(metadata.remote_address.is_none());
        assert!(metadata.local_address.is_none());
        assert!(metadata.encryption_format.is_none());
        assert!(metadata.compression_format.is_none());
        assert!(metadata.additional_info.is_empty());
    }

    #[tokio::test]
    async fn test_simple_transport_send_raw() {
        let transport = SimpleTransport;
        let data = b"test data";

        let result = transport.send_raw(data).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_simple_transport_send_raw_empty() {
        let transport = SimpleTransport;
        let data = b"";

        let result = transport.send_raw(data).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_simple_transport_send_raw_large() {
        let transport = SimpleTransport;
        let data = vec![0u8; 10000];

        let result = transport.send_raw(&data).await;
        assert!(result.is_ok());
    }

    // ========== Integration Tests ==========

    #[tokio::test]
    async fn test_simple_transport_full_workflow() {
        let mut transport = SimpleTransport;

        // Connect
        let connect_result = transport.connect().await;
        assert!(connect_result.is_ok());

        // Check connection
        assert!(transport.is_connected().await);

        // Get metadata
        let metadata = transport.get_metadata().await;
        assert_eq!(metadata.connection_id, "simple");

        // Send message
        let message = MCPMessage::default();
        let send_result = transport.send_message(message).await;
        assert!(send_result.is_ok());

        // Receive message
        let receive_result = transport.receive_message().await;
        assert!(receive_result.is_ok());

        // Send raw
        let raw_result = transport.send_raw(b"test").await;
        assert!(raw_result.is_ok());

        // Disconnect
        let disconnect_result = transport.disconnect().await;
        assert!(disconnect_result.is_ok());
    }

    #[tokio::test]
    async fn test_simple_transport_multiple_messages() {
        let transport = SimpleTransport;

        // Send multiple messages
        for i in 0..10 {
            let message = MCPMessage::default();
            let result = transport.send_message(message).await;
            assert!(result.is_ok(), "Failed to send message {}", i);
        }

        // Receive multiple messages
        for i in 0..10 {
            let result = transport.receive_message().await;
            assert!(result.is_ok(), "Failed to receive message {}", i);
        }
    }

    #[tokio::test]
    async fn test_simple_transport_concurrent_operations() {
        let transport = SimpleTransport;

        // Spawn multiple concurrent operations
        let mut handles = vec![];

        for _ in 0..5 {
            let t = SimpleTransport;
            let handle = tokio::spawn(async move {
                let msg = MCPMessage::default();
                t.send_message(msg).await
            });
            handles.push(handle);
        }

        // Wait for all to complete
        for handle in handles {
            let result = handle.await;
            assert!(result.is_ok());
            assert!(result.unwrap().is_ok());
        }
    }

    // ========== TransportMetadata Serialization Tests ==========

    #[test]
    fn test_transport_metadata_with_ipv4_addresses() {
        use std::net::{IpAddr, Ipv4Addr, SocketAddr};

        let remote_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)), 8080);
        let local_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3000);

        let metadata = types::TransportMetadata {
            connection_id: "ipv4-conn".to_string(),
            remote_address: Some(remote_addr),
            local_address: Some(local_addr),
            connected_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            encryption_format: None,
            compression_format: None,
            additional_info: HashMap::new(),
        };

        let remote = metadata.remote_address.unwrap();
        assert_eq!(remote.ip(), IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)));
        assert_eq!(remote.port(), 8080);
    }

    #[test]
    fn test_transport_metadata_with_ipv6_addresses() {
        use std::net::{IpAddr, Ipv6Addr, SocketAddr};

        let remote_addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 8080);
        let local_addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 3000);

        let metadata = types::TransportMetadata {
            connection_id: "ipv6-conn".to_string(),
            remote_address: Some(remote_addr),
            local_address: Some(local_addr),
            connected_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            encryption_format: None,
            compression_format: None,
            additional_info: HashMap::new(),
        };

        let remote = metadata.remote_address.unwrap();
        assert_eq!(remote.ip(), IpAddr::V6(Ipv6Addr::LOCALHOST));
        assert_eq!(remote.port(), 8080);
    }

    #[test]
    fn test_transport_metadata_connection_id_formats() {
        let uuid_based = types::TransportMetadata {
            connection_id: format!("conn-{}", uuid::Uuid::new_v4()),
            remote_address: None,
            local_address: None,
            connected_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            encryption_format: None,
            compression_format: None,
            additional_info: HashMap::new(),
        };

        assert!(uuid_based.connection_id.starts_with("conn-"));
        assert!(uuid_based.connection_id.len() > 10);

        let simple = types::TransportMetadata {
            connection_id: "conn-1".to_string(),
            remote_address: None,
            local_address: None,
            connected_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            encryption_format: None,
            compression_format: None,
            additional_info: HashMap::new(),
        };

        assert_eq!(simple.connection_id, "conn-1");
    }

    // ========== Encryption + Compression Combination Tests ==========

    #[test]
    fn test_metadata_with_aes128_and_gzip() {
        let metadata = types::TransportMetadata {
            connection_id: "secure-compressed".to_string(),
            remote_address: None,
            local_address: None,
            connected_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            encryption_format: Some(crate::types::EncryptionFormat::Aes128),
            compression_format: Some(crate::types::CompressionFormat::Gzip),
            additional_info: HashMap::new(),
        };

        assert_eq!(
            metadata.encryption_format,
            Some(crate::types::EncryptionFormat::Aes128)
        );
        assert_eq!(
            metadata.compression_format,
            Some(crate::types::CompressionFormat::Gzip)
        );
    }

    #[test]
    fn test_metadata_with_aes256_and_zstd() {
        let metadata = types::TransportMetadata {
            connection_id: "high-security".to_string(),
            remote_address: None,
            local_address: None,
            connected_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            encryption_format: Some(crate::types::EncryptionFormat::Aes256),
            compression_format: Some(crate::types::CompressionFormat::Zstd),
            additional_info: HashMap::new(),
        };

        assert_eq!(
            metadata.encryption_format,
            Some(crate::types::EncryptionFormat::Aes256)
        );
        assert_eq!(
            metadata.compression_format,
            Some(crate::types::CompressionFormat::Zstd)
        );
    }

    #[test]
    fn test_metadata_with_chacha20_and_lz4() {
        let metadata = types::TransportMetadata {
            connection_id: "fast-secure".to_string(),
            remote_address: None,
            local_address: None,
            connected_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            encryption_format: Some(crate::types::EncryptionFormat::ChaCha20),
            compression_format: Some(crate::types::CompressionFormat::Lz4),
            additional_info: HashMap::new(),
        };

        assert_eq!(
            metadata.encryption_format,
            Some(crate::types::EncryptionFormat::ChaCha20)
        );
        assert_eq!(
            metadata.compression_format,
            Some(crate::types::CompressionFormat::Lz4)
        );
    }

    // ========== Edge Cases ==========

    #[test]
    fn test_metadata_with_very_long_connection_id() {
        let long_id = "a".repeat(1000);
        let metadata = types::TransportMetadata {
            connection_id: long_id.clone(),
            remote_address: None,
            local_address: None,
            connected_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            encryption_format: None,
            compression_format: None,
            additional_info: HashMap::new(),
        };

        assert_eq!(metadata.connection_id.len(), 1000);
        assert_eq!(metadata.connection_id, long_id);
    }

    #[test]
    fn test_metadata_with_special_characters_in_connection_id() {
        let special_id = "conn-!@#$%^&*()_+-={}[]|:;<>?,./~`".to_string();
        let metadata = types::TransportMetadata {
            connection_id: special_id.clone(),
            remote_address: None,
            local_address: None,
            connected_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            encryption_format: None,
            compression_format: None,
            additional_info: HashMap::new(),
        };

        assert_eq!(metadata.connection_id, special_id);
    }

    #[test]
    fn test_metadata_with_large_additional_info() {
        let mut additional = HashMap::new();
        for i in 0..100 {
            additional.insert(format!("key_{}", i), format!("value_{}", i));
        }

        let metadata = types::TransportMetadata {
            connection_id: "large-metadata".to_string(),
            remote_address: None,
            local_address: None,
            connected_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            encryption_format: None,
            compression_format: None,
            additional_info: additional,
        };

        assert_eq!(metadata.additional_info.len(), 100);
        assert_eq!(
            metadata.additional_info.get("key_0"),
            Some(&"value_0".to_string())
        );
        assert_eq!(
            metadata.additional_info.get("key_99"),
            Some(&"value_99".to_string())
        );
    }

    // ========== Time-based Tests ==========

    #[test]
    fn test_metadata_activity_tracking_over_time() {
        let start = chrono::Utc::now();

        let metadata1 = types::TransportMetadata {
            connection_id: "time-test".to_string(),
            remote_address: None,
            local_address: None,
            connected_at: start,
            last_activity: start,
            encryption_format: None,
            compression_format: None,
            additional_info: HashMap::new(),
        };

        std::thread::sleep(std::time::Duration::from_millis(10));
        let later = chrono::Utc::now();

        let metadata2 = types::TransportMetadata {
            connection_id: "time-test".to_string(),
            remote_address: None,
            local_address: None,
            connected_at: start,
            last_activity: later,
            encryption_format: None,
            compression_format: None,
            additional_info: HashMap::new(),
        };

        assert!(metadata2.last_activity > metadata1.last_activity);
        assert_eq!(metadata1.connected_at, metadata2.connected_at);
    }
}
