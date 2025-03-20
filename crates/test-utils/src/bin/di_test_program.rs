//! Test program for dependency injection.

use squirrel_app::{Core, core::AppConfig};

fn main() {
    // Create a simple app with our new structure
    let config = AppConfig::default();
    let adapter = Core::new(config);
    
    println!("Di-tests mode active");
    println!("Core initialized: {}", adapter.is_initialized());
} 