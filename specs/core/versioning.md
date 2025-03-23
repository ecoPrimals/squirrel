---
title: Versioning Specification
version: 1.0.0
date: 2024-05-15
status: active
priority: high
---

# Versioning Specification

## Overview

This specification defines the versioning system and build information management for the Squirrel ecosystem. It establishes how version information is stored, retrieved, and presented to users and other components, ensuring consistency across the system while maintaining the core crate's minimal nature.

## Versioning Strategy

### Semantic Versioning

All components in the Squirrel ecosystem must follow [Semantic Versioning 2.0.0](https://semver.org/):

1. **Major Version**: Incremented for incompatible API changes
2. **Minor Version**: Incremented for backward-compatible functionality additions
3. **Patch Version**: Incremented for backward-compatible bug fixes

### Version Representation

Versions should be represented in the standard format:

```
MAJOR.MINOR.PATCH[-PRERELEASE][+BUILD]
```

Where:
- `MAJOR`, `MINOR`, and `PATCH` are non-negative integers
- `PRERELEASE` (optional) is a series of dot-separated identifiers (e.g., `alpha.1`)
- `BUILD` (optional) is build metadata such as commit hash or timestamp

### Version Compatibility

Version compatibility guarantees:

1. **Major Version**: No compatibility guaranteed between different major versions
2. **Minor Version**: Higher minor versions must be compatible with lower minor versions within the same major version
3. **Patch Version**: Pure bug fixes that maintain API compatibility

## Build Information

### Build Metadata Collection

Build information is collected at compile time using the `built` crate:

```rust
// build.rs
fn main() {
    built::write_built_file().expect("Failed to acquire build information");
}
```

### Core Build Information

The following build information must be captured:

1. **Package Version**: The package version from Cargo.toml
2. **Git Commit**: The commit hash (if built from a git repository)
3. **Build Timestamp**: When the build was created
4. **Rust Version**: The version of Rust used for compilation
5. **Target Triple**: The compilation target

### Build Information Module

Build information is exposed through a dedicated module:

```rust
/// Build information
pub mod build_info {
    /// The built info from the build script
    #[allow(clippy::multiple_unsafe_ops_per_block, clippy::wildcard_imports)]
    pub mod built_info {
        include!(concat!(env!("OUT_DIR"), "/built.rs"));
    }

    /// Get the version string with build information
    #[must_use]
    pub fn version() -> String {
        let version = built_info::PKG_VERSION;
        version.to_string()
    }
    
    /// Get the full version string with additional build metadata
    #[must_use]
    pub fn full_version() -> String {
        let version = built_info::PKG_VERSION;
        let git_hash = match option_env!("GIT_HASH") {
            Some(hash) => hash,
            None => built_info::GIT_COMMIT_HASH.unwrap_or("unknown"),
        };
        format!("{} ({})", version, git_hash)
    }
    
    /// Get the build date
    #[must_use]
    pub fn build_date() -> String {
        // Implementation details for build date
        // ...
    }
}
```

## Version Reporting

### Version API

The core crate must provide a simple API for accessing version information:

```rust
impl Core {
    /// Get the version string
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Get detailed version information
    #[must_use]
    pub fn version_info(&self) -> VersionInfo {
        VersionInfo {
            version: self.version.clone(),
            build_date: build_info::build_date(),
            git_commit: build_info::built_info::GIT_COMMIT_HASH.unwrap_or("unknown").to_string(),
            rust_version: build_info::built_info::RUSTC_VERSION.to_string(),
        }
    }
}
```

### VersionInfo Struct

A dedicated struct should represent detailed version information:

```rust
/// Detailed version information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionInfo {
    /// Version string (semantic version)
    pub version: String,
    /// Build date
    pub build_date: String,
    /// Git commit hash
    pub git_commit: String,
    /// Rust compiler version
    pub rust_version: String,
}
```

### Command Line Version Display

For command-line tools, display version information consistently:

```
squirrel-cli 1.2.3 (a1b2c3d)
Built: 2024-05-15
Rust: 1.75.0
```

### In-App Version Display

For graphical interfaces, display version information clearly but unobtrusively:

1. Basic version (`1.2.3`) in title bars or footers
2. Detailed version info in "About" dialogues or equivalent

## Version Compatibility Checking

### API Version Check

For plugin compatibility checking:

```rust
/// Check if the given version is compatible with the current version
pub fn is_compatible(required_version: &str, current_version: &str) -> bool {
    // Parse the versions
    let required = semver::Version::parse(required_version).unwrap_or_default();
    let current = semver::Version::parse(current_version).unwrap_or_default();
    
    // Check compatibility (same major version, current >= required)
    required.major == current.major && current >= required
}
```

### Plugin Version Requirements

Plugins must specify their required core version:

```rust
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    // ...
    pub required_core_version: String,
    // ...
}
```

## Workspace Version Management

### Shared Workspace Versions

Use workspace versioning in `Cargo.toml` to ensure all crates in the workspace share the same version:

```toml
[workspace]
members = ["crates/*"]
version = "1.2.3"

[workspace.package]
version = "1.2.3"
edition = "2021"
authors = ["DataScienceBioLab"]
```

Individual crates should reference the workspace version:

```toml
[package]
name = "squirrel-core"
version.workspace = true
edition.workspace = true
authors.workspace = true
```

## Implementation Status

- Semantic versioning adherence: **Implemented**
- Build information collection: **Implemented**
- Version API: **Implemented**
- Version compatibility checking: **Planned**
- Workspace version management: **Implemented**

## Future Enhancements

While maintaining minimalism, future enhancements may include:

1. Enhanced compatibility checking with more granular controls
2. Automatic version bumping and changelog generation tools
3. Component-specific version tracking for internal dependencies

## Conclusion

The versioning system provides a minimal but effective framework for managing versions and build information across the Squirrel ecosystem. By following these standards, components can ensure consistent version representation and reliable compatibility checking while keeping the core crate focused on its essential responsibilities. 