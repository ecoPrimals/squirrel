# Seccomp Filtering Implementation

This document provides an overview of the seccomp filtering implementation for the Linux sandbox.

## Overview

The seccomp (secure computing) implementation provides syscall filtering for Linux sandboxes, allowing fine-grained control over what system calls a sandboxed plugin can make. This is implemented using:

1. `libseccomp` for generating BPF programs
2. Argument-based filtering for more precise control
3. Integration with security contexts for permission-based filtering
4. Real-world usage profiles for common application types
5. Capability-based customization for fine-grained security control

## Key Components

### SeccompFilterBuilder

The main interface for creating and configuring seccomp filters:

```rust
let mut builder = SeccompFilterBuilder::new(SeccompAction::Kill);

// Add allowed syscalls
builder.add_rule(SyscallRule::new("read", SeccompAction::Allow));
builder.add_rule(SyscallRule::new("write", SeccompAction::Allow));

// Add a group of related rules
builder.add_rule_group("file_read");

// Use a real-world test suite configuration
builder.real_world_test_suite(plugin_id, "web_browser");

// Customize based on capabilities
let capabilities = ["fs.read", "net.client"].iter().cloned().collect();
builder.customize_for_capabilities(capabilities);

// Generate a BPF file
builder.generate_bpf_file(path)?;

// Apply to a process
builder.apply_to_process(pid)?;

// Check if a rule exists
if !builder.has_rule("socket") {
    builder.add_rule(SyscallRule::new("socket", SeccompAction::Allow));
}
```

### SyscallRule

Defines rules for specific syscalls, including argument filtering:

```rust
// Allow file open only for specific paths
let rule = SyscallRule::new("open", SeccompAction::Allow)
    .with_arg_filter(0, ArgFilter::path_prefix("/tmp/"));

// Allow specific socket types
let socket_rule = SyscallRule::new("socket", SeccompAction::Allow)
    .with_arg_filter(0, ArgFilter::equal(2))  // AF_INET
    .with_arg_filter(1, ArgFilter::equal(1)); // SOCK_STREAM
```

### SeccompAction

Defines the action to take when a syscall matches a rule:

- `Allow`: Allow the syscall to proceed
- `Kill`: Kill the process (with SIGSYS)
- `Trap`: Send SIGSYS to the process
- `Log`: Allow but log the syscall (useful for debugging)
- `Errno`: Return a specific error code

### ArgFilter

Provides filtering based on syscall arguments with enhanced helper methods:

```rust
// Equality/inequality comparisons
ArgFilter::equal(42)
ArgFilter::not_equal(0)

// Range checking
ArgFilter::greater_than(1000)
ArgFilter::less_than(4096)
ArgFilter::in_range(100, 200)

// Bit operations
ArgFilter::masked_equal(0xff00, 0x1200)

// Path operations
ArgFilter::path_prefix("/home/user/")

// Setting argument index
ArgFilter::equal(42).with_arg_index(0)
```

## Rule Groups and Real-World Profiles

The implementation includes pre-configured rule groups and real-world application profiles:

### Rule Groups

```rust
// Add common rule groups
builder.add_rule_group("file_read");     // Basic file read operations
builder.add_rule_group("file_write");    // File write operations
builder.add_rule_group("network_client"); // Client network operations
builder.add_rule_group("network_server"); // Server network operations
builder.add_rule_group("process_management"); // Process control
builder.add_rule_group("essential");     // Essential system operations
```

### Real-World Usage Profiles

```rust
// Configure for common application types
builder.real_world_test_suite(plugin_id, "web_browser");   // Web browser profile
builder.real_world_test_suite(plugin_id, "file_processor"); // File processing app
builder.real_world_test_suite(plugin_id, "web_server");    // Web server profile
builder.real_world_test_suite(plugin_id, "database");      // Database profile
```

## Capability-Based Customization

Filters can be customized based on a set of capabilities:

