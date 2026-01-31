//! Universal Transport Abstraction
//!
//! This module provides a universal transport abstraction that works across
//! all platforms without hardcoded platform-specific code paths.
//!
//! ## Philosophy: Universal & Agnostic
//!
//! Instead of:
//! ```ignore
//! #[cfg(unix)]
//! use_unix_socket();
//!
//! #[cfg(windows)]
//! use_named_pipe();
//!
//! #[cfg(target_os = "macos")]
//! use_xpc();
//! ```
//!
//! We use:
//! ```rust,no_run
//! use universal_patterns::transport::{UniversalTransport, UniversalListener};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Client: Runtime detection, automatic platform selection
//! let transport = UniversalTransport::connect("service_name", None).await?;
//!
//! // Server: Universal bind and accept
//! let listener = UniversalListener::bind("service_name", None).await?;
//! let (stream, addr) = listener.accept().await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Transport Hierarchy
//!
//! The universal transport automatically selects the best transport for the
//! current platform with automatic fallback:
//!
//! 1. **Unix Domain Sockets** (Linux, macOS, BSD)
//!    - Abstract namespace sockets (Linux)
//!    - Filesystem sockets (all Unix)
//!
//! 2. **Named Pipes** (Windows)
//!    - `\\.\pipe\name` format
//!
//! 3. **XPC** (macOS system services)
//!    - Only for system-level services
//!
//! 4. **TCP** (Universal fallback)
//!    - localhost:port
//!    - Works everywhere
//!
//! 5. **In-Process** (Testing, embedded)
//!    - Direct function calls
//!    - Zero overhead

use std::io::{self, Result as IoResult};
use std::path::PathBuf;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::{TcpListener, TcpStream, UnixListener, UnixStream};

#[cfg(windows)]
use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeClient, ServerOptions};

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

/// Configuration for transport connection
///
/// Provides hints for transport selection and connection behavior.
#[derive(Debug, Clone)]
pub struct TransportConfig {
    /// Preferred transport type (None = automatic)
    pub preferred_transport: Option<TransportType>,

    /// Enable automatic fallback on connection failure
    pub enable_fallback: bool,

    /// Connection timeout in milliseconds
    pub timeout_ms: u64,

    /// Base directory for filesystem sockets
    pub socket_base_dir: Option<PathBuf>,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            preferred_transport: None,
            enable_fallback: true,
            timeout_ms: 5000,
            socket_base_dir: None,
        }
    }
}

/// Transport type enumeration
///
/// Used for explicit transport selection or preference specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportType {
    /// Unix domain socket (abstract namespace on Linux)
    UnixAbstract,

    /// Unix domain socket (filesystem)
    UnixFilesystem,

    /// Named pipe (Windows)
    NamedPipe,

    /// TCP connection
    Tcp,

    /// In-process channel
    InProcess,
}

/// In-process transport for testing and embedded scenarios
///
/// Provides zero-overhead communication within the same process.
#[derive(Debug)]
pub struct InProcessTransport {
    // Placeholder for in-process channel implementation
    // In a real implementation, this would use tokio::sync::mpsc or similar
    _marker: std::marker::PhantomData<()>,
}

impl UniversalTransport {
    /// Connect to a service using automatic transport selection
    ///
    /// Automatically selects the best transport for the current platform
    /// with fallback to TCP if preferred transport fails.
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
    /// ```rust,no_run
    /// use universal_patterns::transport::UniversalTransport;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Automatic platform detection and connection
    /// let transport = UniversalTransport::connect("squirrel", None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect(service_name: &str, config: Option<TransportConfig>) -> IoResult<Self> {
        let config = config.unwrap_or_default();

        // Determine transport hierarchy based on platform
        let transport_order = Self::get_transport_hierarchy(&config);

        let mut last_error = None;

