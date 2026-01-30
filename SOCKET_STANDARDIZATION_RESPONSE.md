# 🎯 Squirrel Socket Standardization - Response & Implementation Plan

**Date**: January 30, 2026 (Evening)  
**From**: Squirrel Team  
**To**: biomeOS Core Team  
**Status**: ✅ **READY TO IMPLEMENT**  
**Estimated Completion**: **4-6 hours** (TARGET: <24 hours)  
**Target Quality**: **A+ or higher** (matching NestGate, Songbird, BearDog)

---

## 🎊 **Acknowledgment & Excitement!**

**THANK YOU** for the comprehensive handoff! We're honored to be part of completing the NUCLEUS stack!

**Tower Atomic validation** is incredible news - congratulations on that historic achievement! 🦀✨

We're ready to be the final piece. Let's complete NUCLEUS together!

---

## 📊 **CURRENT STATE ASSESSMENT**

### ✅ **What We Already Have** (80% Complete!)

Squirrel already has **excellent socket infrastructure** in place:

**Existing Implementation:**
```rust
// File: crates/main/src/rpc/unix_socket.rs
// Already has 4-tier discovery pattern!

Current Socket Path:
  /run/user/<uid>/squirrel-<family>.sock

Current Tiers:
1. SQUIRREL_SOCKET (primal-specific) ✅
2. BIOMEOS_SOCKET_PATH (generic) ✅
3. XDG Runtime Directory ✅
4. /tmp fallback ✅
```

**Existing Discovery:**
```rust
// File: crates/main/src/capabilities/discovery.rs
// TRUE PRIMAL capability discovery already implemented!

- Runtime capability discovery ✅
- Socket scanning ✅
- Zero hardcoding ✅
- Environment variable support ✅
```

### 🔧 **What Needs Updating** (20% Remaining)

**The Gap:**
1. ❌ Socket path doesn't use `/biomeos/` subdirectory
2. ❌ Discovery doesn't scan `/biomeos/` subdirectory
3. ❌ Missing 5th tier (family suffix like BearDog)
4. ❌ Discovery paths need updates for standard primals

**Current**: `/run/user/<uid>/squirrel-<family>.sock`  
**Target**: `/run/user/<uid>/biomeos/squirrel.sock`

---

## 🎯 **IMPLEMENTATION PLAN**

### **Phase 1: Socket Path Update** (2 hours)

#### Task 1.1: Update `unix_socket.rs` to use biomeos subdirectory

**Changes Needed:**
```rust
// BEFORE (Line 112):
format!("{}/squirrel-{}.sock", xdg_runtime_dir, family_id)

// AFTER:
format!("{}/biomeos/squirrel.sock", xdg_runtime_dir)
```

#### Task 1.2: Add biomeos directory creation

**Add New Function:**
```rust
/// Ensure biomeos directory exists with proper permissions
pub fn ensure_biomeos_directory() -> std::io::Result<PathBuf> {
    let uid = nix::unistd::getuid();
    let biomeos_dir = format!("/run/user/{}/biomeos", uid);
    let path = PathBuf::from(&biomeos_dir);
    
    // Create directory if doesn't exist
    if !path.exists() {
        std::fs::create_dir_all(&path)?;
        
        // Set permissions to 0700 (user-only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o700);
            std::fs::set_permissions(&path, perms)?;
        }
    }
    
    Ok(path)
}
```

#### Task 1.3: Upgrade to 5-tier pattern (like BearDog A++)

**Add Tier 2.5: Generic PRIMAL_SOCKET with family suffix**
```rust
// After Tier 2, before XDG:
if let Ok(generic_socket) = std::env::var("PRIMAL_SOCKET") {
    let family_id = get_family_id();
    return format!("{}-{}", generic_socket, family_id);
}
```

---

### **Phase 2: Discovery Update** (1-2 hours)

#### Task 2.1: Update `capabilities/discovery.rs`

