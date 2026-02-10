# Socket Registry Specification

**Version**: 1.0  
**Date**: January 14, 2026  
**Status**: Proposed Standard

## Overview

The Socket Registry enables **capability-based discovery** of Unix sockets, eliminating hardcoded primal names and enabling true runtime discovery.

## Problem Solved

**Before** (Hardcoded primal names):
```rust
// ❌ Hardcodes "songbird" - what if it evolves?
let socket = "/run/user/1000/songbird-nat0.sock";
```

**After** (Capability-based):
```rust
// ✅ Request by capability - any primal can provide it
let client = UnixSocketClient::connect_by_capability("orchestration").await?;
```

## Registry File

### Location

`/run/user/<uid>/socket-registry.json`

**Examples**:
- `/run/user/1000/socket-registry.json`
- `/run/user/0/socket-registry.json` (root)

### Format

```json
{
  "orchestration": "/run/user/1000/songbird-nat0.sock",
  "security": "/run/user/1000/beardog-nat0.sock",
  "storage": "/run/user/1000/nestgate-nat0.sock",
  "compute": "/run/user/1000/toadstool-nat0.sock",
  "ai": "/run/user/1000/squirrel-squirrel.sock",
  "core": "/run/user/1000/biomeos-nucleus.sock"
}
```

### Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "patternProperties": {
    "^[a-z][a-z0-9_-]*$": {
      "type": "string",
      "pattern": "^/.*\\.sock$"
    }
  },
  "additionalProperties": false
}
```

**Key format**: Lowercase capability name (alphanumeric, hyphens, underscores)  
**Value format**: Absolute path to Unix socket ending in `.sock`

## Discovery Priority

When discovering a capability socket:

1. **Environment Variable** (highest priority)
   ```bash
   export ORCHESTRATION_SOCKET=/custom/path.sock
   ```

2. **Socket Registry**
   - Read `/run/user/<uid>/socket-registry.json`
   - Look up capability key
   
3. **Legacy Convention** (deprecated)
   - Map capability to conventional primal name
   - Use XDG path: `/run/user/<uid>/<primal>-<family>.sock`

4. **Error** (no hardcoded fallback!)

## Capability Names

### Standard Capabilities

| Capability | Description | Typical Provider |
|------------|-------------|------------------|
| `orchestration` | Service mesh coordination | Songbird |
| `security` | Authentication & authorization | BearDog |
| `storage` | Data persistence | NestGate |
| `compute` | Workload execution | Toadstool |
| `ai` | AI inference & intelligence | Squirrel |
| `core` | Ecosystem coordination | biomeOS |
| `communication` | Messaging & events | PetalTongue |
| `encryption` | Cryptographic services | rhizoCrypt |
| `distributed` | Distributed storage | LoamSpine |
| `federation` | Cross-biome federation | SweetGrass |

### Custom Capabilities

Format: `vendor.capability`

Examples:
- `acme.data-pipeline`
- `custom.ml-training`
- `internal.audit-log`

## Registry Maintenance

### Who Maintains It?

**Orchestration Layer** (typically Songbird or biomeOS):

```rust
// When a primal registers with capabilities
async fn register_primal(primal: PrimalInfo) {
    let registry_path = format!("/run/user/{}/socket-registry.json", getuid());
    
    // Load existing registry
    let mut registry = load_registry(&registry_path).await?;
    
    // Update capability mappings
    for capability in &primal.capabilities {
        registry.insert(capability.clone(), primal.socket_path.clone());
    }
    
    // Save atomically
    save_registry(&registry_path, &registry).await?;
}
```

### Atomic Updates

**Requirements**:
1. Write to temporary file first
2. Atomic rename to final location
3. File permissions: 0600 (user-only)
4. Handle concurrent updates (file locking)

**Example**:
```rust
use std::fs;
use std::os::unix::fs::PermissionsExt;

// Write to temp file
let temp_path = format!("{}.tmp", registry_path);
fs::write(&temp_path, json)?;

