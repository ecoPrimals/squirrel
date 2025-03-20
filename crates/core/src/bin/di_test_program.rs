//! Test program for Dependency Injection pattern

// Remove unused imports at the top level
use crate::app::AppInterface;
use crate::mcp::MCPInterface;

/// Error types
mod error {
    use std::error::Error;
    use std::fmt;

    #[derive(Debug)]
    pub enum SquirrelError {
        AppInitialization(AppInitializationError),
        AppOperation(AppOperationError),
        Generic(String),
    }

    impl Error for SquirrelError {}

    impl fmt::Display for SquirrelError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                SquirrelError::AppInitialization(e) => write!(f, "App initialization error: {}", e),
                SquirrelError::AppOperation(e) => write!(f, "App operation error: {}", e),
                SquirrelError::Generic(msg) => write!(f, "Error: {}", msg),
            }
        }
    }

    impl From<AppInitializationError> for SquirrelError {
        fn from(err: AppInitializationError) -> Self {
            SquirrelError::AppInitialization(err)
        }
    }

    impl From<AppOperationError> for SquirrelError {
        fn from(err: AppOperationError) -> Self {
            SquirrelError::AppOperation(err)
        }
    }

    #[derive(Debug)]
    pub enum AppInitializationError {
        AlreadyInitialized,
        InvalidConfiguration(String),
    }

    impl Error for AppInitializationError {}

    impl fmt::Display for AppInitializationError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                AppInitializationError::AlreadyInitialized => {
                    write!(f, "Application already initialized")
                }
                AppInitializationError::InvalidConfiguration(msg) => {
                    write!(f, "Invalid configuration: {}", msg)
                }
            }
        }
    }

    #[derive(Debug)]
    pub enum AppOperationError {
        NotInitialized,
        UnsupportedOperation(String),
    }

    impl Error for AppOperationError {}

    impl fmt::Display for AppOperationError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                AppOperationError::NotInitialized => {
                    write!(f, "Application not initialized")
                }
                AppOperationError::UnsupportedOperation(msg) => {
                    write!(f, "Unsupported operation: {}", msg)
                }
            }
        }
    }
}

/// App module
mod app {
    use super::error::{AppInitializationError, AppOperationError, SquirrelError};
    use std::sync::{Arc, Mutex, RwLock};

    #[derive(Debug, Clone)]
    pub struct AppConfig {
        pub name: String,
        pub version: String,
    }

    impl Default for AppConfig {
        fn default() -> Self {
            Self {
                name: "Test App".to_string(),
                version: "1.0.0".to_string(),
            }
        }
    }

    #[derive(Debug)]
    pub struct AppState {
        pub initialized: bool,
        pub config: AppConfig,
    }

    impl AppState {
        pub fn new(config: AppConfig) -> Self {
            Self {
                initialized: false,
                config,
            }
        }
    }

    pub struct App {
        state: RwLock<AppState>,
    }

    impl App {
        pub fn new(config: AppConfig) -> Self {
            Self {
                state: RwLock::new(AppState::new(config)),
            }
        }

        pub fn initialize(&self) -> Result<(), AppInitializationError> {
            let mut state = self.state.write().unwrap();
            if state.initialized {
                return Err(AppInitializationError::AlreadyInitialized);
            }
            
            state.initialized = true;
            Ok(())
        }

        pub fn is_initialized(&self) -> bool {
            self.state.read().unwrap().initialized
        }
        
        pub fn get_config(&self) -> Result<AppConfig, AppOperationError> {
            let state = self.state.read().unwrap();
            if !state.initialized {
                return Err(AppOperationError::NotInitialized);
            }
            
            Ok(state.config.clone())
        }
    }

    pub trait AppInterface {
        fn initialize(&self) -> Result<(), SquirrelError>;
        fn is_initialized(&self) -> bool;
        fn get_config(&self) -> Result<AppConfig, SquirrelError>;
    }

    pub struct AppAdapter {
        app: Arc<App>,
        init_mutex: Mutex<()>,
    }

    impl AppAdapter {
        pub fn new(config: AppConfig) -> Self {
            Self {
                app: Arc::new(App::new(config)),
                init_mutex: Mutex::new(()),
            }
        }
        
        pub fn new_initialized(config: AppConfig) -> Result<Self, SquirrelError> {
            let adapter = Self::new(config);
            adapter.initialize()?;
            Ok(adapter)
        }
    }

    impl AppInterface for AppAdapter {
        fn initialize(&self) -> Result<(), SquirrelError> {
            let _lock = self.init_mutex.lock().unwrap();
            self.app.initialize().map_err(Into::into)
        }
        
        fn is_initialized(&self) -> bool {
            self.app.is_initialized()
        }
        
        fn get_config(&self) -> Result<AppConfig, SquirrelError> {
            self.app.get_config().map_err(Into::into)
        }
    }
}

/// MCP module
mod mcp {
    use super::error::{AppInitializationError, AppOperationError, SquirrelError};
    use std::sync::{Arc, Mutex, RwLock};

    #[derive(Debug, Clone)]
    pub struct MCPConfig {
        pub version: String,
        pub max_message_size: u64,
        pub timeout_ms: u64,
        pub encryption_enabled: bool,
    }

