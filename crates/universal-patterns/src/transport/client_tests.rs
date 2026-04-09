// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use super::*;
use std::io;

#[test]
fn test_transport_config_default() {
    let config = TransportConfig::default();
    assert!(config.enable_fallback);
    assert_eq!(config.timeout_ms, 5000);
    assert!(config.preferred_transport.is_none());
    assert!(config.socket_base_dir.is_none());
}

#[test]
fn test_transport_config_custom() {
    let config = TransportConfig {
        preferred_transport: Some(TransportType::Tcp),
        enable_fallback: false,
        timeout_ms: 10000,
        socket_base_dir: Some(PathBuf::from("/tmp/custom")),
    };

    assert_eq!(config.preferred_transport, Some(TransportType::Tcp));
    assert!(!config.enable_fallback);
    assert_eq!(config.timeout_ms, 10000);
    assert_eq!(config.socket_base_dir, Some(PathBuf::from("/tmp/custom")));
}

#[test]
fn test_transport_hierarchy_linux() {
    #[cfg(target_os = "linux")]
    {
        let config = TransportConfig::default();
        let hierarchy = UniversalTransport::get_transport_hierarchy(&config);
        assert_eq!(hierarchy.len(), 3);
        assert_eq!(hierarchy[0], TransportType::UnixAbstract);
        assert_eq!(hierarchy[1], TransportType::UnixFilesystem);
        assert_eq!(hierarchy[2], TransportType::Tcp);
    }
}

#[test]
fn test_transport_hierarchy_unix_non_linux() {
    #[cfg(all(unix, not(target_os = "linux")))]
    {
        let config = TransportConfig::default();
        let hierarchy = UniversalTransport::get_transport_hierarchy(&config);
        assert_eq!(hierarchy.len(), 2);
        assert_eq!(hierarchy[0], TransportType::UnixFilesystem);
        assert_eq!(hierarchy[1], TransportType::Tcp);
    }
}

#[test]
fn test_transport_hierarchy_windows() {
    #[cfg(windows)]
    {
        let config = TransportConfig::default();
        let hierarchy = UniversalTransport::get_transport_hierarchy(&config);
        assert_eq!(hierarchy.len(), 2);
        assert_eq!(hierarchy[0], TransportType::NamedPipe);
        assert_eq!(hierarchy[1], TransportType::Tcp);
    }
}

#[test]
fn test_transport_hierarchy_other_platform() {
    #[cfg(not(any(unix, windows)))]
    {
        let config = TransportConfig::default();
        let hierarchy = UniversalTransport::get_transport_hierarchy(&config);
        assert_eq!(hierarchy.len(), 1);
        assert_eq!(hierarchy[0], TransportType::Tcp);
    }
}

#[test]
fn test_transport_hierarchy_with_preference() {
    let config = TransportConfig {
        preferred_transport: Some(TransportType::Tcp),
        ..Default::default()
    };

    let hierarchy = UniversalTransport::get_transport_hierarchy(&config);
    assert_eq!(hierarchy.len(), 2);
    assert_eq!(hierarchy[0], TransportType::Tcp);
    assert_eq!(hierarchy[1], TransportType::Tcp);
}

#[test]
fn test_transport_hierarchy_with_preference_unix() {
    #[cfg(unix)]
    {
        let config = TransportConfig {
            preferred_transport: Some(TransportType::UnixFilesystem),
            ..Default::default()
        };

        let hierarchy = UniversalTransport::get_transport_hierarchy(&config);
        assert_eq!(hierarchy.len(), 2);
        assert_eq!(hierarchy[0], TransportType::UnixFilesystem);
        assert_eq!(hierarchy[1], TransportType::Tcp);
    }
}

#[test]
fn test_transport_hierarchy_no_fallback() {
    let config = TransportConfig {
        preferred_transport: Some(TransportType::Tcp),
        enable_fallback: false,
        ..Default::default()
    };

    let hierarchy = UniversalTransport::get_transport_hierarchy(&config);
    assert_eq!(hierarchy.len(), 1);
    assert_eq!(hierarchy[0], TransportType::Tcp);
}

#[test]
fn test_is_platform_constraint_unsupported() {
    let error = io::Error::new(io::ErrorKind::Unsupported, "Address family not supported");
    assert!(UniversalTransport::is_platform_constraint(&error));
}

#[test]
fn test_is_platform_constraint_connection_refused() {
    let error = io::Error::new(io::ErrorKind::ConnectionRefused, "Connection refused");
    assert!(UniversalTransport::is_platform_constraint(&error));
}

#[test]
fn test_is_platform_constraint_not_found() {
    let error = io::Error::new(io::ErrorKind::NotFound, "No such file or directory");
    assert!(UniversalTransport::is_platform_constraint(&error));
}

#[test]
fn test_is_platform_constraint_permission_denied() {
    // Permission denied may or may not be a platform constraint
    // depending on SELinux/AppArmor detection
    let error = io::Error::new(io::ErrorKind::PermissionDenied, "Permission denied");
    // This will check actual system state, so we just verify it doesn't panic
    let _ = UniversalTransport::is_platform_constraint(&error);
}

#[test]
fn test_is_platform_constraint_real_error() {
    let error = io::Error::new(io::ErrorKind::NetworkUnreachable, "Network unreachable");
    assert!(!UniversalTransport::is_platform_constraint(&error));
}

#[test]
fn test_is_platform_constraint_host_unreachable() {
    let error = io::Error::new(io::ErrorKind::HostUnreachable, "Host unreachable");
    assert!(!UniversalTransport::is_platform_constraint(&error));
}

#[test]
fn test_is_platform_constraint_timed_out() {
    let error = io::Error::new(io::ErrorKind::TimedOut, "Connection timed out");
    assert!(!UniversalTransport::is_platform_constraint(&error));
}

