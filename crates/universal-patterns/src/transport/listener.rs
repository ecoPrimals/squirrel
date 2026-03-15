// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Universal listener implementation for server-side transport

use std::io::{self, Result as IoResult};
use std::path::PathBuf;
use tokio::net::{TcpListener, UnixListener};

#[cfg(windows)]
use tokio::net::windows::named_pipe::{ClientOptions, ServerOptions};

use crate::transport::client::UniversalTransport;
use crate::transport::discovery::write_tcp_discovery_file;
use crate::transport::types::{ListenerConfig, RemoteAddr, TransportType};

/// Universal listener abstraction for server-side transport
///
/// Provides platform-appropriate server binding with automatic
/// transport selection and graceful fallback.
///
/// ## Usage
///
/// ```ignore,no_run
/// use universal_patterns::transport::UniversalListener;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Bind to platform-appropriate transport
/// let listener = UniversalListener::bind("my_service", None).await?;
///
/// // Accept connections
/// loop {
///     let (stream, addr) = listener.accept().await?;
///     tokio::spawn(async move {
///         // Handle connection
///     });
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub enum UniversalListener {
    /// Unix domain socket listener (Linux, macOS, BSD)
    #[cfg(unix)]
    UnixSocket(UnixListener),

    /// Named pipe server (Windows)
    #[cfg(windows)]
    NamedPipe {
        /// Pipe name for creating new instances
        pipe_name: String,
        /// First server instance (for accept)
        server: tokio::net::windows::named_pipe::NamedPipeServer,
    },

    /// TCP listener (universal fallback)
    Tcp(TcpListener),
}

impl UniversalListener {
    /// Bind a listener using automatic transport selection
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
    /// * `service_name` - Name of the service to bind
    /// * `config` - Optional listener configuration (uses defaults if None)
    ///
    /// # Returns
    ///
    /// A bound `UniversalListener` instance
    ///
    /// # Example
    ///
    /// ```ignore,no_run
    /// use universal_patterns::transport::UniversalListener;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Automatic platform detection and binding (isomorphic)
    /// let listener = UniversalListener::bind("squirrel", None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn bind(service_name: &str, config: Option<ListenerConfig>) -> IoResult<Self> {
        let config = config.unwrap_or_default();

        // Determine transport hierarchy based on platform
        let transport_order = Self::get_transport_hierarchy(&config);

        tracing::info!("🔌 Starting IPC server (isomorphic mode)...");
        tracing::info!("   Service: {}", service_name);

        let mut last_error = None;

        for transport_type in transport_order {
            tracing::info!("   Trying {:?}...", transport_type);

            match Self::try_bind(service_name, transport_type, &config).await {
                Ok(listener) => {
                    tracing::info!("✅ Listening on {:?}", transport_type);

                    // Write TCP discovery file when using TCP fallback
                    if let UniversalListener::Tcp(ref tcp_listener) = listener {
                        if let Ok(addr) = tcp_listener.local_addr() {
                            if let Err(e) = write_tcp_discovery_file(service_name, &addr) {
                                tracing::warn!("⚠️  Could not write TCP discovery file: {}", e);
                            } else {
                                tracing::info!("📁 TCP discovery file written");
                                tracing::info!(
                                    "   Status: READY ✅ (isomorphic TCP fallback active)"
                                );
                            }
                        }
                    } else {
                        tracing::info!("   Status: READY ✅");
                    }

                    return Ok(listener);
                }

                // DETECT: Platform constraint (expected, adapt)
                Err(e) if UniversalTransport::is_platform_constraint(&e) => {
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
                    last_error = Some(e);

                    if !config.enable_fallback {
                        break;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            io::Error::new(
                io::ErrorKind::AddrNotAvailable,
                format!(
                    "Failed to bind service: {} (all transports exhausted)",
                    service_name
                ),
            )
        }))
    }