```rust
// Create a set of capabilities
let mut capabilities = HashSet::new();
capabilities.insert("fs.read".to_string());
capabilities.insert("net.client".to_string());
capabilities.insert("proc.create".to_string());

// Customize filter based on capabilities
builder.customize_for_capabilities(capabilities);
```

Common capabilities include:
- `fs.read`: File system read operations
- `fs.write`: File system write operations
- `net.client`: Network client operations
- `net.server`: Network server operations
- `proc.create`: Process creation
- `sys.admin`: System administration
- `debug`: Debugging capabilities
- `net.raw`: Raw network access

## Integration with Security Contexts

Seccomp filters can be generated based on security contexts:

```rust
let context = SecurityContext::new(PermissionLevel::Restricted, ResourceLimits::default());
let builder = SeccompFilterBuilder::from_security_context(&context);
```

Different permission levels result in different allowed syscalls:

- `System`: Most permissive, allows most syscalls
- `User`: Moderate restrictions, blocks dangerous syscalls
- `Restricted`: Most restrictive, allows only essential syscalls

## Using in the Linux Sandbox

The Linux sandbox integration automatically handles:

1. Generating seccomp filters based on the plugin's security context
2. Applying the filter to all processes in the sandbox
3. Loading filters from BPF files when needed
4. Customizing filters based on plugin capabilities

## Example: Creating a Custom Filter for a Web Server

```rust
// Create a builder with a default deny action
let mut builder = SeccompFilterBuilder::new(SeccompAction::Errno(EPERM));

// Add essential syscalls
builder.add_rule_group("essential");

// Add file operations with path restrictions
builder.add_rule(SyscallRule::new("open", SeccompAction::Allow)
    .with_arg_filter(0, ArgFilter::path_prefix("/var/www/")));
builder.add_rule(SyscallRule::new("openat", SeccompAction::Allow)
    .with_arg_filter(1, ArgFilter::path_prefix("/var/www/")));
builder.add_rule_group("file_read");

// Add network server capabilities
builder.add_rule_group("network_server");

// Allow socket operations with restrictions
builder.add_rule(SyscallRule::new("socket", SeccompAction::Allow)
    .with_arg_filter(0, ArgFilter::equal(2))  // AF_INET
    .with_arg_filter(1, ArgFilter::in_range(1, 3))); // SOCK_STREAM, SOCK_DGRAM, SOCK_RAW

// Generate and apply the filter
let bpf_path = Path::new("/tmp/webserver_filter.bpf");
builder.generate_bpf_file(&bpf_path)?;
builder.apply_to_process(pid)?;
```

## Testing

The implementation includes comprehensive tests:

- Unit tests for all filter builder and arg filter components
- Integration tests for real-world usage scenarios
- Capability-based customization tests
- Live process tests to verify filter behavior

Run the tests with:

```bash
cargo test --package squirrel-app --test integration_tests -- seccomp_tests
```

## Fallback Mechanism

For systems without proper seccomp tools, a skeleton BPF program is provided as a fallback. This provides basic filtering but without the advanced argument filtering capabilities.

## Debugging

To help with debugging seccomp issues:

1. Use `SeccompAction::Log` to log syscalls without blocking them
2. Enable debug logging to see which syscalls are filtered
3. Check the kernel logs (`dmesg`) for seccomp violation messages
4. Use `strace` to trace syscalls in test processes

## Platform Compatibility

This implementation is Linux-specific and requires:

- Linux kernel 3.5 or newer
- `libseccomp` development libraries

The code includes platform checks to ensure it's only used on Linux.

## Performance Considerations

Seccomp filters incur a slight overhead on each syscall. To optimize performance:

1. Use the minimal set of rules required for your application
2. Prefer rule groups over individual rules when possible
3. Use the most restrictive filter that still allows your application to function
4. Test with real-world workloads to ensure acceptable performance 