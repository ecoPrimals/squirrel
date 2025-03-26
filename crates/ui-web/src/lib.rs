//! Squirrel Web UI implementation.
//!
//! This crate contains the web-based user interface for the Squirrel system.
//! It provides a browser-based interface that communicates with the Squirrel Web API.

pub mod api;
pub mod components;
pub mod assets;

/// UI crate version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// UI crate features
#[cfg(feature = "wasm")]
pub const FEATURES: &[&str] = &["wasm"];
#[cfg(not(feature = "wasm"))]
pub const FEATURES: &[&str] = &[];

/// Build info
pub mod build_info {
    /// Build timestamp
    pub const BUILD_TIMESTAMP: &str = env!("CARGO_PKG_VERSION");
    
    /// Git commit hash
    pub const GIT_COMMIT: Option<&str> = option_env!("GIT_COMMIT");
    
    /// Build profile
    #[cfg(debug_assertions)]
    pub const BUILD_PROFILE: &str = "debug";
    #[cfg(not(debug_assertions))]
    pub const BUILD_PROFILE: &str = "release";
}

/// Initialize the UI
pub fn init() {
    // Initialize the UI
    #[cfg(feature = "wasm")]
    {
        // WASM-specific initialization
        wasm_init();
    }
    
    #[cfg(not(feature = "wasm"))]
    {
        // Non-WASM initialization
        std::env::set_var("SQUIRREL_UI_WEB_VERSION", VERSION);
    }
}

/// Initialize the UI for WASM
#[cfg(feature = "wasm")]
fn wasm_init() {
    // WASM-specific initialization
    // This would be implemented with wasm-bindgen
}

/// Web server integration
pub mod web_integration {
    use super::assets::{create_asset_manager, AssetManager};
    use std::sync::{Arc, Mutex};
    
    /// UI server handler
    #[derive(Clone)]
    pub struct UiHandler {
        /// Asset manager
        asset_manager: Arc<Mutex<AssetManager>>,
    }
    
    impl UiHandler {
        /// Create a new UI handler
        pub fn new() -> Self {
            let mut asset_manager = create_asset_manager();
            
            // Preload assets (this is optional but can improve performance)
            if let Err(e) = asset_manager.preload_all() {
                eprintln!("Error preloading assets: {}", e);
            }
            
            Self {
                asset_manager: Arc::new(Mutex::new(asset_manager)),
            }
        }
        
        /// Get an asset by path
        pub fn get_asset(&self, path: &str) -> Option<Vec<u8>> {
            let mut asset_manager = self.asset_manager.lock().unwrap();
            asset_manager.get_asset(path).map(|data| data.to_vec())
        }
        
        /// Get the MIME type for an asset
        pub fn get_mime_type(&self, path: &str) -> &'static str {
            let asset_manager = self.asset_manager.lock().unwrap();
            asset_manager.get_mime_type(path)
        }
        
        /// List all available assets
        pub fn list_assets(&self) -> Vec<String> {
            let asset_manager = self.asset_manager.lock().unwrap();
            asset_manager.list_assets()
        }
    }
    
    impl Default for UiHandler {
        fn default() -> Self {
            Self::new()
        }
    }
} 