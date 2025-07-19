use tokio::test;

use crate::mcp::transport::TransportConfig;
use crate::mcp::types::{
    ProtocolVersion,
    SecurityLevel,
    CompressionFormat,
    EncryptionFormat,
};

#[test]
async fn test_transport_config_default() {
    // Create default config
    let config = TransportConfig::default();
    
    // Verify default values
    assert_eq!(config.bind_address, "0.0.0.0");
    assert_eq!(config.port, 9000); // Default port
    assert_eq!(config.max_connections, 100);
    assert_eq!(config.max_message_size, 10485760); // 10MB
    assert_eq!(config.protocol_version, ProtocolVersion::new(1, 0));
    assert_eq!(config.security_level, SecurityLevel::None);
    assert_eq!(config.compression, CompressionFormat::None);
    assert_eq!(config.encryption, EncryptionFormat::None);
}

#[test]
async fn test_transport_config_custom() {
    // Create custom config
    let config = TransportConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 8080,
        max_connections: 50,
        max_message_size: 1024 * 1024, // 1MB
        protocol_version: ProtocolVersion::new(2, 0),
        security_level: SecurityLevel::Standard,
        compression: CompressionFormat::GZIP,
        encryption: EncryptionFormat::AES256GCM,
    };
    
    // Verify custom values
    assert_eq!(config.bind_address, "127.0.0.1");
    assert_eq!(config.port, 8080);
    assert_eq!(config.max_connections, 50);
    assert_eq!(config.max_message_size, 1048576); // 1MB
    assert_eq!(config.protocol_version, ProtocolVersion::new(2, 0));
    assert_eq!(config.security_level, SecurityLevel::Standard);
    assert_eq!(config.compression, CompressionFormat::GZIP);
    assert_eq!(config.encryption, EncryptionFormat::AES256GCM);
}

#[test]
async fn test_transport_config_clone() {
    // Create config
    let config = TransportConfig {
        bind_address: "192.168.1.1".to_string(),
        port: 7000,
        max_connections: 10,
        max_message_size: 1024 * 100, // 100KB
        protocol_version: ProtocolVersion::new(1, 1),
        security_level: SecurityLevel::High,
        compression: CompressionFormat::ZSTD,
        encryption: EncryptionFormat::ChaCha20Poly1305,
    };
    
    // Clone the config
    let cloned = config.clone();
    
    // Verify cloned values match original
    assert_eq!(cloned.bind_address, config.bind_address);
    assert_eq!(cloned.port, config.port);
    assert_eq!(cloned.max_connections, config.max_connections);
    assert_eq!(cloned.max_message_size, config.max_message_size);
    assert_eq!(cloned.protocol_version, config.protocol_version);
    assert_eq!(cloned.security_level, config.security_level);
    assert_eq!(cloned.compression, config.compression);
    assert_eq!(cloned.encryption, config.encryption);
}

#[test]
async fn test_transport_config_security_encryption_compatibility() {
    // Test various combinations of security level and encryption format
    
    // Case 1: None security with no encryption (valid)
    let config1 = TransportConfig {
        security_level: SecurityLevel::None,
        encryption: EncryptionFormat::None,
        ..TransportConfig::default()
    };
    
    // Case 2: Standard security with encryption (valid)
    let config2 = TransportConfig {
        security_level: SecurityLevel::Standard,
        encryption: EncryptionFormat::AES256GCM,
        ..TransportConfig::default()
    };
    
    // Case 3: High security with strong encryption (valid)
    let config3 = TransportConfig {
        security_level: SecurityLevel::High,
        encryption: EncryptionFormat::ChaCha20Poly1305,
        ..TransportConfig::default()
    };
    
    // Test that the configurations can be created (validation would happen at runtime)
    assert_eq!(config1.security_level, SecurityLevel::None);
    assert_eq!(config1.encryption, EncryptionFormat::None);
    
    assert_eq!(config2.security_level, SecurityLevel::Standard);
    assert_eq!(config2.encryption, EncryptionFormat::AES256GCM);
    
    assert_eq!(config3.security_level, SecurityLevel::High);
    assert_eq!(config3.encryption, EncryptionFormat::ChaCha20Poly1305);
} 