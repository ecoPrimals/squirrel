// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors
//!
//! Pure Rust system info. Uses `/proc` on Linux; elsewhere uses [`rustix`] (no `nix`).
//! ecoBin v3.0: no infrastructure C (e.g. `/proc` + rustix instead of sysinfo).

use std::io;

/// Memory info from /proc/meminfo (Linux) or defaults.
#[derive(Debug, Clone, Default)]
pub struct MemoryInfo {
    /// Total memory in bytes
    pub total: u64,
    /// Used memory (total - available) in bytes
    pub used: u64,
    /// Available memory in bytes
    pub available: u64,
}

/// Read system memory from /proc/meminfo (Linux).
///
/// # Errors
///
/// Returns `io::Error` if `/proc/meminfo` cannot be read.
#[cfg(target_os = "linux")]
pub fn memory_info() -> Result<MemoryInfo, io::Error> {
    let meminfo = std::fs::read_to_string("/proc/meminfo")?;
    let mut total = 0u64;
    let mut available = 0u64;
    for line in meminfo.lines() {
        if let Some(val) = line.strip_prefix("MemTotal:") {
            total = val
                .split_whitespace()
                .next()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0);
        } else if let Some(val) = line.strip_prefix("MemAvailable:") {
            available = val
                .split_whitespace()
                .next()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0);
        }
    }
    let used = total.saturating_sub(available);
    Ok(MemoryInfo {
        total,
        used,
        available,
    })
}

/// Non-Linux: return defaults (zeros).
///
/// # Errors
///
/// This variant always succeeds; the `Result` signature matches the Linux variant.
#[cfg(not(target_os = "linux"))]
pub fn memory_info() -> Result<MemoryInfo, io::Error> {
    Ok(MemoryInfo::default())
}

/// Process RSS in MB from /proc/self/statm (Linux).
///
/// # Errors
///
/// Returns `io::Error` if `/proc/self/statm` cannot be read.
#[cfg(target_os = "linux")]
pub fn process_rss_mb() -> Result<f64, io::Error> {
    let statm = std::fs::read_to_string("/proc/self/statm")?;
    let parts: Vec<&str> = statm.split_whitespace().collect();
    let rss_pages: u64 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let page_size = 4096u64;
    #[expect(clippy::cast_precision_loss, reason = "CPU percentage calculation")]
    Ok((rss_pages * page_size) as f64 / (1024.0 * 1024.0))
}

/// Non-Linux stub: returns `Ok(0.0)`.
///
/// # Errors
///
/// This variant always succeeds; the `Result` signature matches the Linux variant.
#[cfg(not(target_os = "linux"))]
pub fn process_rss_mb() -> Result<f64, io::Error> {
    Ok(0.0)
}

/// CPU count via `std::thread::available_parallelism` or /proc/cpuinfo (Linux).
///
/// # Errors
///
/// Returns `io::Error` if neither `/proc/cpuinfo` nor `available_parallelism`
/// succeeds.  Falls back to 1 when all probes fail.
pub fn cpu_count() -> Result<u32, io::Error> {
    #[cfg(target_os = "linux")]
    {
        if let Ok(s) = std::fs::read_to_string("/proc/cpuinfo") {
            let count = u32::try_from(s.lines().filter(|l| l.starts_with("processor")).count())
                .unwrap_or(u32::MAX);
            if count > 0 {
                return Ok(count);
            }
        }
    }
    std::thread::available_parallelism()
        .map(|p| u32::try_from(p.get()).unwrap_or(u32::MAX))
        .or(Ok(1))
}

/// System uptime in seconds from /proc/uptime (Linux).
///
/// # Errors
///
/// Returns `io::Error` if `/proc/uptime` cannot be read.
#[cfg(target_os = "linux")]
pub fn uptime_seconds() -> Result<u64, io::Error> {
    let uptime = std::fs::read_to_string("/proc/uptime")?;
    let secs: f64 = uptime
        .split_whitespace()
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "Platform-specific size handling"
    )]
    Ok(secs.max(0.0).round() as u64)
}

/// Non-Linux stub: returns `Ok(0)`.
///
/// # Errors
///
/// This variant always succeeds; the `Result` signature matches the Linux variant.
#[cfg(not(target_os = "linux"))]
pub fn uptime_seconds() -> Result<u64, io::Error> {
    Ok(0)
}

