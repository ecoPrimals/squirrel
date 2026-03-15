// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Universal transport client implementation

use std::io::{self, Result as IoResult};
use std::path::PathBuf;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::{TcpStream, UnixStream};

#[cfg(windows)]
use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeClient};

use crate::transport::types::{InProcessTransport, IpcEndpoint, TransportConfig, TransportType};

/// Universal transport abstraction
///
/// Represents a connection that works across all platforms using the
/// most appropriate transport mechanism.
///
/// ## Platform Selection
///
/// - **Unix (Linux, macOS, BSD)**: Unix domain sockets (abstract or filesystem)
/// - **Windows**: Named pipes
/// - **All platforms**: TCP fallback
/// - **Testing/Embedded**: In-process channels
#[derive(Debug)]
pub enum UniversalTransport {
    /// Unix domain socket (Linux, macOS, BSD)
    #[cfg(unix)]
    UnixSocket(UnixStream),

    /// Named pipe (Windows)
    #[cfg(windows)]
    NamedPipe(NamedPipeClient),

    /// TCP connection (universal fallback)
    Tcp(TcpStream),

    /// In-process channel (testing, embedded)
    InProcess(InProcessTransport),
}

impl UniversalTransport {
    /// Detect if an error is a platform constraint (not a real error)
    ///
    /// Platform constraints indicate the platform lacks support for
    /// the attempted transport, requiring automatic fallback.
    ///
    /// ## Platform Constraints vs Real Errors
    ///
    /// Platform constraints (expected, adapt automatically):
    /// - SELinux/AppArmor blocking Unix sockets (Android, hardened Linux)
    /// - Address family not supported (platform lacks Unix sockets)
    /// - Connection refused (socket doesn't exist yet)
    /// - Not found (socket path doesn't exist)
    ///
    /// Real errors (unexpected, should fail):
    /// - Network unreachable
    /// - Host unreachable
    /// - Protocol errors
    pub(crate) fn is_platform_constraint(error: &io::Error) -> bool {
        match error.kind() {
            // Permission denied often means SELinux/AppArmor blocking
            io::ErrorKind::PermissionDenied => Self::is_security_constraint(),

            // Address family not supported (platform lacks Unix sockets)
            io::ErrorKind::Unsupported => true,

            // Connection refused: socket doesn't exist (expected for fallback)
            io::ErrorKind::ConnectionRefused => true,

            // Not found: socket path doesn't exist (expected for fallback)
            io::ErrorKind::NotFound => true,

            _ => false,
        }
    }

    /// Check if security constraints (SELinux, AppArmor) are enforcing
    ///
    /// Used to distinguish permission errors caused by security policies
    /// (platform constraint) from real permission errors.
    fn is_security_constraint() -> bool {
        // Check SELinux enforcement (Android, Fedora, RHEL)
        if let Ok(enforce) = std::fs::read_to_string("/sys/fs/selinux/enforce") {
            if enforce.trim() == "1" {
                tracing::debug!("SELinux is enforcing (platform constraint detected)");
                return true;
            }
        }

        // Check AppArmor (Ubuntu, Debian)
        if std::fs::metadata("/sys/kernel/security/apparmor").is_ok() {
            tracing::debug!("AppArmor is active (platform constraint detected)");
            return true;
        }

        false
    }

    /// Connect to a service using automatic transport selection
    ///
    /// Automatically selects the best transport for the current platform
    /// with fallback to TCP if preferred transport fails.
    ///
    /// ## Isomorphic IPC
    ///
    /// This implements the Try→Detect→Adapt→Succeed pattern:
    /// 1. **TRY** optimal transport (Unix sockets, Named pipes)
    /// 2. **DETECT** platform constraints vs real errors
    /// 3. **ADAPT** automatically to TCP fallback
    /// 4. **SUCCEED** or fail with real error
    ///
    /// # Arguments
    ///
    /// * `service_name` - Name of the service to connect to
    /// * `config` - Optional transport configuration (uses defaults if None)
    ///
    /// # Returns
    ///
    /// A connected `UniversalTransport` instance
    ///
    /// # Example
    ///
    /// ```ignore,no_run
    /// use universal_patterns::transport::UniversalTransport;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Automatic platform detection and connection (isomorphic)
    /// let transport = UniversalTransport::connect("squirrel", None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect(service_name: &str, config: Option<TransportConfig>) -> IoResult<Self> {
        let config = config.unwrap_or_default();

        // Determine transport hierarchy based on platform
        let transport_order = Self::get_transport_hierarchy(&config);

