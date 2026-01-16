# TRUE PRIMAL Evolution - Unix Socket Migration

**Date**: January 14, 2026  
**Status**: 🔄 **IN PROGRESS**  
**Priority**: HIGH (blocks full NUCLEUS integration)

## 🚨 Issue Identified

Squirrel is using HTTP/reqwest for inter-primal communication, violating TRUE PRIMAL architecture.

### Current Violations

1. ❌ **HTTP for inter-primal communication** - Should use Unix sockets
2. ❌ **Hardcoded HTTP URLs** (`http://localhost:8080`)
3. ❌ **reqwest HTTP client** - Should use JSON-RPC over Unix socket
4. ❌ **Port-based communication** - Should be port-free

### What's Working ✅

- ✅ Squirrel exposes JSON-RPC Unix socket: `/tmp/squirrel-{node_id}.sock`
- ✅ biomeOS can discover and connect via this socket
- ✅ HTTP API available for external/human access (optional)
- ✅ Unix socket standards compliance (`rpc/unix_socket.rs`)

## 🎯 Solution Implemented

### New Module: `unix_socket_client.rs`

**Location**: `crates/main/src/biomeos_integration/unix_socket_client.rs`

**Capabilities** (CAPABILITY-BASED - NO HARDCODED PRIMAL NAMES!):
```rust
// ✅ Connect by capability, not primal name
let orchestration = UnixSocketClient::connect_by_capability("orchestration").await?;
orchestration.register_service(registration).await?;

// ✅ Connect by capability
let security = UnixSocketClient::connect_by_capability("security").await?;

// ❌ DON'T hardcode primal names
// let songbird = connect_to_songbird().await?; // WRONG!
```

**Discovery Order**:
1. **Capability-based env var** (`ORCHESTRATION_SOCKET`, not `SONGBIRD_SOCKET`!)
2. **Socket registry** (`/run/user/<uid>/socket-registry.json`)
3. **Legacy convention** (deprecated, maps capability to conventional name)
4. **Error** (NO hardcoded fallback!)

### Architecture

```
┌─────────────┐  Unix Socket   ┌─────────────┐
│   Squirrel  │ ────JSON-RPC──> │  Songbird   │
│             │ <──────────────  │  (Mesh)     │
└─────────────┘                 └─────────────┘
      │
      │ Unix Socket
      │
      ▼
┌─────────────┐
│   BearDog   │
│  (Security) │
└─────────────┘
```

## 📋 Migration Plan

### Phase 1: Infrastructure ✅ COMPLETE

- [x] Create `unix_socket_client.rs` module
- [x] Implement JSON-RPC 2.0 over Unix sockets
- [x] Add discovery logic (env var → XDG → error)
- [x] Add Songbird connection methods
- [x] Add BearDog connection methods
- [x] Add biomeOS connection methods

### Phase 2: Migration 🔄 NEXT

- [ ] Update service registration to use Unix socket client
- [ ] Deprecate HTTP-based `ecosystem_client.rs`
- [ ] Update all inter-primal calls to use Unix sockets
- [ ] Add BearDog security integration
- [ ] Test with biomeOS NUCLEUS deployment

### Phase 3: Cleanup 📋 PLANNED

- [ ] Remove HTTP client for inter-primal comm
- [ ] Keep HTTP API for external access only
- [ ] Update documentation
- [ ] Verify TRUE PRIMAL compliance

## 🧬 TRUE PRIMAL Compliance

### Current State

| Component | Status | Notes |
|-----------|--------|-------|
| JSON-RPC socket (serving) | ✅ Working | `/tmp/squirrel-{node}.sock` |
| JSON-RPC socket (client) | ✅ Implemented | `unix_socket_client.rs` |
| Songbird discovery | 🔄 Evolving | Was HTTP, now Unix socket |
| BearDog security | 🔄 Ready | Client ready, needs integration |
| Inter-primal comm | 🔄 Mixed | Socket client exists, migration needed |

### Target State

| Component | Status | Notes |
|-----------|--------|-------|
| JSON-RPC socket | ✅ Working | Primary IPC method |
| Songbird discovery | 🎯 Unix Socket | `/run/user/<uid>/songbird-<family>.sock` |
| BearDog security | 🎯 Integrated | Genetic key validation |
| Inter-primal comm | 🎯 Port-free | 100% Unix sockets |

## 💡 Why This Matters

1. **Security**: Unix sockets + BearDog > HTTP
2. **Performance**: Unix sockets are faster (no TCP/IP overhead)
3. **Architecture**: TRUE PRIMAL requires capability-based discovery
4. **Consistency**: All primals use same transport patterns

## 🔄 Migration Example

### Before (HTTP - DEPRECATED)

