//! Utility functions for Linux sandbox capabilities detection

/// Check if cgroups v2 is available on the system
#[cfg(target_os = "linux")]
pub fn has_cgroups_v2() -> bool {
    // Check if cgroups v2 is mounted
    if let Ok(mount_output) = std::fs::read_to_string("/proc/mounts") {
        // Look for cgroup2 fs type
        return mount_output.contains("cgroup2");
    }
    
    // Alternative check: see if unified hierarchy is available
    std::path::Path::new("/sys/fs/cgroup/cgroup.controllers").exists()
}

/// Check if seccomp is available on the system
#[cfg(target_os = "linux")]
pub fn has_seccomp() -> bool {
    // Check if seccomp is available in the kernel
    // By checking if the seccomp directory exists in the kernel config
    if let Ok(config) = std::fs::read_to_string("/proc/config.gz") {
        return config.contains("CONFIG_SECCOMP=y");
    }
    
    // Try checking for seccomp presence via prctl
    unsafe {
        // PR_GET_SECCOMP = 21
        let ret = libc::prctl(21, 0, 0, 0, 0);
        ret >= 0
    }
}

/// Check if namespaces are available on the system
#[cfg(target_os = "linux")]
pub fn has_namespaces() -> bool {
    // Check if we can create a user namespace for the current process
    // This is a simple way to check if namespaces are supported
    unsafe {
        // CLONE_NEWUSER = 0x10000000
        let ret = libc::unshare(0x10000000);
        
        // If successful, restore the original namespace
        if ret == 0 {
            // We successfully created a user namespace, now revert
            let _ = libc::setns(libc::open("/proc/self/ns/user\0".as_ptr() as *const i8, libc::O_RDONLY), 0);
            return true;
        }
        
        // If we couldn't create a user namespace, check if other namespaces are available
        std::path::Path::new("/proc/self/ns").exists()
    }
}

/// Get the list of available namespaces on the system
#[cfg(target_os = "linux")]
pub fn get_available_namespaces() -> Vec<String> {
    let mut namespaces = Vec::new();
    
    // Check if namespace entries exist in /proc/self/ns/
    let ns_path = std::path::Path::new("/proc/self/ns");
    if let Ok(entries) = std::fs::read_dir(ns_path) {
        for entry in entries.filter_map(|result| result.ok()) {
            if let Some(filename) = entry.file_name().to_str() {
                namespaces.push(filename.to_string());
            }
        }
    }
    
    namespaces
}

// For non-Linux platforms, provide stub implementations
#[cfg(not(target_os = "linux"))]
pub fn has_cgroups_v2() -> bool {
    false
}

#[cfg(not(target_os = "linux"))]
pub fn has_seccomp() -> bool {
    false
}

#[cfg(not(target_os = "linux"))]
pub fn has_namespaces() -> bool {
    false
}

#[cfg(not(target_os = "linux"))]
pub fn get_available_namespaces() -> Vec<String> {
    Vec::new()
} 