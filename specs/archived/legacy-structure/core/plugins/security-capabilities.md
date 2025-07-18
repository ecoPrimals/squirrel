---
description: DEFINE security capabilities used in the plugin sandbox
---

# Plugin Security Capabilities

## Context
- When implementing security checks
- When defining plugin permissions
- When creating new plugin features
- When auditing security operations

## Capability Format

All capabilities follow the format `namespace:action` where:
- `namespace` is the category of the resource (e.g., file, plugin, network)
- `action` is the specific permission (e.g., read, write, execute)

Wildcard capabilities can be specified using `*` as in `namespace:*` to grant all permissions within a namespace.

## Standard Capabilities

### File System Capabilities
- `file:read` - Read files within allowed paths
- `file:write` - Write files within allowed paths
- `file:delete` - Delete files within allowed paths
- `file:execute` - Execute files within allowed paths

### Plugin Capabilities
- `plugin:read` - Read plugin information
- `plugin:write` - Modify plugin information
- `plugin:execute` - Execute plugin functions
- `plugin:install` - Install new plugins
- `plugin:uninstall` - Remove plugins

### Network Capabilities
- `network:connect` - Establish outbound connections
- `network:listen` - Create servers/listeners
- `network:*` - All network operations

### Config Capabilities
- `config:read` - Read configuration values
- `config:write` - Modify configuration values

### System Capabilities
- `system:info` - Access system information
- `system:admin` - Perform administrative operations
- `system:resources` - Access resource monitoring

## Default Capability Sets by Permission Level

### System Level
System level has implicit access to all capabilities.

### User Level
```
file:read
file:write
plugin:read
plugin:execute
network:connect
config:read
config:write
system:info
```

### Restricted Level
```
file:read
plugin:read
config:read
```

## Implementation

Capabilities are checked in the security modules through the following methods:
- `validate_capability()` - Direct capability check
- `validate_operation()` - Maps operations to capabilities
- `check_permission()` - Low-level permission verification

## Best Practices

1. **Prefer Capability Checks**: Use fine-grained capability checks rather than permission levels when possible
2. **Explicitly Define All Capabilities**: Avoid implicit capabilities
3. **Follow Least Privilege**: Grant only the minimum capabilities required
4. **Use Namespaces Consistently**: Keep namespace definitions consistent
5. **Document New Capabilities**: Update this document when adding new capabilities

<version>1.0.0</version> 