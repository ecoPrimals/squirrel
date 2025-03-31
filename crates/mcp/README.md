# Machine Context Protocol (MCP)

The Machine Context Protocol (MCP) is a communication framework that facilitates secure and efficient exchange of contextual information between different parts of a distributed system. It provides a standardized way to share, synchronize, and manage context data across multiple nodes.

## Key Features

- **Secure Communication**: Strong encryption, authentication, and authorization mechanisms.
- **Context Management**: Hierarchical context organization with inheritance and validation.
- **Flexible Transport**: Support for multiple transport mechanisms (TCP, WebSocket, in-memory).
- **Role-Based Access Control**: Fine-grained permission system for secure resource access.
- **Token-based Authentication**: JWT-based secure token management.
- **Pluggable Architecture**: Extensible system through a plugin mechanism.

## Security Module Improvements

### Token Management

The security module has been enhanced with a robust JWT-based token implementation:

- **JWT Token Support**: Secure JSON Web Token implementation for authentication.
- **Token Revocation**: In-memory revocation list with automatic cleanup of expired tokens.
- **Token Validation**: Comprehensive validation including expiration, signature, and revocation checks.
- **Secure Key Storage**: Integration with key storage system for token signing keys.

### Enhanced Encryption

The encryption system provides:

- **Multiple Algorithms**: Support for AES-GCM and ChaCha20-Poly1305.
- **Secure Key Management**: Automatic key generation and rotation.
- **Authenticated Encryption**: Protection against tampering and unauthorized access.

### RBAC System

The Role-Based Access Control system offers:

- **Permission Management**: Create, assign, and verify permissions.
- **Role Hierarchy**: Support for role inheritance and organization.
- **Resource Protection**: Fine-grained access control for system resources.
- **Permission Conditions**: Conditional evaluation for complex access rules.

## Getting Started

To use MCP in your project, add it as a dependency in your Cargo.toml:

```toml
[dependencies]
squirrel-mcp = { path = "path/to/crates/mcp" }
```

## Usage Example

```rust
use squirrel_mcp::security::{DefaultAuthManager, Credentials};
use squirrel_mcp::security::token::TokenManager;
use std::sync::Arc;

async fn example() {
    // Initialize security components
    let auth_manager = DefaultAuthManager::new(
        Arc::new(DefaultIdentityManager::new()),
        Arc::new(DefaultCryptoProvider::new()),
        Arc::new(MockRBACManager),
        Arc::new(DefaultTokenManager::new(
            Arc::new(InMemoryKeyStorage::new()),
            Arc::new(DefaultCryptoProvider::new())
        )),
    );
    
    // Authenticate a user
    let credentials = Credentials {
        username: "user".to_string(),
        password: "password".to_string(),
    };
    
    let token = auth_manager.authenticate(&credentials).await.unwrap();
    
    // Use the token for authorization
    let resource = Resource { id: "document-123".to_string(), attributes: None };
    let action = Action::Read;
    
    match auth_manager.authorize(&token, &resource, &action, None).await {
        Ok(_) => println!("Access granted"),
        Err(_) => println!("Access denied"),
    }
}
```

## Contributing

Contributions to the MCP project are welcome! Please follow these steps:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Submit a pull request

## License

This project is licensed under the terms specified in the workspace configuration. 