        for transport_type in transport_order {
            match Self::try_connect(service_name, transport_type, &config).await {
                Ok(transport) => {
                    tracing::info!("Connected to {} using {:?}", service_name, transport_type);
                    return Ok(transport);
                }
                Err(e) => {
                    tracing::debug!(
                        "Failed to connect to {} using {:?}: {}",
                        service_name,
                        transport_type,
                        e
                    );
                    last_error = Some(e);

                    if !config.enable_fallback {
                        break;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("Failed to connect to service: {}", service_name),
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
    fn get_transport_hierarchy(config: &TransportConfig) -> Vec<TransportType> {
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
                Ok(UniversalTransport::InProcess(InProcessTransport {
                    _marker: std::marker::PhantomData,
                }))
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
    fn get_socket_path(service_name: &str, config: &TransportConfig) -> PathBuf {
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

    #[test]
    fn test_transport_config_default() {
        let config = TransportConfig::default();
        assert!(config.enable_fallback);
        assert_eq!(config.timeout_ms, 5000);
        assert!(config.preferred_transport.is_none());
    }

    #[test]
    fn test_transport_hierarchy_linux() {
        #[cfg(target_os = "linux")]
        {
            let config = TransportConfig::default();
            let hierarchy = UniversalTransport::get_transport_hierarchy(&config);
            assert_eq!(hierarchy[0], TransportType::UnixAbstract);
            assert_eq!(hierarchy[1], TransportType::UnixFilesystem);
            assert_eq!(hierarchy[2], TransportType::Tcp);
        }
    }

    #[test]
    fn test_transport_hierarchy_with_preference() {
        let mut config = TransportConfig::default();
        config.preferred_transport = Some(TransportType::Tcp);

        let hierarchy = UniversalTransport::get_transport_hierarchy(&config);
        assert_eq!(hierarchy[0], TransportType::Tcp);
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
    fn test_socket_path_generation() {
        let config = TransportConfig::default();
        let path = UniversalTransport::get_socket_path("test_service", &config);

        assert!(path.to_string_lossy().contains("test_service.sock"));
    }
}

// ============================================================================
// SERVER-SIDE: UniversalListener
// ============================================================================

/// Universal listener abstraction for server-side transport
///
/// Provides platform-appropriate server binding with automatic
/// transport selection and graceful fallback.
///
/// ## Usage
///
/// ```rust,no_run
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

/// Configuration for server listener
///
/// Provides configuration options for binding server sockets.
#[derive(Debug, Clone)]
pub struct ListenerConfig {
    /// Preferred transport type (None = automatic)
    pub preferred_transport: Option<TransportType>,

    /// Enable automatic fallback on bind failure
    pub enable_fallback: bool,

    /// Base directory for filesystem sockets
    pub socket_base_dir: Option<PathBuf>,

    /// Backlog size for accept queue
    pub backlog: Option<u32>,

    /// Unix socket permissions (octal, e.g., 0o666)
    #[cfg(unix)]
    pub unix_permissions: Option<u32>,
}

impl Default for ListenerConfig {
    fn default() -> Self {
        Self {
            preferred_transport: None,
            enable_fallback: true,
            socket_base_dir: None,
            backlog: Some(128),
            #[cfg(unix)]
            unix_permissions: Some(0o666),
        }
    }
}

/// Remote address information
///
/// Represents the remote peer address for an accepted connection.
#[derive(Debug, Clone)]
pub enum RemoteAddr {
    /// Unix socket (path or abstract)
    #[cfg(unix)]
    Unix(Option<std::os::unix::net::SocketAddr>),

    /// Named pipe (Windows)
    #[cfg(windows)]
    NamedPipe(String),

    /// TCP address
    Tcp(std::net::SocketAddr),

    /// In-process
    InProcess,
}

impl UniversalListener {
    /// Bind a server listener using automatic transport selection
    ///
    /// Automatically selects the best transport for the current platform
    /// with fallback to TCP if preferred transport fails.
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
    /// ```rust,no_run
    /// use universal_patterns::transport::UniversalListener;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Automatic platform detection and binding
    /// let listener = UniversalListener::bind("squirrel", None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn bind(service_name: &str, config: Option<ListenerConfig>) -> IoResult<Self> {
        let config = config.unwrap_or_default();

        // Determine transport hierarchy based on platform
        let transport_order = Self::get_transport_hierarchy(&config);

        let mut last_error = None;

        for transport_type in transport_order {
            match Self::try_bind(service_name, transport_type, &config).await {
                Ok(listener) => {
                    tracing::info!("Bound {} server using {:?}", service_name, transport_type);
                    return Ok(listener);
                }
                Err(e) => {
                    tracing::debug!(
                        "Failed to bind {} using {:?}: {}",
                        service_name,
                        transport_type,
                        e
                    );
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
                format!("Failed to bind service: {}", service_name),
            )
        }))
    }

    /// Get transport hierarchy for the current platform
    ///
    /// Returns transport types in order of preference for server binding.
    fn get_transport_hierarchy(config: &ListenerConfig) -> Vec<TransportType> {
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
    /// ```rust,no_run
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
                let next_server = ServerOptions::new().create(pipe_name)?;

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
    fn get_socket_path(service_name: &str, config: &ListenerConfig) -> PathBuf {
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
mod listener_tests {
    use super::*;

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
    fn test_listener_socket_path() {
        let config = ListenerConfig::default();
        let path = UniversalListener::get_socket_path("test_service", &config);

        assert!(path.to_string_lossy().contains("test_service.sock"));
    }
}
