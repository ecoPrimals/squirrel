# Changelog

## [Unreleased]

### Added
- Enhanced security features for credential management
  - Secure credential storage with in-memory and file-based options
  - Credential rotation capabilities with configurable policies
  - Encryption for stored credentials using XChaCha20-Poly1305
  - Environment variable integration for secure credential loading
  - SecretString type for secure management of sensitive strings
- New security documentation in SECURITY.md
- New examples demonstrating security features
  - examples/security_usage.rs for basic security usage
  - examples/enhanced_security.rs for advanced security features
- New configuration options for security settings
  - Key rotation period configuration
  - Credential history size configuration
  - Encryption settings for credential storage

### Changed
- Updated GalaxyConfig to support enhanced security options
- Modified GalaxyAdapter to incorporate SecurityManager
- Improved credential validation and rotation in the adapter
- Extended configuration to support environment variable integration

### Security
- Added credential rotation to improve operational security
- Implemented encryption for credentials at rest
- Added secure memory handling for sensitive strings
- Enhanced validation of credentials against Galaxy API

## [0.1.0] - 2023-12-15

### Added
- Initial implementation of Galaxy MCP adapter
- Galaxy API client with basic functionality
- Configuration management system
- Plugin architecture
- Error handling system
- MCP protocol integration
- Models for Galaxy resources
- Example code for common use cases 