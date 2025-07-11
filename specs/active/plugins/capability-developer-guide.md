---
description: GUIDE for plugin developers implementing and using the capability-based security system
---

# Plugin Capability Developer Guide

## Context
- When developing plugins that require specific security permissions
- When implementing new capabilities for plugins
- When requesting access to protected resources
- When integrating with the Squirrel security model

## Overview

The Squirrel platform uses a capability-based security model to control what plugins can and cannot do. This guide explains how to work with this system as a plugin developer, including how to request and use capabilities, define custom capabilities, and follow security best practices.

## Understanding Capabilities

### What is a Capability?

A capability is a specific permission to perform an action on a resource. All capabilities follow the format `namespace:action` where:

- `namespace`: The category or type of resource (e.g., `file`, `network`, `plugin`)
- `action`: The specific operation on that resource (e.g., `read`, `write`, `execute`)

For example, `file:read` grants permission to read files, while `network:connect` allows creating outbound network connections.

### Standard Capability Namespaces

The platform provides several standard capability namespaces:

| Namespace | Description | Example Capabilities |
|-----------|-------------|---------------------|
| `file` | File system operations | `file:read`, `file:write`, `file:delete` |
| `network` | Network operations | `network:connect`, `network:listen` |
| `plugin` | Plugin management | `plugin:read`, `plugin:execute`, `plugin:install` |
| `config` | Configuration access | `config:read`, `config:write` |
| `system` | System-level operations | `system:info`, `system:admin`, `system:resources` |

### Capability Wildcard Support

The system supports wildcard capabilities using the `*` character in the action part:

- `file:*` - All file operations
- `network:*` - All network operations

Wildcards can only be used in the action part, not in the namespace.

## Requesting Capabilities

### In Plugin Manifest

To request capabilities for your plugin, include them in your plugin's manifest file:

```json
{
  "name": "my-awesome-plugin",
  "version": "1.0.0",
  "capabilities": [
    "file:read",
    "file:write",
    "network:connect",
    "config:read"
  ],
  "customCapabilities": [
    "my-plugin:special-action"
  ]
}
```

The `capabilities` array contains standard capabilities, while `customCapabilities` lists any custom capabilities your plugin defines.

### Runtime Capability Requests

Sometimes, plugins need additional capabilities at runtime. Use the `requestCapability` API:

```rust
async fn need_more_capabilities(&self, context: &PluginContext) -> Result<()> {
    // Request a new capability at runtime
    if let Err(e) = context.security.request_capability("network:listen").await {
        // Handle rejection
        return Err(format!("Capability request denied: {}", e).into());
    }
    
    // Now we can use the capability
    self.start_listener()?;
    
    Ok(())
}
```

Runtime requests will trigger a user prompt to approve or deny the request.

## Checking Capabilities

### Verifying Capabilities Before Use

Always check if your plugin has a capability before attempting operations that require it:

```rust
async fn write_data(&self, context: &PluginContext, path: &Path, data: &str) -> Result<()> {
    // Check if we have the required capability
    if !context.security.has_capability("file:write").await? {
        return Err("Missing file:write capability".into());
    }
    
    // Proceed with the operation
    context.fs.write_file(path, data).await?;
    
    Ok(())
}
```

### Graceful Degradation

Design your plugin to gracefully handle missing capabilities:

```rust
async fn initialize(&self, context: &PluginContext) -> Result<()> {
    // Check for optimal capabilities
    let can_write = context.security.has_capability("file:write").await?;
    let can_connect = context.security.has_capability("network:connect").await?;
    
    if can_write && can_connect {
        // Full functionality available
        self.mode = PluginMode::Full;
    } else if can_connect {
        // Network-only mode
        self.mode = PluginMode::NetworkOnly;
        log::info!("Running in network-only mode (missing file:write capability)");
    } else if can_write {
        // Local-only mode
        self.mode = PluginMode::LocalOnly;
        log::info!("Running in local-only mode (missing network:connect capability)");
    } else {
        // Minimal mode
        self.mode = PluginMode::Minimal;
        log::warn!("Running in minimal mode (missing required capabilities)");
    }
    
    Ok(())
}
```

## Defining Custom Capabilities

### When to Create Custom Capabilities

Create custom capabilities when:
1. Your plugin exposes functionality to other plugins
2. You need fine-grained control over access to your plugin's features
3. You're implementing domain-specific operations not covered by standard capabilities

### Custom Capability Guidelines

1. Always use your plugin's unique namespace
2. Use clear, descriptive action names
3. Document what each capability allows
4. Follow the `namespace:action` format
5. Keep capabilities focused on specific actions

### Example Custom Capability Implementation

