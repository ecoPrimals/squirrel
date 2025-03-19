//! Entry point for the Squirrel Core binary.

#[cfg(not(feature = "di-tests"))]
use squirrel_core::{Core, MCP};

#[cfg(not(feature = "di-tests"))]
use squirrel_core::error::Result;

#[cfg(feature = "di-tests")]
use squirrel_core::{Core, app::AppConfig};

#[cfg(not(feature = "di-tests"))]
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the core
    let core = Core::new(squirrel_core::app::AppConfig::default());
    let _mcp = MCP::new(squirrel_core::mcp::MCPConfig::default());

    println!("Core initialized successfully");
    Ok(())
}

#[cfg(feature = "di-tests")]
fn main() {
    // Create a simple app with our new structure
    let config = AppConfig::default();
    let adapter = Core::new(config);
    
    println!("Di-tests mode active");
    println!("Core initialized: {}", adapter.is_initialized());
} 