#[test]
fn test_socket_path_generation_default() {
    let config = TransportConfig::default();
    let path = UniversalTransport::get_socket_path("test_service", &config);

    assert!(path.to_string_lossy().contains("test_service.sock"));
    assert!(path.to_string_lossy().ends_with(".sock"));
}

#[test]
fn test_socket_path_generation_custom_dir() {
    let config = TransportConfig {
        socket_base_dir: Some(PathBuf::from("/tmp/custom_sockets")),
        ..Default::default()
    };

    let path = UniversalTransport::get_socket_path("my_service", &config);

    assert_eq!(path, PathBuf::from("/tmp/custom_sockets/my_service.sock"));
}

#[test]
fn test_socket_path_generation_different_services() {
    let config = TransportConfig::default();
    let path1 = UniversalTransport::get_socket_path("service1", &config);
    let path2 = UniversalTransport::get_socket_path("service2", &config);

    assert_ne!(path1, path2);
    assert!(path1.to_string_lossy().contains("service1"));
    assert!(path2.to_string_lossy().contains("service2"));
}

#[tokio::test]
async fn test_try_connect_invalid_tcp_address() {
    let config = TransportConfig {
        timeout_ms: 100, // Short timeout for fast failure
        ..Default::default()
    };

    // Try to connect to an invalid TCP address (should timeout or fail quickly)
    let result = UniversalTransport::try_connect(
        "nonexistent_service_that_does_not_exist_12345",
        TransportType::Tcp,
        &config,
    )
    .await;

    // Should fail with either timeout or connection error
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(
        error.kind() == io::ErrorKind::TimedOut
            || error.kind() == io::ErrorKind::ConnectionRefused
            || error.kind() == io::ErrorKind::Other
    );
}

#[tokio::test]
async fn test_try_connect_invalid_unix_socket() {
    #[cfg(unix)]
    {
        let config = TransportConfig {
            timeout_ms: 100,
            socket_base_dir: Some(PathBuf::from("/nonexistent/path/that/does/not/exist")),
            ..Default::default()
        };

        let result = UniversalTransport::try_connect(
            "nonexistent_service",
            TransportType::UnixFilesystem,
            &config,
        )
        .await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        // Should fail with NotFound or TimedOut
        assert!(error.kind() == io::ErrorKind::NotFound || error.kind() == io::ErrorKind::TimedOut);
    }
}

#[tokio::test]
async fn test_try_connect_unsupported_transport() {
    #[cfg(not(unix))]
    {
        let config = TransportConfig::default();
        let result =
            UniversalTransport::try_connect("test", TransportType::UnixAbstract, &config).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::Unsupported);
    }

    #[cfg(not(windows))]
    {
        let config = TransportConfig::default();
        let result =
            UniversalTransport::try_connect("test", TransportType::NamedPipe, &config).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::Unsupported);
    }
}

#[tokio::test]
async fn test_try_connect_inprocess() {
    let config = TransportConfig::default();
    let result =
        UniversalTransport::try_connect("test_service", TransportType::InProcess, &config).await;

    assert!(result.is_ok());
    let transport = result.expect("should succeed");
    assert_eq!(transport.transport_type(), TransportType::InProcess);
}

#[test]
fn test_transport_type_detection() {
    // Test that transport_type() returns correct type for each variant
    let inprocess = UniversalTransport::InProcess(InProcessTransport::new());
    assert_eq!(inprocess.transport_type(), TransportType::InProcess);
}

#[test]
fn test_config_parsing_timeout() {
    let mut config = TransportConfig::default();
    assert_eq!(config.timeout_ms, 5000);

    config.timeout_ms = 0;
    assert_eq!(config.timeout_ms, 0);

    config.timeout_ms = u64::MAX;
    assert_eq!(config.timeout_ms, u64::MAX);
}

#[test]
fn test_config_parsing_preferred_transport() {
    let mut config = TransportConfig::default();
    assert!(config.preferred_transport.is_none());

    config.preferred_transport = Some(TransportType::Tcp);
    assert_eq!(config.preferred_transport, Some(TransportType::Tcp));

    config.preferred_transport = Some(TransportType::UnixFilesystem);
    assert_eq!(
        config.preferred_transport,
        Some(TransportType::UnixFilesystem)
    );

    config.preferred_transport = None;
    assert!(config.preferred_transport.is_none());
}

#[test]
fn test_config_parsing_fallback_flag() {
    let mut config = TransportConfig::default();
    assert!(config.enable_fallback);

    config.enable_fallback = false;
    assert!(!config.enable_fallback);

    config.enable_fallback = true;
    assert!(config.enable_fallback);
}

#[test]
fn test_discover_tcp_endpoint_reads_discovery_file() {
    use crate::transport::discovery::discover_tcp_endpoint;
    use crate::transport::types::IpcEndpoint;
    let name = format!("ut-transport-{}", std::process::id());
    let path = std::path::PathBuf::from("/tmp").join(format!("{name}-ipc-port"));
    std::fs::write(&path, "tcp:127.0.0.1:65533\n").expect("should succeed");
    let ep = discover_tcp_endpoint(&name).expect("tcp discovery");
    match ep {
        IpcEndpoint::TcpLocal(addr) => assert_eq!(addr.port(), 65533),
        #[cfg(unix)]
        IpcEndpoint::UnixSocket(_) => unreachable!("expected tcp from file"),
        #[cfg(windows)]
        IpcEndpoint::NamedPipe(_) => unreachable!("expected tcp from file"),
    }
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_discover_ipc_endpoint_delegates() {
    let name = format!("ut-no-socket-{}", std::process::id());
    let r = UniversalTransport::discover_ipc_endpoint(&name);
    assert!(r.is_err());
}