```rust
// crates/main/src/biomeos_integration/ecosystem_client.rs
let client = EcosystemClient::new(); // Uses reqwest HTTP
let songbird_url = "http://localhost:8080"; // ❌ Hardcoded HTTP
client.register_service_with_songbird(registration).await?;
```

### After (Unix Socket - TRUE PRIMAL)

```rust
// crates/main/src/biomeos_integration/unix_socket_client.rs
let mut client = UnixSocketClient::connect_to_songbird().await?; // ✅ Auto-discovers socket
client.register_with_songbird(registration).await?; // ✅ JSON-RPC over Unix socket
```

## 🚀 Environment Variables

### New Variables

**Capability-Based Discovery** (TRUE PRIMAL compliant):
- `ORCHESTRATION_SOCKET` - Socket for orchestration capability
- `SECURITY_SOCKET` - Socket for security capability
- `STORAGE_SOCKET` - Socket for storage capability
- `COMPUTE_SOCKET` - Socket for compute capability
- `AI_SOCKET` - Socket for AI capability
- `CORE_SOCKET` - Socket for core/biomeOS capability

**Legacy** (deprecated - use capability-based instead):
- ~~`SONGBIRD_SOCKET`~~ → Use `ORCHESTRATION_SOCKET`
- ~~`BEARDOG_SOCKET`~~ → Use `SECURITY_SOCKET`
- ~~`BIOMEOS_SOCKET`~~ → Use `CORE_SOCKET`

**Family/Atomic Configuration**:
- `FAMILY_ID` or `SQUIRREL_FAMILY_ID` - Family identifier (default: "default")
- `SQUIRREL_NODE_ID` - Node identifier (default: hostname)

### Discovery Examples

**Capability-Based** (Recommended):
```bash
# ✅ Request by capability
export ORCHESTRATION_SOCKET=/run/user/1000/songbird-nat0.sock
export SECURITY_SOCKET=/run/user/1000/beardog-nat0.sock

# Or use socket registry (managed by orchestration layer)
cat > /run/user/1000/socket-registry.json << EOF
{
  "orchestration": "/run/user/1000/songbird-nat0.sock",
  "security": "/run/user/1000/beardog-nat0.sock",
  "compute": "/run/user/1000/toadstool-nat0.sock"
}
EOF

# Let code discover by capability
./squirrel  # Reads registry automatically
```

**Legacy** (Deprecated):
```bash
# ❌ OLD: Hardcodes primal name
export SONGBIRD_SOCKET=/run/user/1000/songbird-nat0.sock

# ✅ NEW: Capability-based
export ORCHESTRATION_SOCKET=/run/user/1000/songbird-nat0.sock
```

## 📖 Related Files

### Infrastructure (Working)
- `crates/main/src/rpc/server.rs` - Unix socket JSON-RPC server
- `crates/main/src/rpc/unix_socket.rs` - Socket path utilities

### Old (HTTP-based - DEPRECATED)
- `crates/main/src/biomeos_integration/ecosystem_client.rs` - HTTP client
- Uses `reqwest::Client` for HTTP calls
- Hardcoded `http://localhost:8080` fallback

### New (Unix socket - TRUE PRIMAL)
- `crates/main/src/biomeos_integration/unix_socket_client.rs` - Unix socket client
- JSON-RPC 2.0 over Unix sockets
- Auto-discovery via env vars + XDG

## 🎯 Next Steps

### Immediate

1. Test `UnixSocketClient` with running Songbird instance
2. Update service registration flow to use Unix socket
3. Verify JSON-RPC compatibility with biomeOS

### Short-term

1. Integrate BearDog security validation
2. Update all inter-primal calls
3. Deprecate HTTP-based ecosystem_client
4. Update documentation

### Long-term

1. Remove HTTP client completely for inter-primal comm
2. Keep HTTP API for external access only
3. Add chaos testing for socket resilience
4. Performance benchmarking (Unix vs HTTP)

## ✅ Success Criteria

- [ ] All inter-primal communication uses Unix sockets
- [ ] Zero hardcoded HTTP URLs for primals
- [ ] BearDog security integrated
- [ ] biomeOS NUCLEUS integration working
- [ ] Documentation updated
- [ ] HTTP API optional (external access only)

---

**Status**: Infrastructure complete, migration in progress  
**Blocker**: None (workaround: biomeOS can connect to Squirrel directly)  
**Next**: Update service registration to use Unix socket client

## 📚 References

- biomeOS deep debt issue: `biomeos/docs/SQUIRREL_DEEP_DEBT.md`
- TRUE PRIMAL principles: Port-free, capability-based, secure-by-default
- Socket standards: `docs/sessions/2026-01-11/BIOMEOS_SOCKET_STANDARDS.md`

