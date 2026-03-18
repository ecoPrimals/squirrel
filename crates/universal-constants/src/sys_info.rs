// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors
//!
//! Pure Rust system info (no C deps). Uses /proc on Linux, defaults elsewhere.
//! ecoBin v3.0: no infrastructure C (e.g. /proc + nix instead of sysinfo).

#![forbid(unsafe_code)]

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
#[cfg(not(target_os = "linux"))]
pub fn memory_info() -> Result<MemoryInfo, io::Error> {
    Ok(MemoryInfo::default())
}

/// Process RSS in MB from /proc/self/statm (Linux).
#[cfg(target_os = "linux")]
pub fn process_rss_mb() -> Result<f64, io::Error> {
    let statm = std::fs::read_to_string("/proc/self/statm")?;
    let parts: Vec<&str> = statm.split_whitespace().collect();
    let rss_pages: u64 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let page_size = 4096u64;
    #[allow(clippy::cast_precision_loss)]
    Ok((rss_pages * page_size) as f64 / (1024.0 * 1024.0))
}

#[cfg(not(target_os = "linux"))]
pub fn process_rss_mb() -> Result<f64, io::Error> {
    Ok(0.0)
}

/// CPU count via `std::thread::available_parallelism` or /proc/cpuinfo (Linux).
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
#[cfg(target_os = "linux")]
pub fn uptime_seconds() -> Result<u64, io::Error> {
    let uptime = std::fs::read_to_string("/proc/uptime")?;
    let secs: f64 = uptime
        .split_whitespace()
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    Ok(secs.max(0.0).round() as u64)
}

#[cfg(not(target_os = "linux"))]
pub fn uptime_seconds() -> Result<u64, io::Error> {
    Ok(0)
}

/// Hostname: HOSTNAME env var or gethostname via nix.
pub fn hostname() -> Result<String, io::Error> {
    if let Ok(h) = std::env::var("HOSTNAME")
        && !h.is_empty()
    {
        return Ok(h);
    }
    #[cfg(unix)]
    {
        let name = nix::unistd::gethostname().map_err(io::Error::from)?;
        let s = name.to_string_lossy().to_string();
        if !s.is_empty() {
            return Ok(s);
        }
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "hostname not available",
    ))
}

/// System CPU usage % from /proc/stat (Linux). Average since boot.
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
    #[allow(clippy::cast_precision_loss)]
    Ok(if total > 0 {
        ((total - idle) as f64 / total as f64) * 100.0
    } else {
        0.0
    })
}

#[cfg(not(target_os = "linux"))]
pub fn system_cpu_usage_percent() -> Result<f64, io::Error> {
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
}