```rust
// Define your capability constants
const CAP_DATASCI_READ: &str = "datasci:read";
const CAP_DATASCI_WRITE: &str = "datasci:write";
const CAP_DATASCI_ANALYZE: &str = "datasci:analyze";

// Register your custom capabilities
fn register_capabilities(&self, context: &PluginContext) -> Result<()> {
    context.security.register_capability(
        CAP_DATASCI_READ,
        "Read access to data science resources",
        CapabilityImpact::Low
    )?;
    
    context.security.register_capability(
        CAP_DATASCI_WRITE,
        "Write access to data science resources",
        CapabilityImpact::Medium
    )?;
    
    context.security.register_capability(
        CAP_DATASCI_ANALYZE,
        "Execute analysis on data science resources",
        CapabilityImpact::Medium
    )?;
    
    Ok(())
}

// Check capabilities in your API implementation
async fn perform_analysis(&self, context: &PluginContext, data: &[u8]) -> Result<Analysis> {
    // Check for required capability
    if !context.security.has_capability(CAP_DATASCI_ANALYZE).await? {
        return Err("Missing datasci:analyze capability".into());
    }
    
    // Proceed with analysis
    let result = self.analyze_data(data)?;
    
    Ok(result)
}
```

## Best Practices

### Capability Management

1. **Request Minimum Capabilities**: Only request capabilities your plugin truly needs
2. **Explain Usage**: Document why your plugin needs each capability
3. **Graceful Degradation**: Handle missing capabilities gracefully
4. **Clear Error Messages**: Provide helpful error messages when capabilities are missing
5. **Capability Checking**: Always check capabilities before performing protected operations

### Security Considerations

1. **Don't Circumvent**: Never try to bypass capability checks
2. **Validate Input**: Always validate input data, even with capabilities
3. **Avoid Privilege Escalation**: Don't use capabilities to gain more privileges
4. **Audit Usage**: Log usage of sensitive capabilities for auditing
5. **Temporary Capabilities**: Release capabilities when no longer needed

### Custom Capability Design

1. **Unique Namespaces**: Use unique identifiers for your plugin's namespace
2. **Granular Actions**: Define specific actions rather than broad permissions
3. **Clear Documentation**: Document what each capability allows and why it's needed
4. **Impact Assessment**: Accurately rate the security impact of each capability
5. **Versioning**: Version your capabilities if their meaning changes

## Common Patterns

### Capability-Based API Design

Structure your API around capabilities:

```rust
// Service with capability-gated APIs
struct MyPluginService {
    // API methods require specific capabilities
}

impl MyPluginService {
    // Read operation - requires read capability
    async fn get_data(&self, context: &PluginContext, id: &str) -> Result<Data> {
        context.security.check_capability("my-plugin:read")?;
        // Implementation
    }
    
    // Write operation - requires write capability
    async fn save_data(&self, context: &PluginContext, data: &Data) -> Result<()> {
        context.security.check_capability("my-plugin:write")?;
        // Implementation
    }
    
    // Admin operation - requires admin capability
    async fn purge_data(&self, context: &PluginContext) -> Result<()> {
        context.security.check_capability("my-plugin:admin")?;
        // Implementation
    }
}
```

### Progressive Enhancement

Enhance functionality based on available capabilities:

```rust
// Feature levels based on capabilities
enum FeatureLevel {
    Basic,    // Basic features only
    Standard, // Standard feature set
    Advanced, // Advanced features
    Admin     // Administrative features
}

// Determine feature level from capabilities
async fn get_feature_level(context: &PluginContext) -> Result<FeatureLevel> {
    if context.security.has_capability("my-plugin:admin").await? {
        return Ok(FeatureLevel::Admin);
    }
    
    if context.security.has_capability("my-plugin:advanced").await? {
        return Ok(FeatureLevel::Advanced);
    }
    
    if context.security.has_capability("my-plugin:write").await? {
        return Ok(FeatureLevel::Standard);
    }
    
    // Basic level is default
    Ok(FeatureLevel::Basic)
}
```

## Troubleshooting

### Common Capability Issues

1. **Capability Not Found**: Ensure the capability is properly declared in your manifest
2. **Permission Denied**: User may have denied the capability request
3. **Invalid Capability Format**: Check that you're using the correct `namespace:action` format
4. **Custom Namespace Collision**: Your namespace may conflict with another plugin

### Debugging Capability Issues

1. Enable security debug logging to see capability checks:
   ```rust
   context.debug.set_security_logging(true);
   ```

2. Check your plugin's granted capabilities:
   ```rust
   let capabilities = context.security.list_capabilities().await?;
   for cap in capabilities {
       log::debug!("Granted capability: {}", cap);
   }
   ```

3. Use the capability inspection tool:
   ```bash
   squirrel-cli plugin inspect-capabilities my-plugin
   ```

## Conclusion

The capability-based security model provides a flexible and powerful way to secure plugin interactions while allowing plugin developers to clearly communicate their security requirements. By following this guide, you can create plugins that respect security boundaries while providing rich functionality to users.

<version>1.0.0</version> 