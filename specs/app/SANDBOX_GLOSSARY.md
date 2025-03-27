# Plugin Sandbox System Glossary

This document provides definitions for technical terms and concepts used throughout the plugin sandbox system documentation. Use this as a reference when working with the CrossPlatformSandbox API.

## Core Concepts

### Sandbox
A controlled environment that restricts what a plugin can do by limiting its access to system resources, files, and capabilities. The sandbox provides isolation to prevent plugins from interfering with the host system or other plugins.

### CrossPlatformSandbox
The main API provided by the plugin system that unifies platform-specific sandbox implementations into a consistent interface. It automatically selects the appropriate implementation based on the platform.

### Platform Capability
A feature or technology available on a specific platform for creating secure sandboxes. Examples include Windows Job Objects, Linux seccomp, or macOS App Sandbox. The CrossPlatformSandbox detects available capabilities at runtime.

### Security Context
A configuration object that defines the permissions, allowed paths, capabilities, and resource limits for a plugin. It determines what a plugin can and cannot do within the sandbox.

### Permission Level
A predefined set of permissions that determines the overall access level of a plugin:
- **Restricted**: Minimal access, tightly controlled
- **User**: Standard user-level permissions
- **Admin**: Elevated permissions for trusted plugins

### Capability
A specific permission or ability that can be granted to a plugin, such as "file:read" or "network:client". Capabilities are more granular than permission levels and allow for fine-tuning access.

### Resource Monitor
A component that tracks and limits the resources (CPU, memory, etc.) used by plugins.

### Graceful Degradation
The system's ability to fall back to alternative implementations when a preferred sandbox technology is not available, ensuring functionality across different platforms and environments.

## Platform-Specific Terms

### Windows-Specific

#### Job Object
A Windows kernel object that allows groups of processes to be managed as a unit. Used for process isolation and resource limiting in the Windows sandbox implementation.

#### Desktop Isolation
A Windows security feature that creates a separate desktop for sandboxed processes, preventing them from interacting with the user's desktop or capturing screen content.

#### Process Priority Control
A Windows feature for controlling the CPU scheduling priority of processes, used to prevent plugins from consuming excessive CPU resources.

#### Integrity Level
A Windows security feature that assigns "trust levels" to processes, determining what system resources they can access. The sandbox uses this to restrict plugin access.

### Linux-Specific

#### Seccomp
Short for "secure computing mode," a Linux kernel feature that restricts the system calls a process can make, thereby limiting its ability to interact with the kernel.

#### Seccomp-BPF
An extension of seccomp that uses Berkeley Packet Filter (BPF) programs to filter system calls based on their arguments, providing more granular control.

#### Namespace
A Linux kernel feature that partitions kernel resources, making it appear to processes within a namespace that they have their own isolated instance of the resource. Types include:
- **PID Namespace**: Isolates process IDs
- **Network Namespace**: Isolates network resources
- **Mount Namespace**: Isolates filesystem mount points
- **User Namespace**: Isolates user and group IDs

#### cgroups (Control Groups)
A Linux kernel feature for limiting, accounting for, and isolating resource usage (CPU, memory, disk I/O, network, etc.) of process groups.

#### cgroups v2
The unified hierarchy version of cgroups, providing improved resource control and monitoring capabilities. More consistent and controllable than the legacy cgroups v1.

### macOS-Specific

#### App Sandbox
Apple's sandboxing technology that limits what operations an application can perform on macOS, including restricting file access, network use, and hardware access.

#### TCC (Transparency, Consent, and Control)
Apple's privacy framework that requires user consent for applications to access sensitive data or device capabilities like the camera, microphone, or location services.

#### Entitlements
Digital property lists that grant specific capabilities to macOS applications, such as accessing protected resources or using system services.

#### Seatbelt
The low-level sandboxing technology in macOS, which uses sandbox profiles to restrict what actions a process can perform.

## Security Terms

### Path Validation
The process of checking if a plugin has permission to access a specific file or directory path before allowing the operation.

### Capability Validation
The process of checking if a plugin has been granted a specific capability before allowing an operation that requires that capability.

### Isolation Boundary
The security perimeter established by the sandbox that separates the plugin from the host system and other plugins.

### Privilege Escalation
An attack where a plugin attempts to gain higher permission levels than it should have. The sandbox is designed to prevent this.

### Resource Throttling
A mechanism that limits the rate at which a plugin can consume resources (CPU, memory, network, etc.) to prevent denial-of-service attacks.

### Security Context Inheritance
The process by which child processes created by a plugin inherit the security restrictions of their parent, ensuring that the isolation boundary is maintained.

## API-Related Terms

### Feature Application
The process of applying a specific sandbox feature to a plugin using the `apply_feature` or `apply_feature_with_degradation` methods.

### Feature Degradation
The automatic fallback to less optimal but still functional implementations when the preferred implementation is not available on the current platform.

### Platform Detection
The process by which the CrossPlatformSandbox identifies the current operating system and available security features.

### Error Standardization
The process of converting platform-specific error messages and codes into consistent, cross-platform error formats that are easier to understand and handle.

### Plugin Identifier
A unique UUID assigned to each plugin that is used to track its sandbox, security context, and resource usage.

### Resource Usage Metrics
Measurements of a plugin's resource consumption, including memory usage, CPU usage, disk I/O, and network activity.

## Implementation Details

### SandboxError
A standardized error type that represents various failure modes in the sandbox system, including permission denied, resource limits exceeded, and platform-specific errors.

### SecurityContextBuilder
A builder pattern implementation that helps construct valid security contexts with a fluent API.

### Platform Implementation
The platform-specific code that implements the sandbox functionality for a particular operating system (Windows, Linux, or macOS).

### Sandbox Lifecycle
The series of stages a sandbox goes through, including creation, configuration, operation, and destruction.

### Hook System
A mechanism that allows for the insertion of custom code at specific points in the sandbox lifecycle, enabling extensibility and customization.

### Secure IPC (Inter-Process Communication)
Methods for safely communicating between the host application and sandboxed plugins without compromising the isolation boundary.

## References

- [Platform Capabilities API](PLATFORM_CAPABILITIES_API.md)
- [Platform Usage Examples](PLATFORM_USAGE_EXAMPLES.md)
- [Implementation Progress](IMPLEMENTATION_PROGRESS.md)
- [Task Tracking](TASK_TRACKING.md) 