    /// Get transport hierarchy for the current platform
    ///
    /// Returns transport types in order of preference for server binding.
    pub(crate) fn get_transport_hierarchy(config: &ListenerConfig) -> Vec<TransportType> {
        // If explicit preference, try that first
        if let Some(preferred) = config.preferred_transport {
            if config.enable_fallback {
                return vec![preferred, TransportType::Tcp];
            } else {
                return vec![preferred];
            }
        }

        // Platform-appropriate hierarchy (same as client)
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

    /// Try to bind using a specific transport type
    ///
    /// Attempts binding with the specified transport mechanism.
    async fn try_bind(
        service_name: &str,
        transport_type: TransportType,
        config: &ListenerConfig,
    ) -> IoResult<Self> {
        match transport_type {
            #[cfg(unix)]
            TransportType::UnixAbstract => {
                #[cfg(target_os = "linux")]
                {
                    // Abstract socket: starts with null byte
                    let path = format!("\0{}", service_name);
                    let listener = UnixListener::bind(path)?;
                    Ok(UniversalListener::UnixSocket(listener))
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

                // Remove existing socket file if present
                if socket_path.exists() {
                    std::fs::remove_file(&socket_path)?;
                }

                // Create parent directory if needed
                if let Some(parent) = socket_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }

                let listener = UnixListener::bind(&socket_path)?;

                // Set permissions if specified
                #[cfg(unix)]
                if let Some(perms) = config.unix_permissions {
                    use std::os::unix::fs::PermissionsExt;
                    let permissions = std::fs::Permissions::from_mode(perms);
                    std::fs::set_permissions(&socket_path, permissions)?;
                }

                Ok(UniversalListener::UnixSocket(listener))
            }

            #[cfg(windows)]
            TransportType::NamedPipe => {
                // Named pipe: \\.\pipe\name
                let pipe_name = format!(r"\\.\pipe\{}", service_name);

                let server = ServerOptions::new()
                    .first_pipe_instance(true)
                    .create(&pipe_name)?;

                Ok(UniversalListener::NamedPipe { pipe_name, server })
            }

            TransportType::Tcp => {
                // TCP fallback: bind to localhost with port from service registry
                let port = Self::get_tcp_port(service_name);
                let addr = format!("127.0.0.1:{}", port);

                let listener = TcpListener::bind(&addr).await?;

                Ok(UniversalListener::Tcp(listener))
            }

            TransportType::InProcess => {
                // In-process not supported for server-side
                Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "In-process transport not supported for server binding",
                ))
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

    /// Accept a new connection
    ///
    /// Waits for and accepts a new incoming connection.
    ///
    /// # Returns
    ///
    /// A tuple of (`UniversalTransport`, `RemoteAddr`) representing the
    /// accepted connection and remote peer address.
    ///
    /// # Example
    ///
    /// ```ignore,no_run
    /// use universal_patterns::transport::UniversalListener;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let listener = UniversalListener::bind("my_service", None).await?;
    ///
    /// loop {
    ///     let (stream, addr) = listener.accept().await?;
    ///     println!("Accepted connection from {:?}", addr);
    ///     // Handle stream...
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn accept(&self) -> IoResult<(UniversalTransport, RemoteAddr)> {
        match self {
            #[cfg(unix)]
            UniversalListener::UnixSocket(listener) => {
                let (stream, addr) = listener.accept().await?;
                Ok((
                    UniversalTransport::UnixSocket(stream),
                    RemoteAddr::Unix(
                        addr.as_pathname()
                            .and_then(|p| std::os::unix::net::SocketAddr::from_pathname(p).ok()),
                    ),
                ))
            }

            #[cfg(windows)]
            UniversalListener::NamedPipe { pipe_name, server } => {
                // Wait for client connection
                server.connect().await?;

                // Create a new server instance for the next connection
                // (Windows named pipes require a new server per connection)
                let _next_server = ServerOptions::new().create(pipe_name)?;

                // Return the connected pipe as a client (for consistency with transport API)
                let client = ClientOptions::new().open(pipe_name)?;

                Ok((
                    UniversalTransport::NamedPipe(client),
                    RemoteAddr::NamedPipe(pipe_name.clone()),
                ))
            }

            UniversalListener::Tcp(listener) => {
                let (stream, addr) = listener.accept().await?;
                Ok((UniversalTransport::Tcp(stream), RemoteAddr::Tcp(addr)))
            }
        }
    }

    /// Get filesystem socket path for a service
    pub(crate) fn get_socket_path(service_name: &str, config: &ListenerConfig) -> PathBuf {
        use crate::federation::cross_platform::CrossPlatform;

        let base_dir = config
            .socket_base_dir
            .clone()
            .unwrap_or_else(|| CrossPlatform::get_runtime_dir("squirrel"));

        base_dir.join(format!("{}.sock", service_name))
    }

    /// Get TCP port for a service
    fn get_tcp_port(service_name: &str) -> u16 {
        use universal_constants::network::get_service_port;
        get_service_port(service_name)
    }

    /// Get the local address this listener is bound to
    ///
    /// Returns the local address information for this listener.
    pub fn local_addr(&self) -> IoResult<String> {
        match self {
            #[cfg(unix)]
            UniversalListener::UnixSocket(listener) => {
                let addr = listener.local_addr()?;
                Ok(format!("{:?}", addr))
            }

            #[cfg(windows)]
            UniversalListener::NamedPipe { pipe_name, .. } => Ok(pipe_name.clone()),

            UniversalListener::Tcp(listener) => {
                let addr = listener.local_addr()?;
                Ok(format!("{}", addr))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_listener_config_default() {
        let config = ListenerConfig::default();
        assert!(config.enable_fallback);
        assert_eq!(config.backlog, Some(128));
        #[cfg(unix)]
        assert_eq!(config.unix_permissions, Some(0o666));
    }

    #[test]
    fn test_listener_transport_hierarchy() {
        #[cfg(target_os = "linux")]
        {
            let config = ListenerConfig::default();
            let hierarchy = UniversalListener::get_transport_hierarchy(&config);
            assert_eq!(hierarchy[0], TransportType::UnixAbstract);
            assert_eq!(hierarchy[1], TransportType::UnixFilesystem);
            assert_eq!(hierarchy[2], TransportType::Tcp);
        }
    }

    #[test]
    fn test_listener_hierarchy_with_preference() {
        let mut config = ListenerConfig::default();
        config.preferred_transport = Some(TransportType::Tcp);

        let hierarchy = UniversalListener::get_transport_hierarchy(&config);
        assert_eq!(hierarchy[0], TransportType::Tcp);
    }

    #[test]
    fn test_listener_hierarchy_with_preference_and_fallback() {
        let mut config = ListenerConfig::default();
        config.preferred_transport = Some(TransportType::Tcp);
        config.enable_fallback = true;

        let hierarchy = UniversalListener::get_transport_hierarchy(&config);
        assert_eq!(hierarchy.len(), 2);
        assert_eq!(hierarchy[0], TransportType::Tcp);
        assert_eq!(hierarchy[1], TransportType::Tcp);
    }

    #[test]
    fn test_listener_hierarchy_no_fallback() {
        let mut config = ListenerConfig::default();
        config.enable_fallback = false;
        config.preferred_transport = Some(TransportType::UnixFilesystem);

        let hierarchy = UniversalListener::get_transport_hierarchy(&config);
        assert_eq!(hierarchy.len(), 1);
        assert_eq!(hierarchy[0], TransportType::UnixFilesystem);
    }

    #[test]
    fn test_listener_socket_path() {
        let config = ListenerConfig::default();
        let path = UniversalListener::get_socket_path("test_service", &config);

        assert!(path.to_string_lossy().contains("test_service.sock"));
    }

    #[test]
    fn test_listener_socket_path_with_custom_base_dir() {
        let mut config = ListenerConfig::default();
        config.socket_base_dir = Some(PathBuf::from("/tmp/custom_sockets"));

        let path = UniversalListener::get_socket_path("my_service", &config);

        assert!(path.to_string_lossy().contains("my_service.sock"));
        assert!(path.to_string_lossy().contains("custom_sockets"));
    }

    #[tokio::test]
    async fn test_listener_bind_tcp_and_accept() {
        // Use TCP explicitly for reliable cross-platform binding
        let mut config = ListenerConfig::default();
        config.preferred_transport = Some(TransportType::Tcp);

        let listener = UniversalListener::bind("test_bind_accept", Some(config))
            .await
            .expect("bind should succeed");

        let addr = listener.local_addr().expect("local_addr should succeed");
        assert!(!addr.is_empty());
        assert!(addr.contains("127.0.0.1") || addr.contains("localhost"));

        // Spawn a task to connect
        let connect_addr = addr.clone();
        let connect_handle =
            tokio::spawn(async move { tokio::net::TcpStream::connect(connect_addr).await });

        // Accept the connection
        let accept_result = listener.accept().await;
        assert!(
            accept_result.is_ok(),
            "accept should succeed: {:?}",
            accept_result
        );

        let (_transport, remote_addr) = accept_result.unwrap();
        assert!(matches!(remote_addr, RemoteAddr::Tcp(_)));

        // Clean up - connect_handle may have failed if we parsed addr wrong, that's ok
        let _ = connect_handle.await;
    }

    #[tokio::test]
    async fn test_listener_bind_with_preferred_tcp() {
        let mut config = ListenerConfig::default();
        config.preferred_transport = Some(TransportType::Tcp);

        let result = UniversalListener::bind("test_tcp_bind", Some(config)).await;
        assert!(result.is_ok());

        let listener = result.unwrap();
        let addr_str = listener.local_addr().unwrap();
        assert!(addr_str.contains(':'));
    }
}
