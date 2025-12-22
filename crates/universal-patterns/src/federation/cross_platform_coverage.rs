//! Comprehensive tests for cross-platform module
//!
//! This module provides complete test coverage for cross-platform detection,
//! compatibility checking, and configuration. Target: 0% → 100% coverage.

#[cfg(test)]
mod cross_platform_tests {
    use super::super::cross_platform::*;
    use super::super::{
        CloudProvider, ContainerRuntime, LinuxVariant, MacOSVariant, Platform, WindowsVariant,
    };

    /// Test current platform detection
    #[test]
    fn test_detect_current_platform() {
        let platform = CrossPlatform::detect_current_platform();

        // Platform should be detected (one of Linux/Windows/MacOS)
        match platform {
            Platform::Linux(_) => {
                // Running on Linux - expected on most CI/CD systems
                assert!(true);
            }
            Platform::Windows(_) => {
                // Running on Windows
                assert!(true);
            }
            Platform::MacOS(_) => {
                // Running on macOS
                assert!(true);
            }
            _ => {
                // Should not reach here with current implementation
                panic!("Unexpected platform detected");
            }
        }
    }

    /// Test platform compatibility - same variants
    #[test]
    fn test_platforms_compatible_same_type() {
        let linux1 = Platform::Linux(LinuxVariant::Ubuntu);
        let linux2 = Platform::Linux(LinuxVariant::Debian);

        // Same platform type (Linux) should be compatible
        assert!(CrossPlatform::are_compatible(&linux1, &linux2));
    }

    /// Test platform compatibility - different variants
    #[test]
    fn test_platforms_incompatible_different_types() {
        let linux = Platform::Linux(LinuxVariant::Generic("test".to_string()));
        let windows = Platform::Windows(WindowsVariant::Generic("test".to_string()));

        // Different platform types should not be compatible
        assert!(!CrossPlatform::are_compatible(&linux, &windows));
    }

    /// Test platform compatibility - Windows and MacOS
    #[test]
    fn test_windows_macos_incompatible() {
        let windows = Platform::Windows(WindowsVariant::WindowsServer2019);
        let macos = Platform::MacOS(MacOSVariant::Monterey);

        assert!(!CrossPlatform::are_compatible(&windows, &macos));
    }

    /// Test Linux platform configuration
    #[test]
    fn test_linux_platform_config() {
        let linux = Platform::Linux(LinuxVariant::Ubuntu);
        let config = CrossPlatform::get_platform_config(&linux);

        assert_eq!(config.path_separator, "/");
        assert_eq!(config.executable_extension, "");
        assert_eq!(config.max_file_descriptors, 65536);
    }

    /// Test Windows platform configuration
    #[test]
    fn test_windows_platform_config() {
        let windows = Platform::Windows(WindowsVariant::Windows10);
        let config = CrossPlatform::get_platform_config(&windows);

        assert_eq!(config.path_separator, "\\");
        assert_eq!(config.executable_extension, ".exe");
        assert_eq!(config.max_file_descriptors, 2048);
    }

    /// Test MacOS platform configuration
    #[test]
    fn test_macos_platform_config() {
        let macos = Platform::MacOS(MacOSVariant::Monterey);
        let config = CrossPlatform::get_platform_config(&macos);

        assert_eq!(config.path_separator, "/");
        assert_eq!(config.executable_extension, "");
        assert_eq!(config.max_file_descriptors, 10240);
    }

    /// Test container platform configuration (default case)
    #[test]
    fn test_container_platform_config() {
        let container = Platform::Container(ContainerRuntime::Docker);
        let config = CrossPlatform::get_platform_config(&container);

        // Container uses default config
        assert_eq!(config.path_separator, "/");
        assert_eq!(config.executable_extension, "");
        assert_eq!(config.max_file_descriptors, 1024);
    }