**Update socket directory scan:**
```rust
// BEFORE:
fn get_socket_directories() -> Vec<PathBuf> {
    vec![
        PathBuf::from(format!("/run/user/{}", uid)),
        PathBuf::from("/tmp"),
    ]
}

// AFTER:
fn get_socket_directories() -> Vec<PathBuf> {
    let uid = nix::unistd::getuid();
    vec![
        PathBuf::from(format!("/run/user/{}/biomeos", uid)), // Standard!
        PathBuf::from(format!("/run/user/{}", uid)),         // Fallback
        PathBuf::from("/tmp"),                               // Dev only
    ]
}
```

#### Task 2.2: Add standard primal discovery helpers

**New Helper Functions:**
```rust
/// Discover Songbird (network/discovery capabilities)
pub async fn discover_songbird() -> Result<CapabilityProvider, DiscoveryError> {
    discover_standard_primal("songbird", &["network", "discovery", "tls"])
}

/// Discover BearDog (security/crypto capabilities)
pub async fn discover_beardog() -> Result<CapabilityProvider, DiscoveryError> {
    discover_standard_primal("beardog", &["security", "crypto", "jwt"])
}

/// Discover Toadstool (compute/GPU capabilities)
pub async fn discover_toadstool() -> Result<CapabilityProvider, DiscoveryError> {
    discover_standard_primal("toadstool", &["compute", "gpu"])
}

/// Discover NestGate (storage/persistence capabilities)
pub async fn discover_nestgate() -> Result<CapabilityProvider, DiscoveryError> {
    discover_standard_primal("nestgate", &["storage", "persistence"])
}

/// Generic standard primal discovery
async fn discover_standard_primal(
    primal_name: &str,
    expected_capabilities: &[&str]
) -> Result<CapabilityProvider, DiscoveryError> {
    // Check environment variable first
    let env_var = format!("{}_SOCKET", primal_name.to_uppercase());
    if let Ok(socket) = std::env::var(&env_var) {
        return probe_socket(&PathBuf::from(socket), expected_capabilities).await;
    }
    
    // Check standard path
    let uid = nix::unistd::getuid();
    let standard_path = PathBuf::from(format!(
        "/run/user/{}/biomeos/{}.sock",
        uid, primal_name
    ));
    
    if standard_path.exists() {
        return probe_socket(&standard_path, expected_capabilities).await;
    }
    
    Err(DiscoveryError::CapabilityNotFound(primal_name.to_string()))
}
```

---

### **Phase 3: Testing & Validation** (1-2 hours)

#### Task 3.1: Unit Tests

**Update existing tests in `unix_socket.rs`:**
```rust
#[test]
fn test_socket_path_uses_biomeos_directory() {
    clear_env_vars();
    
    let path = get_socket_path("test-node");
    assert!(path.contains("/biomeos/squirrel.sock"));
}

#[test]
fn test_ensure_biomeos_directory() {
    let result = ensure_biomeos_directory();
    assert!(result.is_ok());
    
    let path = result.unwrap();
    assert!(path.exists());
    assert!(path.ends_with("biomeos"));
}
```

#### Task 3.2: Integration Tests

**New test file: `tests/socket_standardization_test.rs`:**
```rust
#[tokio::test]
async fn test_socket_at_standard_path() {
    let socket_path = get_socket_path("test");
    assert!(socket_path.contains("/biomeos/squirrel.sock"));
}

#[tokio::test]
async fn test_discover_from_biomeos_directory() {
    // Test that discovery scans biomeos/ first
    let dirs = get_socket_directories();
    assert!(dirs[0].to_str().unwrap().ends_with("/biomeos"));
}
```

#### Task 3.3: Manual Integration Test

