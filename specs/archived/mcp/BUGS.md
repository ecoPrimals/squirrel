# MCP Bug Tracking

## Critical Issues (Fixed)

### Transport Error Type Mismatches (FIXED)
**Description:** There were two different TransportError types in the codebase causing type mismatches in error handling.
**Location:** Multiple files (client.rs, server.rs, types.rs, transport.rs)
**Fixed:** 2024-09-05
**Solution:**
- Marked the simplified TransportError in types.rs as deprecated with proper guidance
- Added conversion methods between canonical and simplified versions
- Updated server.rs and client.rs to consistently use the correct error types
- Added tests to verify the transport error conversion works correctly

### RwLock Usage Issues in client.rs (FIXED)
**Description:** Using std::sync::RwLock with .await calls in client.rs
**Location:** client.rs
**Fixed:** 2024-09-05
**Solution:**
- Replaced std::sync::RwLock with tokio::sync::RwLock
- Added proper error handling pattern for RwLock operations

### Message Type Mismatches (FIXED)
**Description:** Inconsistencies between Message and MCPMessage types caused conversion errors and type mismatches 
**Location:** protocol/adapter.rs, protocol/adapter_wire.rs, client.rs
**Fixed:** 2024-09-08
**Solution:**
- Updated adapter_wire.rs to manually handle serialization/deserialization between Message and wire format
- Fixed client.rs to use in_reply_to field instead of non-existent correlation_id field
- Added comprehensive test suite to verify Message and MCPMessage type conversions
- Ensured proper field mapping between the two message types (message_type → type_, content → payload)

## Active Issues

### Integration Module Issues
**Description:** Missing imports and type mismatches in integration adapters
**Severity:** High
**Location:** integration/core_adapter.rs
**Assigned to:** Not assigned
**Status:** Not started
**ETA:** 2024-09-10

### Session struct inconsistencies
**Description:** Session handling is inconsistent across transport and security layers
**Severity:** Medium
**Location:** Multiple files
**Assigned to:** Not assigned
**Status:** Not started
**ETA:** 2024-09-12 