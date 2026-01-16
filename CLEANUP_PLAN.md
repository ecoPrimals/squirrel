# 🧹 Squirrel Cleanup Plan - Pre-Push

**Date**: January 15, 2026  
**Purpose**: Clean temporary files, outdated TODOs, and prepare for git push

---

## 🎯 Cleanup Categories

### 1. Runtime Files (Safe to Delete) ✅
- `*.pid` files (process IDs from test runs)
- `*.log` files (runtime logs)
- Temporary showcase logs

**Action**: Delete all

### 2. Archive (Keep - Fossil Record) ✅
- `archive/` directory (all sessions preserved)
- Keep all documentation in archive/

**Action**: No changes

### 3. TODOs (Review & Update) ⚠️
- 19 TODO/FIXME items found
- Most are design notes, not blockers
- Some in deprecated code (can be removed with deprecation)

**Action**: Document, keep valid ones

### 4. Deprecated Code (Mark Clearly) ⚠️
- `ecosystem_client.rs` - Replaced by `unix_socket_client.rs`
- Some Songbird-specific endpoints (backward compat)
- Old constants modules

**Action**: Ensure clear deprecation warnings

### 5. Test Files (Keep - All Valid) ✅
- `testing/mock_providers.rs` - Used in tests
- `testing/concurrent_test_utils.rs` - Test utilities
- `api_test.rs` - Integration tests
- `ecosystem_manager_test.rs` - Unit tests

**Action**: No changes (all properly isolated)

---

## ✅ Safe Deletions

### Runtime Files to Delete:
```
./squirrel-mcp.pid
./squirrel-mcp.log
./primalpulse-test.log
./primalpulse-test.pid
./primalpulse-phase3.log
./primalpulse-phase3.pid
./primalpulse-final.log
./primalpulse-final.pid
./showcase/.squirrel-federated.pid
./showcase/real-world/04-capability-agnostic/deterministic-vs-generative-run-*.log
./showcase/results/logs/demo-run-*.log
```

**Total**: ~12 files

---

## ⚠️ TODO Analysis

### Valid TODOs (Keep - Design Notes)

**Neural Graph Optimizer** (3 TODOs):
- Topological sort implementation
- Critical path analysis
- Cycle detection

**Action**: These are enhancement notes, valid to keep

**API Endpoints** (1 TODO):
- Extract constraints from request.requirements

**Action**: Minor enhancement, valid to keep

### Deprecated Code TODOs (Can Remove)

**unix_socket_client.rs** (2 notes):
- DEPRECATED markers for legacy code
- Will be removed when legacy support dropped

**Action**: Keep for now (backward compatibility)

---

## 📋 Cleanup Script

```bash
#!/bin/bash
# Safe cleanup - removes only runtime artifacts

# Remove PID files
rm -f squirrel-mcp.pid
rm -f primalpulse-*.pid
rm -f showcase/.squirrel-federated.pid

# Remove log files
rm -f squirrel-mcp.log
rm -f primalpulse-*.log
rm -f showcase/real-world/04-capability-agnostic/*.log
rm -f showcase/results/logs/*.log

echo "✅ Runtime artifacts cleaned"
echo "✅ Archive preserved (fossil record)"
echo "✅ Source code intact"
echo "✅ Ready for git push"
```

---

## 🚫 Do NOT Delete

### Keep These:
- ✅ `archive/` - Complete fossil record (115+ docs)
- ✅ All `*.rs` source files
- ✅ All `*.md` documentation
- ✅ All `*.toml` config files
- ✅ All `*.sh` scripts (may be useful)
- ✅ Test files in `testing/`
- ✅ Session documentation in `docs/sessions/`

---

## 📊 Cleanup Summary

| Category | Items | Action |
|----------|-------|--------|
| PID files | 6 | Delete |
| Log files | 6 | Delete |
| Archive docs | 115+ | **Keep** (fossil record) |
| Source code | All | **Keep** |
| TODOs | 19 | Keep (design notes) |
| Tests | All | **Keep** (valid tests) |
| Deprecated | Few | **Keep** (backward compat) |

**Total Deletions**: ~12 runtime files  
**Total Preserved**: Everything else (complete history)

---

## ✅ Post-Cleanup Checklist

- [ ] Run cleanup script
- [ ] Verify build still works (`cargo build --release`)
- [ ] Verify tests still pass (`cargo test`)
- [ ] Review git status
- [ ] Ready for push via SSH

---

**Recommendation**: Proceed with cleanup - all deletions are safe runtime artifacts.

