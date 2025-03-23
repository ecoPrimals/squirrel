# Dashboard Security Features

This document outlines the security features available in the monitoring dashboard, including their configuration, usage, and best practices.

## Overview

The monitoring dashboard includes a comprehensive security framework that addresses several key security concerns:

- **TLS Encryption**: Secure communication between clients and the server
- **Authentication**: Verify user identity
- **Authorization**: Control access to dashboard features based on roles
- **Rate Limiting**: Prevent abuse through request throttling
- **Origin Verification**: Control which origins can connect to the dashboard
- **Data Masking**: Hide sensitive information in logs and responses
- **Audit Logging**: Track security-relevant events for later review

These features can be configured through the `DashboardConfig` structure when initializing the dashboard.

## Security Configuration

### Basic Configuration

```rust
use monitoring::dashboard::{
    DashboardConfig, 
    DashboardManager,
    TlsConfig,
    AuthConfig,
    RateLimitConfig,
    security::{
        AuthType,
        MonitoringRole,
        Permission,
        TlsVersion,
        CipherPreferences,
        MaskingRule
    }
};

// Create a basic configuration with security features
let mut config = DashboardConfig::default();

// Configure TLS
config.security.tls = Some(TlsConfig {
    cert_path: PathBuf::from("path/to/cert.pem"),
    key_path: PathBuf::from("path/to/key.pem"),
    min_tls_version: TlsVersion::Tls13,
    cipher_preferences: CipherPreferences::Modern,
});

// Create dashboard manager with secure configuration
let dashboard = DashboardManager::new(config);
```

### Using the Builder Pattern

The `DashboardConfig` struct also supports a builder pattern for more readable configuration:

```rust
let config = DashboardConfig::default()
    .with_host("0.0.0.0")
    .with_port(8765)
    .with_tls("path/to/cert.pem", "path/to/key.pem")
    .with_allowed_origins(vec![
        "https://example.com".to_string(),
        "https://admin.example.com".to_string(),
    ])
    .with_masking_rule(r"[0-9]{4}-[0-9]{4}-[0-9]{4}-[0-9]{4}", "****-****-****-****")
    .with_audit_logging("logs/audit.log");

let dashboard = DashboardManager::new(config);
```

## TLS Encryption

TLS (Transport Layer Security) encryption secures the WebSocket connection between clients and the server.

### Configuration

```rust
// Configure TLS
config.security.tls = Some(TlsConfig {
    cert_path: PathBuf::from("path/to/cert.pem"),
    key_path: PathBuf::from("path/to/key.pem"),
    min_tls_version: TlsVersion::Tls13,
    cipher_preferences: CipherPreferences::Modern,
});
```

### TLS Versions

- `TlsVersion::Tls12`: TLS 1.2 (older but widely supported)
- `TlsVersion::Tls13`: TLS 1.3 (newer, more secure, recommended)

### Cipher Preferences

- `CipherPreferences::Modern`: Most secure ciphers, may not work with older clients
- `CipherPreferences::Intermediate`: Balance between security and compatibility
- `CipherPreferences::Legacy`: Maximum compatibility, but less secure (not recommended)

### Best Practices

1. Always use valid certificates from trusted certificate authorities in production
2. Use TLS 1.3 when possible
3. Use Modern cipher preferences unless you need to support older clients
4. Keep private keys secure and regularly rotate certificates
5. Implement certificate pinning in clients for additional security

## Authentication

Authentication verifies the identity of users connecting to the dashboard.

### Configuration

```rust
use std::collections::HashMap;

// Configure users and roles
let mut users = HashMap::new();
users.insert("admin".to_string(), MonitoringRole::Administrator);
users.insert("operator".to_string(), MonitoringRole::Operator);
users.insert("viewer".to_string(), MonitoringRole::Viewer);

// Configure authentication
config.security.auth = AuthConfig {
    auth_type: AuthType::Bearer,
    token_expiration: 8 * 60 * 60, // 8 hours in seconds
    require_reauth: true,
    users,
};
```

### Authentication Types

- `AuthType::None`: No authentication (not recommended for production)
- `AuthType::Basic`: Basic authentication (username/password)
- `AuthType::Bearer`: JWT token authentication (recommended)
- `AuthType::Custom`: Custom authentication mechanism

### Roles and Permissions

The system includes predefined roles:

- `MonitoringRole::Viewer`: Can only view dashboards, metrics, and alerts
- `MonitoringRole::Operator`: Can view and acknowledge alerts, modify dashboards
- `MonitoringRole::Administrator`: Full access to all features
- `MonitoringRole::Custom(Vec<Permission>)`: Custom role with specific permissions

