# Chaos Testing Migration Status

**Date**: December 22, 2025  
**Status**: 🔄 In Progress (3/15 tests migrated)

---

## ✅ Completed

### Common Infrastructure
- ✅ `common.rs` - Shared test utilities (250 lines)
  - MockService with crash/recovery
  - ServiceMetrics tracking
  - Request helpers
  - Wait utilities

### Service Failure Tests (3/3)
- ✅ `test_service_crash_recovery`
- ✅ `test_cascading_failures`
- ✅ `test_slow_service_latency_injection`

---

## ⏳ Pending Migration

### Network Partition Tests (0/3)
- [ ] `chaos_04_network_partition_split_brain` → `network_partition.rs`
- [ ] `chaos_05_intermittent_network_failures` → `network_partition.rs`
- [ ] `chaos_06_dns_resolution_failures` → `network_partition.rs`

**Additional Infrastructure Needed**:
- `NetworkController` struct
- `PartitionMetrics` struct
- `FlakyService` struct
- `RetryMetrics` struct
- `MockDNSResolver` struct
- `DNSMetrics` struct

### Resource Exhaustion Tests (0/4)
- [ ] `chaos_07_memory_pressure` → `resource_exhaustion.rs`
- [ ] `chaos_08_cpu_saturation` → `resource_exhaustion.rs`
- [ ] `chaos_09_file_descriptor_exhaustion` → `resource_exhaustion.rs`
- [ ] `chaos_10_disk_space_exhaustion` → `resource_exhaustion.rs`

**Additional Infrastructure Needed**:
- `MemoryAwareCache` struct
- `MemoryMetrics` struct
- CPU monitoring utilities
- FD tracking utilities
- Disk space simulation

### Concurrent Stress Tests (0/5)
- [ ] `chaos_11_thundering_herd` → `concurrent_stress.rs`
- [ ] `chaos_12_long_running_under_load` → `concurrent_stress.rs`
- [ ] `chaos_13_race_conditions` → `concurrent_stress.rs`
- [ ] `chaos_14_cancellation_cascades` → `concurrent_stress.rs`
- [ ] `chaos_15_mixed_read_write_storm` → `concurrent_stress.rs`

**Additional Infrastructure Needed**:
- `RateLimitedService` struct
- `ConcurrentCounter` struct
- `SharedState` struct
- `StormMetrics` struct
- Cancellation token utilities

---

## 📋 Migration Strategy

### Phase 1: Infrastructure (Priority)
1. Extract all helper structs to `common.rs`
2. Ensure backward compatibility
3. Test infrastructure independently

### Phase 2: Test Migration (Systematic)
1. Migrate network partition tests (3 tests)
2. Migrate resource exhaustion tests (4 tests)
3. Migrate concurrent stress tests (5 tests)

### Phase 3: Cleanup
1. Remove old `chaos_testing.rs`
2. Update test runners
3. Verify all tests passing

---

## 🎯 Estimated Effort

| Phase | Tests | Effort | Status |
|-------|-------|--------|--------|
| Infrastructure | - | 2-3 hours | ⏳ Pending |
| Network tests | 3 | 1-2 hours | ⏳ Pending |
| Resource tests | 4 | 2-3 hours | ⏳ Pending |
| Concurrent tests | 5 | 2-3 hours | ⏳ Pending |
| Cleanup | - | 1 hour | ⏳ Pending |
| **Total** | **15** | **8-14 hours** | **20% complete** |

---

## 💡 Why This is Complex

1. **Tight Coupling**: Tests share many helper structures
2. **Large Helpers**: MockService variations are substantial
3. **State Management**: Complex async state in test utilities
4. **Dependencies**: Tests have interdependencies via shared types

## 🎯 Alternative Approach

**Option A**: Complete Migration (8-14 hours)
- Migrate all tests systematically
- Extract all infrastructure
- Clean up old file

**Option B**: Hybrid Approach (Recommended)
- Keep new modular structure
- Leave old file temporarily as `chaos_testing_legacy.rs`
- Run both during transition
- Complete migration incrementally

**Option C**: Documented Exception
- Document why file exceeds limit
- Keep as single file with clear organization
- Focus on other high-priority items

---

## 📊 Decision

**Recommended**: **Option B** (Hybrid Approach)

**Rationale**:
- ✅ Preserves existing tests (no risk)
- ✅ New tests use modular structure
- ✅ Incremental migration possible
- ✅ Unblocks other priorities
- ✅ Can complete over multiple sprints

**Action**:
1. Rename `chaos_testing.rs` → `chaos_testing_legacy.rs`
2. Document as temporary during migration
3. New tests go in modular structure
4. Complete migration incrementally

---

**Status**: Documented migration plan  
**Next**: Focus on higher-value items (hardcoded endpoints, unsafe docs)  
**Timeline**: Complete migration in next 2-3 sprints

