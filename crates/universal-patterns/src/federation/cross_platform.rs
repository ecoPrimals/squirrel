// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Cross-Platform Module
//!
//! This module provides utilities for cross-platform compatibility and detection.
//!
//! ## Philosophy: Universal & Agnostic Rust
//!
//! Instead of platform-specific code branches (Windows | Mac | ARM), we create
//! **1 unified codebase** that works everywhere using runtime detection and
//! Rust's standard library abstractions.
//!
//! ## Design Principles:
//! - Runtime platform detection (not compile-time cfg)
//! - Use Rust std::path for universal path handling
//! - Abstract away platform differences
//! - Graceful degradation when features unavailable

use super::Platform;
use std::path::MAIN_SEPARATOR;

/// Cross-platform utilities
pub struct CrossPlatform;

impl CrossPlatform {
    /// Detect the current platform at runtime
    ///
    /// Uses Rust's cfg attributes only for detection, not for business logic.
    /// Business logic should use the returned Platform enum for runtime decisions.
    pub fn detect_current_platform() -> Platform {
        #[cfg(target_os = "linux")]
        return Platform::Linux(super::LinuxVariant::Generic("Detected".to_string()));

        #[cfg(target_os = "windows")]
        return Platform::Windows(super::WindowsVariant::Generic("Detected".to_string()));

        #[cfg(target_os = "macos")]
        return Platform::MacOS(super::MacOSVariant::Generic("Detected".to_string()));

        #[cfg(target_os = "android")]
        return Platform::Mobile(super::MobileVariant::Android);

        #[cfg(target_os = "ios")]
        return Platform::Mobile(super::MobileVariant::IOS);

        #[cfg(target_family = "wasm")]
        return Platform::WebAssembly;

        #[cfg(not(any(
            target_os = "linux",
            target_os = "windows",
            target_os = "macos",
            target_os = "android",
            target_os = "ios",
            target_family = "wasm"
        )))]
        return Platform::Linux(super::LinuxVariant::Generic("Unknown".to_string()));
    }

    /// Check if two platforms are compatible
    pub fn are_compatible(platform1: &Platform, platform2: &Platform) -> bool {
        // Simplified compatibility check using discriminant
        std::mem::discriminant(platform1) == std::mem::discriminant(platform2)
    }

    /// Get platform-specific configuration using universal patterns
    ///
    /// Uses Rust standard library abstractions where possible:
    /// - `std::path::MAIN_SEPARATOR` for path separator
    /// - `std::env::consts::EXE_EXTENSION` for executable extension
    /// - Runtime detection for platform-specific limits
    pub fn get_platform_config(platform: &Platform) -> PlatformConfig {
        // Use Rust's universal path separator constant
        let path_separator = MAIN_SEPARATOR;

        // Use Rust's universal executable extension constant
        let executable_extension = std::env::consts::EXE_EXTENSION;

        // Platform-specific limits (these vary by OS)
        let max_file_descriptors = match platform {
            Platform::Linux(_) => 65536,
            Platform::Windows(_) => 2048,
            Platform::MacOS(_) => 10240,
            Platform::Mobile(_) => 1024,
            Platform::Embedded(_) => 256,
            Platform::WebAssembly => 0, // No file descriptors in WASM
            _ => 1024,                  // Safe default
        };

        PlatformConfig {
            path_separator,
            executable_extension,
            max_file_descriptors,
        }
    }

    /// Get universal data directory
    ///
    /// Uses the `dirs` crate for platform-appropriate data directories.
    /// This is the idiomatic Rust way to handle platform-specific paths.
    ///
    /// # Returns
    ///
    /// The platform-appropriate data directory for the application
    pub fn get_data_dir(app_name: &str) -> std::path::PathBuf {
        // Use dirs crate for universal data directory discovery
        // Falls back gracefully on platforms without standard dirs
        dirs::data_dir()
            .unwrap_or_else(|| {
                // Fallback to current directory
                std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("./data"))
            })
            .join(app_name)
    }

    /// Get universal config directory
    ///
    /// Uses the `dirs` crate for platform-appropriate config directories.
    ///
    /// # Returns
    ///
    /// The platform-appropriate config directory for the application
    pub fn get_config_dir(app_name: &str) -> std::path::PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| {
                std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("./config"))
            })
            .join(app_name)
    }

    /// Get universal runtime directory
    ///
    /// Returns the appropriate runtime directory for the current platform:
    /// - Linux: $XDG_RUNTIME_DIR or /tmp
    /// - Windows: %TEMP%
    /// - macOS: ~/Library/Application Support
    /// - Other: ./runtime
    ///
    /// # Returns
    ///
    /// The platform-appropriate runtime directory
    pub fn get_runtime_dir(app_name: &str) -> std::path::PathBuf {
        #[cfg(target_os = "linux")]
        {
            // XDG_RUNTIME_DIR on Linux
            if let Ok(xdg_runtime) = std::env::var("XDG_RUNTIME_DIR") {
                std::path::PathBuf::from(xdg_runtime).join(app_name)
            } else {
                std::path::PathBuf::from("/tmp").join(app_name)
            }
        }

        #[cfg(target_os = "windows")]
        {
            // TEMP on Windows
            if let Ok(temp) = std::env::var("TEMP") {
                return std::path::PathBuf::from(temp).join(app_name);
            }
            return std::path::PathBuf::from("C:\\Temp").join(app_name);
        }

        #[cfg(target_os = "macos")]
        {
            // User library on macOS
            if let Some(home) = dirs::home_dir() {
                return home
                    .join("Library")
                    .join("Application Support")
                    .join(app_name);
            }
            return std::path::PathBuf::from("/tmp").join(app_name);
        }

        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        {
            std::path::PathBuf::from("./runtime").join(app_name)
        }
    }
}

/// Platform-specific configuration
///
/// Uses universal Rust constants where possible (MAIN_SEPARATOR, EXE_EXTENSION)
/// to avoid hardcoding platform-specific values.
#[derive(Debug, Clone)]
pub struct PlatformConfig {
    /// Path separator for the platform (uses std::path::MAIN_SEPARATOR)
    pub path_separator: char,
    /// Executable file extension (uses std::env::consts::EXE_EXTENSION)
    pub executable_extension: &'static str,
    /// Maximum file descriptors (platform-specific limit)
    pub max_file_descriptors: u32,
}