    impl Default for MCPConfig {
        fn default() -> Self {
            Self {
                version: "1.0".to_string(),
                max_message_size: 1024 * 1024, // 1MB
                timeout_ms: 5000, // 5 seconds
                encryption_enabled: true,
            }
        }
    }

    #[derive(Debug)]
    pub struct MCPState {
        pub initialized: bool,
        pub config: MCPConfig,
    }

    impl MCPState {
        pub fn new(config: MCPConfig) -> Self {
            Self {
                initialized: false,
                config,
            }
        }
    }

    pub struct MCP {
        state: RwLock<MCPState>,
    }

    impl MCP {
        pub fn new(config: MCPConfig) -> Self {
            Self {
                state: RwLock::new(MCPState::new(config)),
            }
        }

        pub fn initialize(&self) -> Result<(), AppInitializationError> {
            let mut state = self.state.write().unwrap();
            if state.initialized {
                return Err(AppInitializationError::AlreadyInitialized);
            }
            
            state.initialized = true;
            Ok(())
        }

        pub fn is_initialized(&self) -> bool {
            self.state.read().unwrap().initialized
        }
        
        pub fn get_config(&self) -> Result<MCPConfig, AppOperationError> {
            let state = self.state.read().unwrap();
            if !state.initialized {
                return Err(AppOperationError::NotInitialized);
            }
            
            Ok(state.config.clone())
        }

        pub fn send_message(&self, message: &str) -> Result<String, AppOperationError> {
            let state = self.state.read().unwrap();
            if !state.initialized {
                return Err(AppOperationError::NotInitialized);
            }
            
            Ok(format!("Processed: {}", message))
        }
    }

    pub trait MCPInterface {
        fn initialize(&self) -> Result<(), SquirrelError>;
        fn is_initialized(&self) -> bool;
        fn get_config(&self) -> Result<MCPConfig, SquirrelError>;
        fn send_message(&self, message: &str) -> Result<String, SquirrelError>;
    }

    pub struct MCPAdapter {
        mcp: Arc<MCP>,
        init_mutex: Mutex<()>,
    }

    impl MCPAdapter {
        pub fn new(config: MCPConfig) -> Self {
            Self {
                mcp: Arc::new(MCP::new(config)),
                init_mutex: Mutex::new(()),
            }
        }
        
        pub fn new_initialized(config: MCPConfig) -> Result<Self, SquirrelError> {
            let adapter = Self::new(config);
            adapter.initialize()?;
            Ok(adapter)
        }
    }

    impl MCPInterface for MCPAdapter {
        fn initialize(&self) -> Result<(), SquirrelError> {
            let _lock = self.init_mutex.lock().unwrap();
            self.mcp.initialize().map_err(Into::into)
        }
        
        fn is_initialized(&self) -> bool {
            self.mcp.is_initialized()
        }
        
        fn get_config(&self) -> Result<MCPConfig, SquirrelError> {
            self.mcp.get_config().map_err(Into::into)
        }
        
        fn send_message(&self, message: &str) -> Result<String, SquirrelError> {
            self.mcp.send_message(message).map_err(Into::into)
        }
    }
}

fn main() {
    println!("Testing Dependency Injection pattern");
    
    // Test AppAdapter
    println!("\nTesting AppAdapter...");
    let app_config = app::AppConfig::default();
    let app_adapter = app::AppAdapter::new(app_config);
    
    println!("App initialized: {}", app_adapter.is_initialized());
    match app_adapter.initialize() {
        Ok(_) => println!("App initialized successfully"),
        Err(e) => println!("Failed to initialize app: {}", e),
    }
    println!("App initialized: {}", app_adapter.is_initialized());
    
    match app_adapter.get_config() {
        Ok(config) => println!("App config: {} v{}", config.name, config.version),
        Err(e) => println!("Failed to get app config: {}", e),
    }
    
    // Test MCPAdapter
    println!("\nTesting MCPAdapter...");
    let mcp_config = mcp::MCPConfig::default();
    let mcp_adapter = mcp::MCPAdapter::new(mcp_config);
    
    println!("MCP initialized: {}", mcp_adapter.is_initialized());
    match mcp_adapter.initialize() {
        Ok(_) => println!("MCP initialized successfully"),
        Err(e) => println!("Failed to initialize MCP: {}", e),
    }
    println!("MCP initialized: {}", mcp_adapter.is_initialized());
    
    match mcp_adapter.get_config() {
        Ok(config) => println!("MCP config: v{}, timeout: {}ms", config.version, config.timeout_ms),
        Err(e) => println!("Failed to get MCP config: {}", e),
    }
    
    match mcp_adapter.send_message("Hello, MCP!") {
        Ok(response) => println!("MCP response: {}", response),
        Err(e) => println!("Failed to send message: {}", e),
    }
    
    // Test already initialized error
    match mcp_adapter.initialize() {
        Ok(_) => println!("MCP re-initialized successfully (unexpected)"),
        Err(e) => println!("MCP re-initialization failed as expected: {}", e),
    }
    
    // Test factory method
    match mcp::MCPAdapter::new_initialized(mcp::MCPConfig::default()) {
        Ok(adapter) => println!("Factory created initialized adapter: {}", adapter.is_initialized()),
        Err(e) => println!("Factory failed: {}", e),
    }
    
    println!("\nDependency Injection test completed successfully!");
} 