Available permissions:
- `Permission::ViewDashboards`: View dashboards
- `Permission::ModifyDashboards`: Create or modify dashboards
- `Permission::ViewMetrics`: View metrics
- `Permission::ViewAlerts`: View alerts
- `Permission::AcknowledgeAlerts`: Acknowledge alerts
- `Permission::ConfigureAlerts`: Configure alert settings
- `Permission::ConfigureSystem`: Configure system settings
- `Permission::Admin`: Administrative functions

### Client Authentication

Clients must include an `Authorization` header with their WebSocket connection:

```javascript
// Example JavaScript client
const socket = new WebSocket('wss://dashboard.example.com/ws', {
  headers: {
    'Authorization': 'Bearer ' + token
  }
});
```

### Token Management

The system automatically validates tokens based on:
- Expiration time
- Signature validity
- Role and permissions

For token generation, you can use the `AuthManager`:

```rust
// Get JWT token for a user
let auth_manager = /* Get from dashboard manager */;
let token = auth_manager.create_token("username").expect("Token creation failed");
```

## Rate Limiting

Rate limiting prevents abuse by limiting the number of connections and requests per client.

### Configuration

```rust
// Configure rate limiting
config.security.rate_limit = RateLimitConfig {
    max_connections_per_ip: 20,       // Maximum connections from a single IP
    max_messages_per_minute: 300,     // Maximum messages per minute per client
    max_subscription_requests_per_minute: 50,  // Maximum subscription requests
};
```

### Rate Limiting Features

- **Connection Limiting**: Limits the number of concurrent connections from a single IP address
- **Message Rate Limiting**: Limits the number of messages a client can send per minute
- **Subscription Rate Limiting**: Limits the frequency of subscription requests

## Origin Verification

Origin verification restricts which websites or applications can connect to the dashboard.

### Configuration

```rust
// Configure allowed origins
config.security.allowed_origins = vec![
    "https://dashboard.example.com".to_string(),
    "https://admin.example.com".to_string(),
];
```

The system checks the `Origin` header of WebSocket connection requests against this list. If the list is empty, all origins are allowed (not recommended for production).

## Data Masking

Data masking hides sensitive information in logs, responses, and stored data using regular expressions.

### Configuration

```rust
// Configure data masking rules
config.security.masking_rules = vec![
    // Mask credit card numbers
    MaskingRule::new(r"[0-9]{4}-[0-9]{4}-[0-9]{4}-[0-9]{4}", "****-****-****-****"),
    // Mask passwords in configuration strings
    MaskingRule::new(r"password\s*=\s*['\"].*?['\"]", "password=\"*****\""),
    // Mask API keys
    MaskingRule::new(r"api_key\s*=\s*['\"].*?['\"]", "api_key=\"*****\""),
];
```

### Creating Custom Rules

Each masking rule consists of:
- A regular expression pattern to match
- A replacement string to substitute for the matched text
- An optional case sensitivity flag

```rust
// Create a new masking rule
let rule = MaskingRule {
    pattern: r"secret_token\s*:\s*['\"].*?['\"]".to_string(),
    replacement: "secret_token: \"*****\"".to_string(),
    case_sensitive: false,
};
```

## Audit Logging

Audit logging records security-relevant events for later review, which is essential for security compliance and incident investigation.

### Configuration

```rust
// Enable audit logging
config.security.audit = Some(AuditConfig {
    enabled: true,
    storage: AuditStorage::File(PathBuf::from("logs/audit.log")),
    include_user_context: true,
    tamper_proof: true,
});
```

### Storage Options

- `AuditStorage::File(path)`: Store audit logs in a file
- `AuditStorage::Database(connection_string)`: Store audit logs in a database
- `AuditStorage::Custom`: Use a custom storage mechanism

### Event Types

The following security events are logged:

- Authentication attempts (success and failure)
- Authorization failures
- Configuration changes
- Client connections and disconnections
- Component subscriptions
- Admin actions
- Data access to sensitive components

### Log Format

Audit logs are stored in JSON format:

```json
{
  "timestamp": "2023-06-10T15:32:45.123Z",
  "event_type": "authentication_success",
  "username": "admin",
  "details": {
    "ip": "192.168.1.100",
    "user_agent": "Mozilla/5.0...",
    "method": "jwt"
  }
}
```

## Security Logging

In addition to audit logging, the system also provides general security logging through the application's logging system.

### Configuration

```rust
// Configure security logging
config.security.logging = SecurityLoggingConfig {
    log_authentication_attempts: true,
    log_authorization_failures: true,
    log_configuration_changes: true,
    log_data_access: AccessLoggingLevel::Sensitive,
};
```

### Access Logging Levels

- `AccessLoggingLevel::None`: No access logging
- `AccessLoggingLevel::Sensitive`: Log access to sensitive data only
- `AccessLoggingLevel::All`: Log all data access