        tracing::info!("🔌 Starting IPC client connection (isomorphic mode)...");
        tracing::info!("   Service: {}", service_name);

        let mut last_error = None;

        for transport_type in transport_order {
            tracing::info!("   Trying {:?}...", transport_type);

            match Self::try_connect(service_name, transport_type, &config).await {
                Ok(transport) => {
                    tracing::info!("✅ Connected using {:?}", transport_type);
                    return Ok(transport);
                }

                // DETECT: Platform constraint (expected, adapt)
                Err(e) if Self::is_platform_constraint(&e) => {
                    tracing::warn!("⚠️  {:?} unavailable: {}", transport_type, e);
                    tracing::warn!("   Detected platform constraint, adapting...");

                    last_error = Some(e);

                    if !config.enable_fallback {
                        break;
                    }
                    // Continue to next transport in hierarchy
                }

                // Real error (unexpected, fail)
                Err(e) => {
                    tracing::error!("❌ Real error (not platform constraint): {}", e);
                    return Err(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "Failed to connect to service: {} (all transports exhausted)",
                    service_name
                ),
            )
        }))
    }

    /// Get transport hierarchy for the current platform
    ///
    /// Returns transport types in order of preference, with automatic
    /// fallback to TCP as the universal last resort.
    ///
    /// ## Hierarchy
    ///
    /// - **Linux**: Abstract socket → Filesystem socket → TCP
    /// - **macOS**: Filesystem socket → TCP
    /// - **Windows**: Named pipe → TCP
    /// - **Other**: TCP only
    pub(crate) fn get_transport_hierarchy(config: &TransportConfig) -> Vec<TransportType> {
        // If explicit preference, try that first
        if let Some(preferred) = config.preferred_transport {
            if config.enable_fallback {
                return vec![preferred, TransportType::Tcp];
            } else {
                return vec![preferred];
            }
        }

        // Platform-appropriate hierarchy
        #[cfg(target_os = "linux")]
        {
            vec![
                TransportType::UnixAbstract,
                TransportType::UnixFilesystem,
                TransportType::Tcp,
            ]
        }

        #[cfg(all(unix, not(target_os = "linux")))]
        {
            vec![TransportType::UnixFilesystem, TransportType::Tcp]
        }

        #[cfg(windows)]
        {
            vec![TransportType::NamedPipe, TransportType::Tcp]
        }

        #[cfg(not(any(unix, windows)))]
        {
            vec![TransportType::Tcp]
        }
    }

    /// Try to connect using a specific transport type
    ///
    /// Attempts connection with the specified transport mechanism.
    ///
    /// # Arguments
    ///
    /// * `service_name` - Name of the service to connect to
    /// * `transport_type` - Type of transport to use
    /// * `config` - Transport configuration
    ///
    /// # Returns
    ///
    /// A connected `UniversalTransport` instance or an error
    async fn try_connect(
        service_name: &str,
        transport_type: TransportType,
        config: &TransportConfig,
    ) -> IoResult<Self> {
        match transport_type {
            #[cfg(unix)]
            TransportType::UnixAbstract => {
                #[cfg(target_os = "linux")]
                {
                    // Abstract socket: starts with null byte
                    let path = format!("\0{}", service_name);
                    let stream = tokio::time::timeout(
                        std::time::Duration::from_millis(config.timeout_ms),
                        UnixStream::connect(path),
                    )
                    .await
                    .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "Connection timeout"))??;

                    Ok(UniversalTransport::UnixSocket(stream))
                }

                #[cfg(not(target_os = "linux"))]
                {
                    Err(io::Error::new(
                        io::ErrorKind::Unsupported,
                        "Abstract sockets only supported on Linux",
                    ))
                }
            }

            #[cfg(unix)]
            TransportType::UnixFilesystem => {
                // Filesystem socket
                let socket_path = Self::get_socket_path(service_name, config);
                let stream = tokio::time::timeout(
                    std::time::Duration::from_millis(config.timeout_ms),
                    UnixStream::connect(&socket_path),
                )
                .await
                .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "Connection timeout"))??;

                Ok(UniversalTransport::UnixSocket(stream))
            }

            #[cfg(windows)]
            TransportType::NamedPipe => {
                // Named pipe: \\.\pipe\name
                let pipe_name = format!(r"\\.\pipe\{}", service_name);
                let client = ClientOptions::new().open(&pipe_name)?;

                Ok(UniversalTransport::NamedPipe(client))
            }

            TransportType::Tcp => {
                // TCP fallback: localhost with port from service registry
                // In a real implementation, this would query the service registry
                // For now, use a default port mapping
                let port = Self::get_tcp_port(service_name);
                let addr = format!("127.0.0.1:{}", port);

                let stream = tokio::time::timeout(
                    std::time::Duration::from_millis(config.timeout_ms),
                    TcpStream::connect(&addr),
                )
                .await
                .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "Connection timeout"))??;

                Ok(UniversalTransport::Tcp(stream))
            }

            TransportType::InProcess => {
                // In-process channel for testing
                Ok(UniversalTransport::InProcess(InProcessTransport::new()))
            }

            #[cfg(not(unix))]
            TransportType::UnixAbstract | TransportType::UnixFilesystem => Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "Unix sockets not supported on this platform",
            )),

            #[cfg(not(windows))]
            TransportType::NamedPipe => Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "Named pipes not supported on this platform",
            )),
        }
    }

    /// Get filesystem socket path for a service
    ///
    /// Returns platform-appropriate socket path using universal directory
    /// resolution from the CrossPlatform module.
    pub(crate) fn get_socket_path(service_name: &str, config: &TransportConfig) -> PathBuf {
        use crate::federation::cross_platform::CrossPlatform;

        let base_dir = config
            .socket_base_dir
            .clone()
            .unwrap_or_else(|| CrossPlatform::get_runtime_dir("squirrel"));

        base_dir.join(format!("{}.sock", service_name))
    }

    /// Get TCP port for a service
    ///
    /// In a real implementation, this would query the service registry.
    /// For now, uses a simple mapping.
    fn get_tcp_port(service_name: &str) -> u16 {
        // Use universal-constants for port resolution
        use universal_constants::network::get_service_port;
        get_service_port(service_name)
    }

    /// Discover IPC endpoint for a service
    ///
    /// Automatically discovers the correct endpoint, whether Unix socket,
    /// Named pipe, or TCP fallback.
    ///
    /// ## Isomorphic Discovery
    ///
    /// Tries in order:
    /// 1. Unix domain socket (optimal)
    /// 2. Named pipe (Windows)
    /// 3. TCP discovery file
    ///
    /// # Arguments
    ///
    /// * `service_name` - Name of the service to discover
    ///
    /// # Returns
    ///
    /// The discovered IPC endpoint
    ///
    /// # Example
    ///
    /// ```ignore,no_run
    /// use universal_patterns::transport::UniversalTransport;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Discover endpoint (Unix socket OR TCP)
    /// let endpoint = UniversalTransport::discover_ipc_endpoint("squirrel")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn discover_ipc_endpoint(service_name: &str) -> IoResult<IpcEndpoint> {
        crate::transport::discovery::discover_ipc_endpoint(service_name)
    }

    /// Connect using discovered endpoint
    ///
    /// Connects to a service using automatic endpoint discovery.
    ///
    /// ## Isomorphic Connection
    ///
    /// This is the recommended way to connect when you don't know if the
    /// server is using Unix sockets or TCP fallback.
    ///
    /// # Example
    ///
    /// ```ignore,no_run
    /// use universal_patterns::transport::UniversalTransport;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Automatically discovers and connects (Unix OR TCP)
    /// let transport = UniversalTransport::connect_discovered("squirrel").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect_discovered(service_name: &str) -> IoResult<Self> {
        tracing::info!("🔍 Discovering IPC endpoint for {}...", service_name);

        let endpoint = Self::discover_ipc_endpoint(service_name)?;

        tracing::info!("   Found: {:?}", endpoint);

        match endpoint {
            #[cfg(unix)]
            IpcEndpoint::UnixSocket(path) => {
                let stream = UnixStream::connect(path).await?;
                Ok(UniversalTransport::UnixSocket(stream))
            }
            IpcEndpoint::TcpLocal(addr) => {
                let stream = TcpStream::connect(addr).await?;
                Ok(UniversalTransport::Tcp(stream))
            }
            #[cfg(windows)]
            IpcEndpoint::NamedPipe(name) => {
                let client = ClientOptions::new().open(&name)?;
                Ok(UniversalTransport::NamedPipe(client))
            }
        }
    }

    /// Get the transport type of this connection
    ///
    /// Returns the actual transport mechanism being used.
    pub fn transport_type(&self) -> TransportType {
        match self {
            #[cfg(unix)]
            UniversalTransport::UnixSocket(_) => TransportType::UnixFilesystem,
            #[cfg(windows)]
            UniversalTransport::NamedPipe(_) => TransportType::NamedPipe,
            UniversalTransport::Tcp(_) => TransportType::Tcp,
            UniversalTransport::InProcess(_) => TransportType::InProcess,
        }
    }
}

