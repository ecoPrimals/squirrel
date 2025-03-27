# Dashboard Integration TODO List

## Quick Fix Items

### 1. Update sysinfo Imports in All Files

```rust
// Add to all files that interact with system metrics
use sysinfo::{
    System,
    SystemExt,
    ProcessExt,
    NetworksExt,
    DiskExt,
    CpuExt,
    NetworkExt,
    DiskUsageExt
};
```

**Files to update:**
- `crates/monitoring/src/metrics/resource.rs`
- `crates/monitoring/src/plugins/system_metrics.rs`
- `crates/monitoring/src/network/mod.rs`

### 2. Fix Resource Access Methods

| File | Replace | With |
|------|---------|------|
| `resource.rs` | `let disks_info = Disks::new_with_refreshed_list();` | `let disks_info = self.system.disks();` |
| `system_metrics.rs` | `let networks = Networks::new_with_refreshed_list();` | `let networks = self.system.networks();` |
| All files | Direct property access on `system` | Method calls (e.g., `system.memory_used()`) |

### 3. Complete ResourceMetricsCollectorAdapter Implementation

```rust
impl ResourceMetricsCollectorAdapter {
    // Add missing methods:
    
    pub fn collect_cpu_metrics(&mut self) -> f64 {
        self.system.refresh_cpu();
        self.system.global_cpu_info().cpu_usage()
    }
    
    pub fn collect_memory_metrics(&mut self) -> (u64, u64) {
        self.system.refresh_memory();
        (self.system.used_memory(), self.system.total_memory())
    }
    
    pub fn collect_disk_metrics(&mut self) -> (u64, u64) {
        self.system.refresh_disks_list();
        let disks = self.system.disks();
        // Implementation details...
    }
}
```

### 4. Align Data Structures with Dashboard-Core

Ensure these structures match or can convert to dashboard-core equivalents:

1. `NetworkMetricsSnapshot` → `NetworkSnapshot`
2. `SystemMetricsSnapshot` → `SystemSnapshot`
3. `DiskMetricsSnapshot` → Dashboard's disk metrics

## Integration Testing

1. Create a simple test that verifies metrics collection:

```rust
#[test]
fn test_metrics_can_be_converted_to_dashboard_format() {
    let mut collector = ResourceMetricsCollector::new();
    let metrics = collector.collect();
    
    // Verify metrics can be converted to dashboard format
    let dashboard_snapshot = SystemSnapshot::from(metrics);
    
    // Assert fields are properly mapped
    assert!(dashboard_snapshot.cpu_usage >= 0.0);
    assert!(dashboard_snapshot.memory_used > 0);
    // etc.
}
```

2. Test that network interfaces are properly collected:

```rust
#[test]
fn test_network_interfaces_collection() {
    let mut collector = NetworkMetricsCollector::new();
    let metrics = collector.collect();
    
    // Verify network interfaces are properly collected
    let dashboard_network = NetworkSnapshot::from(metrics);
    
    // Assert network interfaces are present
    assert!(!dashboard_network.interfaces.is_empty());
}
```

## Priority Order

1. Fix sysinfo imports (1-2 hours)
2. Fix resource access methods (2-3 hours)
3. Complete adapter implementations (4-6 hours)
4. Align data structures (6-8 hours)
5. Add integration tests (4-6 hours)

## Validation

Once changes are complete, run:

```bash
cargo build -p ui-terminal
cargo test -p monitoring
```

All tests should pass and the terminal UI should build successfully. 