/// Hostname: `HOSTNAME` env var → `/proc/sys/kernel/hostname` (Linux) → `uname().nodename`.
///
/// Pure-Rust on Linux (`/proc`). On other Unix platforms, uses [`rustix::system::uname`].
///
/// # Errors
///
/// Returns `io::Error` if no method produces a non-empty hostname.
pub fn hostname() -> Result<String, io::Error> {
    if let Ok(h) = std::env::var("HOSTNAME")
        && !h.is_empty()
    {
        return Ok(h);
    }
    #[cfg(target_os = "linux")]
    {
        if let Ok(h) = std::fs::read_to_string("/proc/sys/kernel/hostname") {
            let h = h.trim().to_string();
            if !h.is_empty() {
                return Ok(h);
            }
        }
    }
    #[cfg(all(unix, not(target_os = "linux")))]
    {
        let s = rustix::system::uname()
            .nodename()
            .to_string_lossy()
            .into_owned();
        if !s.is_empty() {
            return Ok(s);
        }
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "hostname not available",
    ))
}

/// Current user ID (UID). Pure-Rust on Linux via `/proc/self/status`.
///
/// Falls back to [`rustix::process::getuid`] on other Unix platforms or if `/proc` is unavailable.
#[cfg(unix)]
#[must_use]
pub fn current_uid() -> u32 {
    #[cfg(target_os = "linux")]
    {
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if let Some(rest) = line.strip_prefix("Uid:")
                    && let Some(uid_str) = rest.split_whitespace().next()
                    && let Ok(uid) = uid_str.parse::<u32>()
                {
                    return uid;
                }
            }
        }
        rustix::process::getuid().as_raw()
    }
    #[cfg(not(target_os = "linux"))]
    {
        rustix::process::getuid().as_raw()
    }
}

/// System CPU usage % from /proc/stat (Linux). Average since boot.
///
/// # Errors
///
/// Returns `io::Error` if `/proc/stat` cannot be read.
#[cfg(target_os = "linux")]
pub fn system_cpu_usage_percent() -> Result<f64, io::Error> {
    let stat = std::fs::read_to_string("/proc/stat")?;
    let Some(line) = stat.lines().next() else {
        return Ok(0.0);
    };
    let vals: Vec<u64> = line
        .split_whitespace()
        .skip(1)
        .filter_map(|v| v.parse().ok())
        .collect();
    if vals.len() < 4 {
        return Ok(0.0);
    }
    let idle = vals[3];
    let total: u64 = vals.iter().sum();
    #[expect(clippy::cast_precision_loss, reason = "Percentage calculation")]
    Ok(if total > 0 {
        ((total - idle) as f64 / total as f64) * 100.0
    } else {
        0.0
    })
}

/// Non-Linux stub: returns `Ok(0.0)`.
///
/// # Errors
///
/// This variant always succeeds; the `Result` signature matches the Linux variant.
#[cfg(not(target_os = "linux"))]
pub fn system_cpu_usage_percent() -> Result<f64, io::Error> {
    Ok(0.0)
}

/// Network I/O totals from `/proc/net/dev`.
#[derive(Debug, Clone, Default)]
pub struct NetworkBytes {
    /// Total bytes received across all interfaces
    pub rx_bytes: u64,
    /// Total bytes transmitted across all interfaces
    pub tx_bytes: u64,
}

/// Read cumulative network byte counters from `/proc/net/dev` (Linux).
///
/// # Errors
///
/// Returns `io::Error` if `/proc/net/dev` cannot be read.
#[cfg(target_os = "linux")]
pub fn network_bytes() -> Result<NetworkBytes, io::Error> {
    let content = std::fs::read_to_string("/proc/net/dev")?;
    let mut rx_total: u64 = 0;
    let mut tx_total: u64 = 0;

    for line in content.lines().skip(2) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 10 {
            let iface = parts[0].trim_end_matches(':');
            if iface == "lo" {
                continue;
            }
            if let Ok(rx) = parts[1].parse::<u64>() {
                rx_total = rx_total.saturating_add(rx);
            }
            if let Ok(tx) = parts[9].parse::<u64>() {
                tx_total = tx_total.saturating_add(tx);
            }
        }
    }

    Ok(NetworkBytes {
        rx_bytes: rx_total,
        tx_bytes: tx_total,
    })
}

/// Stub for non-Linux platforms.
#[cfg(not(target_os = "linux"))]
pub fn network_bytes() -> Result<NetworkBytes, io::Error> {
    Ok(NetworkBytes::default())
}

/// Disk usage percentage for the filesystem containing the given path.
///
/// Uses `rustix::fs::statvfs` (pure Rust, no libc FFI).
///
/// # Errors
///
/// Returns `io::Error` if the statvfs call fails.
#[cfg(unix)]
pub fn disk_usage_percent(path: &str) -> Result<f64, io::Error> {
    let stat = rustix::fs::statvfs(path).map_err(io::Error::other)?;

    #[expect(
        clippy::cast_precision_loss,
        reason = "filesystem block counts fit f64 for percentage"
    )]
    let total = stat.f_blocks as f64 * stat.f_frsize as f64;
    #[expect(
        clippy::cast_precision_loss,
        reason = "filesystem block counts fit f64 for percentage"
    )]
    let free = stat.f_bfree as f64 * stat.f_frsize as f64;
    if total == 0.0 {
        return Ok(0.0);
    }
    Ok(((total - free) / total) * 100.0)
}