## Client Security Considerations

When implementing client applications that connect to the dashboard, follow these best practices:

1. **Secure Token Storage**: Store JWT tokens securely, preferably in memory only
2. **HTTPS Only**: Always use HTTPS for the initial page load
3. **WSS Protocol**: Always use `wss://` (WebSocket Secure) instead of `ws://`
4. **Token Refresh**: Implement token refresh before expiration
5. **Error Handling**: Handle authentication errors and redirect to login
6. **Certificate Pinning**: Consider certificate pinning for high-security environments

## Example: Complete Secure Configuration

Here's a complete example of a secure dashboard configuration:

```rust
use std::collections::HashMap;
use std::path::PathBuf;
use monitoring::dashboard::{
    DashboardConfig, 
    DashboardManager,
    TlsConfig,
    AuthConfig,
    RateLimitConfig,
    SecurityLoggingConfig,
    AuditConfig,
    security::{
        AuthType,
        MonitoringRole,
        Permission,
        TlsVersion,
        CipherPreferences,
        MaskingRule,
        AuditStorage,
        AccessLoggingLevel
    }
};

// Create a secure configuration
let mut config = DashboardConfig::default();

// Configure server settings
config.server.host = "0.0.0.0".to_string();
config.server.port = 8765;

// Configure TLS
config.security.tls = Some(TlsConfig {
    cert_path: PathBuf::from("/etc/certs/dashboard.pem"),
    key_path: PathBuf::from("/etc/certs/dashboard.key"),
    min_tls_version: TlsVersion::Tls13,
    cipher_preferences: CipherPreferences::Modern,
});

// Configure authentication
let mut users = HashMap::new();
users.insert("admin".to_string(), MonitoringRole::Administrator);
users.insert("operator".to_string(), MonitoringRole::Operator);
users.insert("viewer".to_string(), MonitoringRole::Viewer);

config.security.auth = AuthConfig {
    auth_type: AuthType::Bearer,
    token_expiration: 8 * 60 * 60, // 8 hours
    require_reauth: true,
    users,
};

// Configure rate limiting
config.security.rate_limit = RateLimitConfig {
    max_connections_per_ip: 20,
    max_messages_per_minute: 300,
    max_subscription_requests_per_minute: 50,
};

// Configure allowed origins
config.security.allowed_origins = vec![
    "https://dashboard.example.com".to_string(),
    "https://admin.example.com".to_string(),
];

// Configure data masking
config.security.masking_rules = vec![
    MaskingRule::new(r"[0-9]{4}-[0-9]{4}-[0-9]{4}-[0-9]{4}", "****-****-****-****"),
    MaskingRule::new(r"password\s*=\s*['\"].*?['\"]", "password=\"*****\""),
];

// Configure security logging
config.security.logging = SecurityLoggingConfig {
    log_authentication_attempts: true,
    log_authorization_failures: true,
    log_configuration_changes: true,
    log_data_access: AccessLoggingLevel::Sensitive,
};

// Configure audit logging
config.security.audit = Some(AuditConfig {
    enabled: true,
    storage: AuditStorage::File(PathBuf::from("/var/log/dashboard/audit.log")),
    include_user_context: true,
    tamper_proof: true,
});

// Create dashboard manager with secure configuration
let dashboard = DashboardManager::new(config);
```

## Security Best Practices

1. **Enable TLS**: Always use TLS encryption in production
2. **Require Authentication**: Always require authentication in production
3. **Principle of Least Privilege**: Assign the minimum necessary permissions
4. **Rate Limiting**: Always enable rate limiting
5. **Origin Verification**: Restrict allowed origins in production
6. **Data Masking**: Configure masking rules for sensitive data
7. **Audit Logging**: Enable audit logging for compliance and security
8. **Regular Updates**: Keep all dependencies updated
9. **Security Testing**: Regularly perform security testing on the dashboard
10. **Secret Management**: Use a secure method to manage JWT secrets and TLS keys

## Troubleshooting

### Common Issues

1. **Authentication Failures**:
   - Check that JWT secret is consistent across server restarts
   - Verify that token expiration is reasonable
   - Check client clock synchronization

2. **TLS Configuration Issues**:
   - Verify certificate and key paths
   - Check certificate validity and expiration
   - Ensure certificate matches domain name

3. **Rate Limiting Too Restrictive**:
   - Adjust rate limiting parameters based on expected usage
   - Consider different limits for different user roles

4. **Missing Audit Logs**:
   - Check file permissions for audit log directory
   - Verify audit storage configuration

For more complex issues, consult the detailed logs by enabling debug logging:

```rust
// Enable debug logging
std::env::set_var("RUST_LOG", "monitoring::dashboard=debug");
tracing_subscriber::fmt::init();
``` 