    /// Test cloud platform configuration (default case)
    #[test]
    fn test_cloud_platform_config() {
        let cloud = Platform::Cloud(CloudProvider::AWS);
        let config = CrossPlatform::get_platform_config(&cloud);

        // Cloud uses default config
        assert_eq!(config.path_separator, "/");
        assert_eq!(config.executable_extension, "");
        assert_eq!(config.max_file_descriptors, 1024);
    }

    /// Test PlatformConfig clone
    #[test]
    fn test_platform_config_clone() {
        let config = PlatformConfig {
            path_separator: "/",
            executable_extension: ".sh",
            max_file_descriptors: 4096,
        };

        let cloned = config.clone();

        assert_eq!(config.path_separator, cloned.path_separator);
        assert_eq!(config.executable_extension, cloned.executable_extension);
        assert_eq!(config.max_file_descriptors, cloned.max_file_descriptors);
    }

    /// Test PlatformConfig debug formatting
    #[test]
    fn test_platform_config_debug() {
        let config = PlatformConfig {
            path_separator: "\\",
            executable_extension: ".exe",
            max_file_descriptors: 2048,
        };

        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("PlatformConfig"));
        assert!(debug_str.contains("2048"));
    }

    /// Test compatibility with all Linux variants
    #[test]
    fn test_all_linux_variants_compatible() {
        let variants = vec![
            Platform::Linux(LinuxVariant::Ubuntu),
            Platform::Linux(LinuxVariant::Debian),
            Platform::Linux(LinuxVariant::RHEL),
            Platform::Linux(LinuxVariant::CentOS),
            Platform::Linux(LinuxVariant::Fedora),
            Platform::Linux(LinuxVariant::Arch),
            Platform::Linux(LinuxVariant::Alpine),
            Platform::Linux(LinuxVariant::Generic("custom".to_string())),
        ];

        // All Linux variants should be compatible with each other
        for i in 0..variants.len() {
            for j in i + 1..variants.len() {
                assert!(
                    CrossPlatform::are_compatible(&variants[i], &variants[j]),
                    "Linux variants should be compatible: {:?} and {:?}",
                    variants[i],
                    variants[j]
                );
            }
        }
    }

    /// Test compatibility with all Windows variants
    #[test]
    fn test_all_windows_variants_compatible() {
        let variants = vec![
            Platform::Windows(WindowsVariant::Windows10),
            Platform::Windows(WindowsVariant::Windows11),
            Platform::Windows(WindowsVariant::WindowsServer2019),
            Platform::Windows(WindowsVariant::WindowsServer2022),
            Platform::Windows(WindowsVariant::Generic("custom".to_string())),
        ];

        // All Windows variants should be compatible with each other
        for i in 0..variants.len() {
            for j in i + 1..variants.len() {
                assert!(
                    CrossPlatform::are_compatible(&variants[i], &variants[j]),
                    "Windows variants should be compatible"
                );
            }
        }
    }

    /// Test compatibility with all MacOS variants
    #[test]
    fn test_all_macos_variants_compatible() {
        let variants = vec![
            Platform::MacOS(MacOSVariant::Monterey),
            Platform::MacOS(MacOSVariant::Ventura),
            Platform::MacOS(MacOSVariant::Sonoma),
            Platform::MacOS(MacOSVariant::Generic("custom".to_string())),
        ];

        // All MacOS variants should be compatible with each other
        for i in 0..variants.len() {
            for j in i + 1..variants.len() {
                assert!(
                    CrossPlatform::are_compatible(&variants[i], &variants[j]),
                    "MacOS variants should be compatible"
                );
            }
        }
    }

    /// Test incompatibility across major platform types
    #[test]
    fn test_major_platforms_incompatible() {
        let linux = Platform::Linux(LinuxVariant::Ubuntu);
        let windows = Platform::Windows(WindowsVariant::Windows10);
        let macos = Platform::MacOS(MacOSVariant::Monterey);
        let container = Platform::Container(ContainerRuntime::Docker);
        let cloud = Platform::Cloud(CloudProvider::AWS);

        // Cross-type compatibility checks
        assert!(!CrossPlatform::are_compatible(&linux, &windows));
        assert!(!CrossPlatform::are_compatible(&linux, &macos));
        assert!(!CrossPlatform::are_compatible(&linux, &container));
        assert!(!CrossPlatform::are_compatible(&linux, &cloud));

        assert!(!CrossPlatform::are_compatible(&windows, &macos));
        assert!(!CrossPlatform::are_compatible(&windows, &container));
        assert!(!CrossPlatform::are_compatible(&windows, &cloud));

        assert!(!CrossPlatform::are_compatible(&macos, &container));
        assert!(!CrossPlatform::are_compatible(&macos, &cloud));

        assert!(!CrossPlatform::are_compatible(&container, &cloud));
    }

    /// Test self-compatibility (same platform instance)
    #[test]
    fn test_self_compatibility() {
        let linux = Platform::Linux(LinuxVariant::Ubuntu);

        // A platform should always be compatible with itself
        assert!(CrossPlatform::are_compatible(&linux, &linux));
    }

    /// Test container runtime compatibility
    #[test]
    fn test_container_runtimes_compatible() {
        let docker = Platform::Container(ContainerRuntime::Docker);
        let podman = Platform::Container(ContainerRuntime::Podman);
        let containerd = Platform::Container(ContainerRuntime::Containerd);

        // All container runtimes should be compatible
        assert!(CrossPlatform::are_compatible(&docker, &podman));
        assert!(CrossPlatform::are_compatible(&docker, &containerd));
        assert!(CrossPlatform::are_compatible(&podman, &containerd));
    }

    /// Test cloud provider compatibility
    #[test]
    fn test_cloud_providers_compatible() {
        let aws = Platform::Cloud(CloudProvider::AWS);
        let azure = Platform::Cloud(CloudProvider::Azure);
        let gcp = Platform::Cloud(CloudProvider::GCP);

        // All cloud providers should be compatible (same platform type)
        assert!(CrossPlatform::are_compatible(&aws, &azure));
        assert!(CrossPlatform::are_compatible(&aws, &gcp));
        assert!(CrossPlatform::are_compatible(&azure, &gcp));
    }

    /// Test platform config with minimum file descriptors
    #[test]
    fn test_platform_config_minimum_fds() {
        let container = Platform::Container(ContainerRuntime::Docker);
        let config = CrossPlatform::get_platform_config(&container);

        // Default config has minimum FDs
        assert_eq!(config.max_file_descriptors, 1024);
    }

    /// Test platform config with maximum file descriptors
    #[test]
    fn test_platform_config_maximum_fds() {
        let linux = Platform::Linux(LinuxVariant::Ubuntu);
        let config = CrossPlatform::get_platform_config(&linux);

        // Linux has highest FD limit
        assert_eq!(config.max_file_descriptors, 65536);
    }

    /// Test Windows-specific file extensions
    #[test]
    fn test_windows_executable_extension() {
        let windows = Platform::Windows(WindowsVariant::WindowsServer2022);
        let config = CrossPlatform::get_platform_config(&windows);

        // Only Windows should have .exe extension
        assert_eq!(config.executable_extension, ".exe");
    }

    /// Test Unix-like path separators
    #[test]
    fn test_unix_path_separators() {
        let linux = Platform::Linux(LinuxVariant::Debian);
        let macos = Platform::MacOS(MacOSVariant::Ventura);
        let container = Platform::Container(ContainerRuntime::Podman);

        let linux_config = CrossPlatform::get_platform_config(&linux);
        let macos_config = CrossPlatform::get_platform_config(&macos);
        let container_config = CrossPlatform::get_platform_config(&container);

        // Unix-like systems use forward slash
        assert_eq!(linux_config.path_separator, "/");
        assert_eq!(macos_config.path_separator, "/");
        assert_eq!(container_config.path_separator, "/");
    }

    /// Test Windows path separator
    #[test]
    fn test_windows_path_separator() {
        let windows = Platform::Windows(WindowsVariant::Windows11);
        let config = CrossPlatform::get_platform_config(&windows);

        // Windows uses backslash
        assert_eq!(config.path_separator, "\\");
    }
}