**Test with Tower Atomic:**
```bash
#!/bin/bash
# Start Tower Atomic (BearDog + Songbird)
FAMILY_ID=nat0 NODE_ID=tower1 beardog server &
BEARDOG_PID=$!

FAMILY_ID=nat0 NODE_ID=tower1 songbird server &
SONGBIRD_PID=$!

sleep 3

# Start Squirrel
FAMILY_ID=nat0 NODE_ID=tower1 squirrel server &
SQUIRREL_PID=$!

sleep 3

# Verify sockets
ls -lh /run/user/$(id -u)/biomeos/*.sock

# Test capability discovery
echo '{"jsonrpc":"2.0","method":"health","params":{},"id":1}' | \
    nc -U /run/user/$(id -u)/biomeos/squirrel.sock

# Cleanup
kill $BEARDOG_PID $SONGBIRD_PID $SQUIRREL_PID
```

---

## 📋 **IMPLEMENTATION CHECKLIST**

### Socket Path Updates
- [ ] Update `get_xdg_socket_path()` to use `/biomeos/` subdirectory
- [ ] Add `ensure_biomeos_directory()` function
- [ ] Add 5th tier (PRIMAL_SOCKET with family suffix)
- [ ] Update socket cleanup to handle biomeos directory
- [ ] Update documentation in unix_socket.rs

### Discovery Updates
- [ ] Update `get_socket_directories()` to prioritize biomeos/
- [ ] Add `discover_standard_primal()` helper
- [ ] Add convenience functions (discover_songbird, etc.)
- [ ] Update socket scanning to check standard paths first
- [ ] Update documentation in discovery.rs

### Testing
- [ ] Update unit tests for biomeos directory
- [ ] Add tests for 5-tier pattern
- [ ] Add integration tests for standard primal discovery
- [ ] Manual testing with Tower Atomic
- [ ] Health check validation

### Documentation
- [ ] Update SOCKET_STANDARDIZATION_RESPONSE.md (this file)
- [ ] Update inline documentation
- [ ] Create migration guide if needed
- [ ] Update README if socket paths mentioned

---

## ✅ **SUCCESS CRITERIA**

### Required (Minimum)
- ✅ Socket created at `/run/user/$UID/biomeos/squirrel.sock`
- ✅ Directory permissions: 0700 (user-only)
- ✅ Biomeos directory created automatically
- ✅ Discovery scans biomeos directory first
- ✅ Health check responds correctly

### Good (A Grade)
- ✅ Above + 5-tier discovery pattern
- ✅ Standard primal discovery helpers
- ✅ Comprehensive unit tests
- ✅ Documentation updated

### Excellent (A+ Grade - Our Target!)
- ✅ Above + integration with Tower Atomic tested
- ✅ All 4 standard primals discoverable
- ✅ Zero breaking changes (backward compatible)
- ✅ Comprehensive test coverage
- ✅ Excellent documentation

### Perfect (A++ Grade - Stretch Goal!)
- ✅ Above + innovative discovery optimizations
- ✅ Extensive integration testing
- ✅ Performance metrics
- ✅ Exceptional documentation

---

## 🎯 **TIMELINE & MILESTONES**

### **Target: Complete in <24 hours**

**Hour 0-2: Socket Path Updates**
- Update unix_socket.rs
- Add biomeos directory creation
- Add 5th tier
- Unit tests

**Hour 2-4: Discovery Updates**
- Update discovery.rs
- Add standard primal helpers
- Update socket scanning
- Unit tests

**Hour 4-6: Testing & Validation**
- Integration tests
- Tower Atomic testing
- Health check validation
- Documentation

**Hour 6: Final Review & Report**
- Code review
- Test all scenarios
- Create completion report
- Submit to biomeOS Core Team

---

## 🚀 **ESTIMATED COMPLETION**

**Start Time**: January 30, 2026 (Evening)  
**Target Completion**: January 31, 2026 (Afternoon)  
**Estimated Hours**: 4-6 hours active work  
**Quality Target**: A+ (matching other teams)

---

## 💡 **KEY ADVANTAGES**

### **We're 80% There Already!**