// Implement AsyncRead for UniversalTransport
impl AsyncRead for UniversalTransport {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<IoResult<()>> {
        match &mut *self {
            #[cfg(unix)]
            UniversalTransport::UnixSocket(stream) => std::pin::Pin::new(stream).poll_read(cx, buf),
            #[cfg(windows)]
            UniversalTransport::NamedPipe(pipe) => std::pin::Pin::new(pipe).poll_read(cx, buf),
            UniversalTransport::Tcp(stream) => std::pin::Pin::new(stream).poll_read(cx, buf),
            UniversalTransport::InProcess(_) => {
                // In-process would implement actual channel reading
                std::task::Poll::Ready(Ok(()))
            }
        }
    }
}

// Implement AsyncWrite for UniversalTransport
impl AsyncWrite for UniversalTransport {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<IoResult<usize>> {
        match &mut *self {
            #[cfg(unix)]
            UniversalTransport::UnixSocket(stream) => {
                std::pin::Pin::new(stream).poll_write(cx, buf)
            }
            #[cfg(windows)]
            UniversalTransport::NamedPipe(pipe) => std::pin::Pin::new(pipe).poll_write(cx, buf),
            UniversalTransport::Tcp(stream) => std::pin::Pin::new(stream).poll_write(cx, buf),
            UniversalTransport::InProcess(_) => {
                // In-process would implement actual channel writing
                std::task::Poll::Ready(Ok(buf.len()))
            }
        }
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<IoResult<()>> {
        match &mut *self {
            #[cfg(unix)]
            UniversalTransport::UnixSocket(stream) => std::pin::Pin::new(stream).poll_flush(cx),
            #[cfg(windows)]
            UniversalTransport::NamedPipe(pipe) => std::pin::Pin::new(pipe).poll_flush(cx),
            UniversalTransport::Tcp(stream) => std::pin::Pin::new(stream).poll_flush(cx),
            UniversalTransport::InProcess(_) => std::task::Poll::Ready(Ok(())),
        }
    }

    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<IoResult<()>> {
        match &mut *self {
            #[cfg(unix)]
            UniversalTransport::UnixSocket(stream) => std::pin::Pin::new(stream).poll_shutdown(cx),
            #[cfg(windows)]
            UniversalTransport::NamedPipe(pipe) => std::pin::Pin::new(pipe).poll_shutdown(cx),
            UniversalTransport::Tcp(stream) => std::pin::Pin::new(stream).poll_shutdown(cx),
            UniversalTransport::InProcess(_) => std::task::Poll::Ready(Ok(())),
        }
    }
}

#[cfg(test)]
mod tests {
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
        let mut config = TransportConfig::default();
        config.preferred_transport = Some(TransportType::Tcp);
        config.enable_fallback = false;
        config.timeout_ms = 10000;
        config.socket_base_dir = Some(PathBuf::from("/tmp/custom"));

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
        let mut config = TransportConfig::default();
        config.preferred_transport = Some(TransportType::Tcp);

