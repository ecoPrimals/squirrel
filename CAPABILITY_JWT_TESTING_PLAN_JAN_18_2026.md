# Capability-Based JWT Testing Plan

**Date**: January 18, 2026  
**Phase**: 4 - Testing  
**Goal**: Validate TRUE PRIMAL capability-based crypto & JWT

---

## 🎯 Testing Strategy

### 1. Unit Tests ✅ (Already Done)
- JWT claims creation
- Token header encoding
- Configuration validation
- Token extraction from headers

### 2. Integration Tests ⏳ (In Progress)
- Crypto capability client with mock provider
- JWT creation/verification end-to-end
- Capability discovery simulation
- Error handling (provider unavailable, timeout, etc.)

### 3. Performance Benchmarks ⏳ (Next)
- Token creation speed
- Token verification speed
- Compare: Capability vs Local JWT
- Unix socket overhead measurement

### 4. Compatibility Tests ⏳
- Old BearDog modules still work (deprecated but functional)
- Migration path validation
- Backward compatibility

---

## 📋 Test Scenarios

### Scenario 1: Happy Path ✅
- Discover crypto capability
- Create JWT token
- Verify JWT token
- Extract claims

### Scenario 2: Provider Unavailable ⏳
- Socket doesn't exist
- Connection refused
- Provider crashes mid-request
- Graceful error handling

### Scenario 3: Invalid Tokens ⏳
- Malformed token (wrong format)
- Invalid signature
- Expired token
- Not-yet-valid token (nbf)

### Scenario 4: Capability Discovery ⏳
- Environment variable set (CRYPTO_CAPABILITY_SOCKET)
- Default socket path fallback
- Multiple providers (future)

### Scenario 5: Performance ⏳
- 1000 tokens/sec creation
- 1000 tokens/sec verification
- Acceptable latency (<200µs overhead)

---

## 🧪 Test Implementation

### Mock Crypto Provider
Create a simple mock server that:
1. Listens on Unix socket
2. Responds to JSON-RPC requests
3. Simulates Ed25519 operations
4. Can simulate failures/timeouts

### Integration Test Suite
```rust
#[tokio::test]
async fn test_capability_jwt_full_flow() {
    // 1. Start mock crypto provider
    // 2. Create CapabilityJwtService
    // 3. Create token
    // 4. Verify token
    // 5. Validate claims
}
```

---

## 📊 Success Criteria

### Functional ✅
- ✅ All unit tests pass
- ⏳ Integration tests pass (with mock provider)
- ⏳ Error handling validated
- ⏳ Capability discovery works

### Performance ✅
- ⏳ Token creation: <200µs (with Unix socket)
- ⏳ Token verification: <250µs (with Unix socket)
- ⏳ Comparable to old BearDog implementation
- ⏳ Acceptable overhead vs local JWT

### Quality ✅
- ✅ Zero compiler errors
- ✅ Zero critical warnings
- ⏳ Code coverage >80%
- ⏳ All edge cases handled

---

## 🚀 Execution Plan

### Step 1: Mock Crypto Provider (30 min)
Create simple mock server for testing

### Step 2: Integration Tests (1 hour)
Write comprehensive integration test suite

### Step 3: Performance Benchmarks (30 min)
Measure and document performance

### Step 4: Documentation (30 min)
Document test results and findings

**Total**: 2-3 hours

---

*Status: In Progress*  
*Next: Create mock crypto provider for testing*