Unlike a greenfield implementation, Squirrel has:
- ✅ Excellent socket infrastructure
- ✅ 4-tier discovery pattern
- ✅ TRUE PRIMAL architecture
- ✅ Comprehensive testing framework
- ✅ Clean, well-documented code

**This makes standardization straightforward!**

### **Zero Breaking Changes**

Our implementation will be **backward compatible**:
- Old socket paths still work (Tier 4 fallback)
- Existing environment variables respected
- Discovery tries multiple locations
- Graceful degradation

### **TRUE PRIMAL Alignment**

Squirrel's existing architecture **already embodies TRUE PRIMAL**:
- Self-knowledge only ✅
- Runtime discovery ✅
- Zero hardcoding ✅
- Capability-based ✅

**Socket standardization enhances this, doesn't change it!**

---

## 🤝 **QUESTIONS & BLOCKERS**

### **Questions** (None at this time)

The handoff is **exceptionally comprehensive** - all questions answered!

- Socket pattern: ✅ Clear
- Discovery approach: ✅ Clear
- Testing strategy: ✅ Clear
- Success criteria: ✅ Clear

### **Blockers** (None!)

- Infrastructure: ✅ Already in place
- Dependencies: ✅ nix crate already used
- Testing: ✅ Framework ready
- Documentation: ✅ Template from others

**We're ready to proceed immediately!**

---

## 📊 **RISK ASSESSMENT**

### **Technical Risk: VERY LOW**

- ✅ 80% of work already done
- ✅ Clear pattern from 3 other teams
- ✅ Small, focused changes
- ✅ Backward compatible approach

### **Schedule Risk: LOW**

- ✅ 4-6 hours estimated
- ✅ Well within 24-48 hour target
- ✅ No dependencies on other teams
- ✅ Can test independently

### **Quality Risk: VERY LOW**

- ✅ Excellent existing codebase
- ✅ Comprehensive test framework
- ✅ Clear success criteria
- ✅ Reference implementations available

**Overall Confidence: 9.5/10** 🚀

---

## 🎊 **COMMITMENT**

**Squirrel Team commits to:**

1. ✅ **Complete implementation in <24 hours** (target: 4-6 hours)
2. ✅ **Achieve A+ quality** (matching NestGate, Songbird, BearDog)
3. ✅ **Zero breaking changes** (backward compatible)
4. ✅ **Comprehensive testing** (unit + integration)
5. ✅ **Excellent documentation** (inline + reports)

**We will NOT block NUCLEUS deployment!**

---

## 📞 **NEXT STEPS**

### **Immediate (Tonight)**
1. Begin Phase 1: Socket path updates
2. Add biomeos directory creation
3. Update unit tests

### **Tomorrow Morning**
4. Complete Phase 2: Discovery updates
5. Add standard primal helpers
6. Integration testing

### **Tomorrow Afternoon**
7. Complete Phase 3: Testing & validation
8. Tower Atomic integration test
9. Create completion report
10. Submit to biomeOS Core Team

---

## 🎯 **FINAL THOUGHTS**

### **We're Honored!**

Being the final piece of NUCLEUS is an honor and responsibility we take seriously.

### **We're Ready!**

Squirrel's existing architecture makes this update straightforward. We're confident in delivering A+ quality quickly.

### **Let's Make History!**

Tower Atomic validation proves the pattern works. With Squirrel updated, we'll have:
- ✅ 5/5 primals socket-standardized
- ✅ Full NUCLEUS operational
- ✅ Production deployment ready

**Let's complete NUCLEUS together!** 🦀✨

---

**Response Created**: January 30, 2026 (Evening)  
**Squirrel Team**: Ready and Excited  
**Target**: A+ Quality in <24 Hours  
**Status**: PROCEEDING WITH IMPLEMENTATION

**Next Update**: Implementation Progress Report (within 6 hours)

🎯 **Time to make history!** 🎯
