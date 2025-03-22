---
title: "Galaxy MCP Adapter Version Management"
description: "Versioning strategy and compatibility management for the Galaxy MCP adapter crate"
version: "0.1.0"
last_updated: "2025-03-27"
status: "draft"
owners:
  primary: ["DataScienceBioLab", "mcp-team"]
  reviewers: ["core-team", "api-team"]
---

# Galaxy MCP Adapter Version Management

## 1. Overview

This specification outlines the versioning strategy and compatibility management for the Galaxy MCP adapter crate. It covers crate versioning, dependency management, backward compatibility guarantees, and strategies for maintaining compatibility with both the core MCP crates and the Galaxy API.

## 2. Versioning Strategy

### 2.1 Semantic Versioning

The Galaxy MCP adapter crate follows [Semantic Versioning 2.0.0](https://semver.org/) principles:

**Format**: `MAJOR.MINOR.PATCH[-PRERELEASE][+BUILD]`

- **MAJOR**: Incompatible API changes
- **MINOR**: New functionality in a backward-compatible manner
- **PATCH**: Backward-compatible bug fixes
- **PRERELEASE**: Pre-release version identifier (e.g., `alpha.1`, `beta.2`)
- **BUILD**: Build metadata (e.g., `build.123`)

### 2.2 Version Progression

Example version progression for the Galaxy MCP adapter crate:

| Version | Description |
|---------|-------------|
| 0.1.0 | Initial development release |
| 0.2.0 | Additional features, still in development |
| 0.5.0 | Feature complete, pre-release |
| 0.9.0 | Release candidate |
| 1.0.0 | First stable release |
| 1.1.0 | New features added |
| 1.1.1 | Bug fixes to 1.1.0 |
| 2.0.0 | Breaking changes introduced |

### 2.3 Development Phases

- **0.x.y**: Development phase
  - APIs may change between minor versions
  - Not recommended for production use
  - Frequent updates and experimental features

- **1.x.y**: Stable phase
  - Stable API with backward compatibility guarantees
  - Production-ready with regular maintenance
  - New features without breaking changes

- **x.0.0**: Major releases
  - Can include breaking changes
  - Require update to documentation and examples
  - May require client code modifications

## 3. Dependency Management

### 3.1 Core Dependencies

| Dependency | Versioning Strategy | Update Frequency | Notes |
|------------|---------------------|------------------|-------|
| MCP crate | Pinned major version | On MCP crate releases | Core protocol implementation |
| Context crate | Pinned major version | On context crate releases | Context management functionality |
| `reqwest` | Pinned minor version | Quarterly assessment | HTTP client for Galaxy API |
| `serde` | Pinned minor version | Yearly assessment | Serialization framework |
| `tokio` | Pinned minor version | Yearly assessment | Async runtime |

### 3.2 Cargo.toml Example

```toml
[package]
name = "galaxy-mcp"
version = "0.1.0"
edition = "2021"
authors = ["DataScienceBioLab"]
description = "Galaxy adapter for the Machine Context Protocol"
# Other metadata...

[dependencies]
# Core dependencies from the parent project
mcp = { version = "0.12", path = "../mcp" }
context = { version = "0.9", path = "../context" }

# External dependencies
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.25", features = ["full"] }
log = "0.4"
thiserror = "1.0"
async-trait = "0.1"

[dev-dependencies]
mockito = "1.0"
tokio-test = "0.4"
```

### 3.3 Dependency Update Policy

1. **Security Updates**: Apply immediately, may require patch version bump
2. **Bug Fixes**: Assess impact, apply if beneficial in next patch release
3. **Feature Updates**: Evaluate for minor releases, ensure compatibility
4. **Major Version Updates**: Plan migration strategy, align with next major release

## 4. Core MCP Compatibility

### 4.1 Core MCP Version Targeting

The Galaxy MCP adapter targets specific versions of the core MCP crate:

```rust
// In lib.rs
#[cfg(not(mcp_version_check))]
compile_error!("Requires mcp 0.12.x or compatible version");

// Version compatibility check
#[cfg(mcp_version_check)]
const _: () = {
    const MIN_MCP_VERSION: (u32, u32) = (0, 12);
    const MAX_MCP_VERSION: (u32, u32) = (0, 13);
    const CURRENT_MCP_VERSION: (u32, u32) = (
        mcp::VERSION_MAJOR,
        mcp::VERSION_MINOR,
    );
    
    // Static assertions to ensure compatibility
    static_assertions::const_assert!(CURRENT_MCP_VERSION.0 >= MIN_MCP_VERSION.0);
    static_assertions::const_assert!(
        CURRENT_MCP_VERSION.0 > MIN_MCP_VERSION.0 || 
        CURRENT_MCP_VERSION.1 >= MIN_MCP_VERSION.1
    );
    static_assertions::const_assert!(CURRENT_MCP_VERSION.0 < MAX_MCP_VERSION.0 || 
        (CURRENT_MCP_VERSION.0 == MAX_MCP_VERSION.0 && 
         CURRENT_MCP_VERSION.1 < MAX_MCP_VERSION.1)
    );
};
```

### 4.2 Core MCP Feature Compatibility

The adapter maintains a compatibility table for MCP features:

| MCP Feature | Min Version | Status in Adapter | Notes |
|-------------|-------------|-------------------|-------|
| Tool Discovery | 0.10.0 | Implemented | Core feature |
| Tool Execution | 0.10.0 | Implemented | Core feature |
| Job Status | 0.10.0 | Implemented | Core feature |
| Job Cancellation | 0.11.0 | Implemented | Core feature |
| Advanced Parameter Types | 0.12.0 | Implemented | Enhanced features |
| Interactive Tools | 0.13.0 | Planned | Future feature |

### 4.3 Conditional Features

The adapter uses feature flags to handle conditional MCP functionality:

```rust
#[cfg(feature = "mcp_interactive_tools")]
pub mod interactive {
    // Implementation for MCP 0.13+ interactive tools feature
}

pub fn create_adapter() -> GalaxyAdapter {
    let adapter = GalaxyAdapter::new();
    
    #[cfg(feature = "mcp_interactive_tools")]
    adapter.enable_interactive_tools();
    
    adapter
}
```

## 5. Galaxy API Compatibility

### 5.1 Galaxy API Version Support

The adapter supports specific Galaxy API versions:

| Galaxy Version | API Version | Support Status | EOL Date |
|----------------|-------------|----------------|----------|
| 23.0 | 2 | Full Support | 2026-01 |
| 22.1 | 2 | Full Support | 2025-05 |
| 22.0 | 2 | Maintenance | 2025-01 |
| 21.x | 1 | Limited Support | 2024-05 |
| < 21.0 | 1 | Not Supported | - |

### 5.2 Galaxy API Version Detection

The adapter checks Galaxy API version at runtime:

```rust
impl GalaxyAdapter {
    pub async fn new(config: GalaxyAdapterConfig) -> Result<Self, GalaxyAdapterError> {
        let client = reqwest::Client::new();
        
        // Detect Galaxy version
        let version_info = detect_galaxy_version(&client, &config.galaxy_url).await?;
        
        if !is_supported_version(&version_info) {
            log::warn!(
                "Galaxy version {} may not be fully supported. Recommended versions: 22.0+",
                version_info.version
            );
        }
        
        // Create adapter with version-specific handlers
        let adapter = match version_info.api_version {
            1 => Self::create_v1_adapter(client, config, version_info),
            2 => Self::create_v2_adapter(client, config, version_info),
            _ => {
                log::warn!("Unknown API version {}, using latest supported version", version_info.api_version);
                Self::create_v2_adapter(client, config, version_info)
            }
        };
        
        Ok(adapter)
    }
}
```

### 5.3 API Differences Handling

The adapter handles differences between Galaxy API versions:

```rust
#[derive(Debug)]
enum GalaxyApiVersion {
    V1,
    V2,
}

impl GalaxyAdapter {
    async fn execute_tool(&self, tool_id: &str, inputs: &HashMap<String, ToolInput>) -> Result<String, Error> {
        // Format inputs according to version-specific format
        let formatted_inputs = match self.api_version {
            GalaxyApiVersion::V1 => self.format_v1_inputs(inputs),
            GalaxyApiVersion::V2 => self.format_v2_inputs(inputs),
        };
        
        // Call appropriate API endpoint
        let job_id = match self.api_version {
            GalaxyApiVersion::V1 => {
                self.client.post(&format!("{}/tools/{}/execute", self.base_url, tool_id))
                    .json(&formatted_inputs)
                    .send()
                    .await?
                    .json::<V1JobResponse>()
                    .await?
                    .id
            },
            GalaxyApiVersion::V2 => {
                self.client.post(&format!("{}/tools", self.base_url))
                    .json(&ToolExecutionRequest {
                        tool_id: tool_id.to_string(),
                        inputs: formatted_inputs,
                    })
                    .send()
                    .await?
                    .json::<V2JobResponse>()
                    .await?
                    .job.id
            },
        };
        
        Ok(job_id)
    }
}
```

## 6. Backward Compatibility

### 6.1 Public API Stability

The adapter follows these principles for public API stability:

1. **Public API**: Functions, structs, and traits marked as `pub` are part of the stable API
2. **Internal API**: Items marked with `pub(crate)` or in modules marked as `#[doc(hidden)]` may change
3. **Backward Compatibility**: Public API is backward compatible within the same major version
4. **Deprecation**: Functions marked with `#[deprecated]` may be removed in the next major version

Example deprecation pattern:

```rust
/// Execute a Galaxy tool
///
/// # Deprecated
/// Use `execute_tool_with_params` instead, which provides more flexibility
#[deprecated(since = "1.2.0", note = "Use execute_tool_with_params instead")]
pub async fn execute_tool(&self, tool_id: &str, inputs: &HashMap<String, String>) -> Result<String, Error> {
    // Convert to new format and call new function
    let params = inputs.iter()
        .map(|(k, v)| (k.clone(), ToolInput::String(v.clone())))
        .collect();
    
    self.execute_tool_with_params(tool_id, &params).await
}

/// Execute a Galaxy tool with typed parameters
pub async fn execute_tool_with_params(
    &self, 
    tool_id: &str, 
    inputs: &HashMap<String, ToolInput>
) -> Result<String, Error> {
    // Implementation
}
```

### 6.2 Migration Guides

For each non-backward compatible change, the adapter provides migration guides:

- Located in the `docs/migrations/` directory
- Named according to the version: `v1-to-v2.md`
- Includes code examples for before and after
- Lists all breaking changes and their replacements

Example migration guide structure:

```markdown
# Migration Guide: v1.x to v2.0

This guide helps you migrate from Galaxy MCP adapter v1.x to v2.0.

## Breaking Changes

### 1. Tool Execution API

**Before (v1.x):**
```rust
let job_id = adapter.execute_tool("tool_id", &inputs).await?;
```

**After (v2.0):**
```rust
let execution = adapter.create_execution("tool_id")
    .with_inputs(&inputs)
    .execute()
    .await?;
let job_id = execution.job_id;
```

### 2. Configuration Changes

**Before (v1.x):**
```rust
let config = GalaxyAdapterConfig {
    url: "https://galaxy.example.org".to_string(),
    timeout: 30,
};
```

**After (v2.0):**
```rust
let config = GalaxyAdapterConfig::builder()
    .with_url("https://galaxy.example.org")
    .with_timeout(Duration::from_secs(30))
    .build();
```
```

### 6.3 Version Checking

The adapter provides utility functions for version checking:

```rust
/// Check if the current adapter version is compatible with the given version requirements
pub fn is_compatible_with(min_version: &str, max_version: &str) -> bool {
    let current = semver::Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
    let min = semver::Version::parse(min_version).unwrap();
    let max = semver::Version::parse(max_version).unwrap();
    
    current >= min && current < max
}

/// Returns the current adapter version
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
```

## 7. Upgrading Dependencies

### 7.1 Dependency Upgrade Process

When upgrading dependencies, follow this process:

1. **Review Changes**: Read release notes of the dependency
2. **Compatibility Check**: Verify compatibility with other dependencies
3. **Update Cargo.toml**: Modify version constraints
4. **Update Code**: Make necessary code changes
5. **Run Tests**: Ensure all tests pass
6. **Update Documentation**: Update documentation if API changes
7. **Version Bump**: Bump adapter version according to impact

### 7.2 Galaxy API Updates

For Galaxy API changes:

1. **Monitor Release Notes**: Track Galaxy project releases
2. **Test with Beta/RC**: Test adapter with Galaxy pre-releases
3. **Update API Mappings**: Update API endpoint mappings and data structures
4. **Version-Specific Code**: Implement version-specific handlers if needed
5. **Update Version Table**: Update supported Galaxy versions table

## 8. Publishing Process

### 8.1 Pre-Release Checklist

Before publishing a new version:

- [ ] Update CHANGELOG.md with all changes
- [ ] Update version in Cargo.toml
- [ ] Ensure documentation is up to date
- [ ] Run all tests, including with different Galaxy versions
- [ ] Review public API for unintentional breaking changes
- [ ] Verify compatibility with supported Galaxy versions
- [ ] Check compatibility with dependent crates

### 8.2 Publishing Steps

1. Update version in `Cargo.toml`
2. Commit changes: `git commit -am "Bump version to X.Y.Z"`
3. Create Git tag: `git tag -a vX.Y.Z -m "Version X.Y.Z"`
4. Run final verification: `cargo package --list`
5. Publish to crates.io: `cargo publish`
6. Push tag: `git push origin vX.Y.Z`
7. Create GitHub release with release notes

### 8.3 Experimental Features

For experimental features, use feature flags:

```toml
[features]
default = ["standard-features"]
standard-features = []
experimental-workflow-builder = []
interactive-tools = []
```

```rust
#[cfg(feature = "experimental-workflow-builder")]
pub mod workflow_builder {
    // Experimental API that may change even in minor versions
    #[doc(hidden)]
    pub struct WorkflowBuilder {
        // Implementation
    }
}
```

## 9. Versioning Tools

### 9.1 Version Information in Code

The adapter exposes version information at runtime:

```rust
/// Version information for the Galaxy MCP adapter
pub struct VersionInfo {
    /// Adapter version (Cargo package version)
    pub adapter_version: &'static str,
    /// Minimum supported MCP version
    pub min_mcp_version: &'static str,
    /// Maximum supported MCP version (exclusive)
    pub max_mcp_version: &'static str,
    /// Minimum supported Galaxy version
    pub min_galaxy_version: &'static str,
    /// Build timestamp (set at compile time)
    pub build_timestamp: &'static str,
    /// Git commit hash (if available)
    pub git_commit: Option<&'static str>,
}

impl GalaxyAdapter {
    /// Get version information
    pub fn version_info() -> VersionInfo {
        VersionInfo {
            adapter_version: env!("CARGO_PKG_VERSION"),
            min_mcp_version: "0.12.0",
            max_mcp_version: "0.13.0",
            min_galaxy_version: "22.0.0",
            build_timestamp: env!("BUILD_TIMESTAMP"),
            git_commit: option_env!("GIT_COMMIT"),
        }
    }
}
```

### 9.2 Version Command

The adapter includes a version utility:

```rust
/// Print version information to stdout
pub fn print_version() {
    let info = GalaxyAdapter::version_info();
    println!("Galaxy MCP Adapter v{}", info.adapter_version);
    println!("Compatible with MCP v{} to v{}", info.min_mcp_version, info.max_mcp_version);
    println!("Supports Galaxy v{} and newer", info.min_galaxy_version);
    println!("Build timestamp: {}", info.build_timestamp);
    if let Some(commit) = info.git_commit {
        println!("Git commit: {}", commit);
    }
}
```

## 10. Related Specifications

- [Galaxy MCP Integration Plan](galaxy-mcp-integration.md)
- [Configuration Management](configuration-management.md)
- [API Mapping](api-mapping.md)

<version>0.1.0</version> 