        let hierarchy = UniversalTransport::get_transport_hierarchy(&config);
        assert_eq!(hierarchy.len(), 2);
        assert_eq!(hierarchy[0], TransportType::Tcp);
        assert_eq!(hierarchy[1], TransportType::Tcp);
    }

    #[test]
    fn test_transport_hierarchy_with_preference_unix() {
        #[cfg(unix)]
        {
            let mut config = TransportConfig::default();
            config.preferred_transport = Some(TransportType::UnixFilesystem);

            let hierarchy = UniversalTransport::get_transport_hierarchy(&config);
            assert_eq!(hierarchy.len(), 2);
            assert_eq!(hierarchy[0], TransportType::UnixFilesystem);
            assert_eq!(hierarchy[1], TransportType::Tcp);
        }
    }

    #[test]
    fn test_transport_hierarchy_no_fallback() {
        let mut config = TransportConfig::default();
        config.preferred_transport = Some(TransportType::Tcp);
        config.enable_fallback = false;

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
        let mut config = TransportConfig::default();
        config.socket_base_dir = Some(PathBuf::from("/tmp/custom_sockets"));

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
            assert!(
                error.kind() == io::ErrorKind::NotFound || error.kind() == io::ErrorKind::TimedOut
            );
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
            UniversalTransport::try_connect("test_service", TransportType::InProcess, &config)
                .await;

        assert!(result.is_ok());
        let transport = result.unwrap();
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
}