// Set permissions
let mut perms = fs::metadata(&temp_path)?.permissions();
perms.set_mode(0o600);
fs::set_permissions(&temp_path, perms)?;

// Atomic rename
fs::rename(&temp_path, &registry_path)?;
```

## Usage Examples

### Capability-Based Connection

```rust
// ✅ Discover orchestration by capability
let orchestration = UnixSocketClient::connect_by_capability("orchestration").await?;
orchestration.register_service(registration).await?;

// ✅ Discover security by capability
let security = UnixSocketClient::connect_by_capability("security").await?;
security.validate_credentials(creds).await?;

// ❌ DON'T hardcode primal names
// let songbird = connect_to_songbird().await?; // WRONG!
```

### Environment Variable Override

```bash
# Override specific capability
export ORCHESTRATION_SOCKET=/custom/songbird.sock

# Run primal - it will use override
./squirrel
```

### Legacy Compatibility

If registry doesn't exist, falls back to convention:

```
orchestration → /run/user/<uid>/songbird-<family>.sock
security     → /run/user/<uid>/beardog-<family>.sock
storage      → /run/user/<uid>/nestgate-<family>.sock
```

**Note**: This is **deprecated** and should only be used during migration.

## Migration Path

### Phase 1: Add Registry Support (Current)

```rust
// Code supports both registry and legacy
let socket = discover_socket_by_capability("orchestration")?;
// Tries: env var → registry → legacy → error
```

### Phase 2: Deploy Registry

```bash
# Orchestration layer creates registry
cat > /run/user/1000/socket-registry.json << EOF
{
  "orchestration": "/run/user/1000/songbird-nat0.sock",
  "security": "/run/user/1000/beardog-nat0.sock"
}
EOF
```

### Phase 3: Remove Legacy Support

```rust
// Remove legacy_capability_to_socket() function
// Error if not in registry or env var
```

## Security Considerations

### File Permissions

- **Owner**: User running primals (e.g., user 1000)
- **Permissions**: `0600` (read/write user only)
- **No world-readable**: Prevents unauthorized socket discovery

### Validation

Before using socket path:
1. Verify file exists
2. Verify it's a Unix socket (not regular file)
3. Verify permissions are secure
4. Test connection

```rust
use std::os::unix::fs::FileTypeExt;

fn validate_socket(path: &str) -> Result<(), Error> {
    let metadata = fs::metadata(path)?;
    
    if !metadata.file_type().is_socket() {
        return Err("Not a Unix socket".into());
    }
    
    // Additional security checks...
    Ok(())
}
```

## Benefits

### 1. Zero Hardcoding

No primal names in code - only capability requests.

### 2. Evolution-Friendly

If Songbird evolves or is replaced, just update registry:
```json
{
  "orchestration": "/run/user/1000/new-mesh-v2.sock"
}
```

Code doesn't need to change!

### 3. Multi-Provider Support

Multiple primals can provide same capability:
```json
{
  "orchestration": "/run/user/1000/songbird-nat0.sock",
  "orchestration-backup": "/run/user/1000/songbird-nat1.sock"
}
```

### 4. Testing

Easy to override for testing:
```bash
export ORCHESTRATION_SOCKET=/tmp/mock-orchestration.sock
cargo test
```

## Standards Compliance

- ✅ **XDG Base Directory**: Uses `/run/user/<uid>/`
- ✅ **Unix Socket Standards**: Paths end in `.sock`
- ✅ **JSON Format**: Standard, parseable, extensible
- ✅ **Infant Primal Pattern**: Zero hardcoded knowledge

## References

- TRUE PRIMAL principles: Port-free, capability-based
- XDG Base Directory Specification
- Unix Domain Socket Best Practices
- biomeOS Atomic Architecture Standards

---

**Status**: Specification complete, implementation in progress  
**Next**: Update Songbird/biomeOS to maintain registry