/// Stub for non-Unix platforms.
#[cfg(not(unix))]
pub fn disk_usage_percent(_path: &str) -> Result<f64, io::Error> {
    Ok(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "linux")]
    #[test]
    fn test_memory_info_linux() {
        let info = memory_info().expect("memory_info should succeed on Linux");
        assert!(info.total > 0, "total memory should be > 0");
        assert!(
            info.available <= info.total,
            "available should not exceed total"
        );
        assert!(info.used == info.total.saturating_sub(info.available));
    }

    #[cfg(not(target_os = "linux"))]
    #[test]
    fn test_memory_info_non_linux() {
        let info = memory_info().expect("memory_info should return Ok with defaults on non-Linux");
        assert_eq!(info.total, 0);
        assert_eq!(info.used, 0);
        assert_eq!(info.available, 0);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_process_rss_mb_linux() {
        let rss = process_rss_mb().expect("process_rss_mb should succeed on Linux");
        assert!(rss >= 0.0, "RSS should be non-negative");
    }

    #[cfg(not(target_os = "linux"))]
    #[test]
    fn test_process_rss_mb_non_linux() {
        let rss = process_rss_mb().expect("process_rss_mb should return Ok(0.0) on non-Linux");
        assert_eq!(rss, 0.0);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_cpu_count_linux() {
        let count = cpu_count().expect("cpu_count should succeed on Linux");
        assert!(count > 0, "cpu_count should be > 0");
    }

    #[cfg(not(target_os = "linux"))]
    #[test]
    fn test_cpu_count_non_linux() {
        let count = cpu_count().expect("cpu_count should succeed on non-Linux");
        assert!(count >= 1, "cpu_count should be at least 1 (fallback)");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_uptime_seconds_linux() {
        let uptime = uptime_seconds().expect("uptime_seconds should succeed on Linux");
        assert!(uptime > 0, "uptime should be > 0 on a running system");
    }

    #[cfg(not(target_os = "linux"))]
    #[test]
    fn test_uptime_seconds_non_linux() {
        let uptime = uptime_seconds().expect("uptime_seconds should return Ok(0) on non-Linux");
        assert_eq!(uptime, 0);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_hostname_linux() {
        let name = hostname().expect("hostname should succeed on Linux");
        assert!(!name.is_empty(), "hostname should be non-empty");
    }

    #[cfg(not(target_os = "linux"))]
    #[test]
    fn test_hostname_non_linux() {
        match hostname() {
            Ok(s) => assert!(!s.is_empty(), "if Ok, hostname should be non-empty"),
            Err(_) => { /* graceful error is acceptable */ }
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_system_cpu_usage_percent_linux() {
        let usage =
            system_cpu_usage_percent().expect("system_cpu_usage_percent should succeed on Linux");
        assert!(
            (0.0..=100.0).contains(&usage),
            "cpu usage should be between 0 and 100, got {usage}"
        );
    }

    #[cfg(not(target_os = "linux"))]
    #[test]
    fn test_system_cpu_usage_percent_non_linux() {
        let usage = system_cpu_usage_percent()
            .expect("system_cpu_usage_percent should return Ok(0.0) on non-Linux");
        assert_eq!(usage, 0.0);
    }

    #[cfg(unix)]
    #[test]
    fn test_current_uid() {
        let uid = current_uid();
        assert!(uid < 65534, "UID should be within normal range");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_network_bytes_linux() {
        let net = network_bytes().expect("network_bytes should succeed on Linux");
        // A running system with any network activity should have non-zero counters
        // (at least loopback traffic from localhost test connections)
        assert!(
            net.rx_bytes > 0 || net.tx_bytes > 0,
            "at least one direction should have traffic"
        );
    }

    #[cfg(not(target_os = "linux"))]
    #[test]
    fn test_network_bytes_non_linux() {
        let net = network_bytes().expect("network_bytes should return Ok with defaults");
        assert_eq!(net.rx_bytes, 0);
        assert_eq!(net.tx_bytes, 0);
    }

    #[cfg(unix)]
    #[test]
    fn test_disk_usage_percent_root() {
        let usage = disk_usage_percent("/").expect("disk_usage_percent should succeed for /");
        assert!(
            (0.0..=100.0).contains(&usage),
            "disk usage should be 0-100%, got {usage}"
        );
        assert!(usage > 0.0, "root filesystem should have some usage");
    }

    #[cfg(not(unix))]
    #[test]
    fn test_disk_usage_percent_non_unix() {
        let usage = disk_usage_percent("/").expect("should return 0.0 on non-unix");
        assert_eq!(usage, 0.0);
    }
}
