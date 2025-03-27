#![no_main]

use std::io::Write;
use std::sync::Arc;
use tempfile::NamedTempFile;
use libfuzzer_sys::fuzz_target;

// Import the necessary parts of the plugin system
use squirrel_plugins::discovery::DefaultPluginDiscovery;
use squirrel_plugins::discovery::PluginDiscovery;

/// Create a temporary file with the fuzzer-generated data
fn create_temp_library(data: &[u8]) -> std::io::Result<NamedTempFile> {
    let mut file = NamedTempFile::new()?;
    file.write_all(data)?;
    file.flush()?;
    Ok(file)
}

fuzz_target!(|data: &[u8]| {
    // Only fuzz if we have enough data to be meaningful (at least 64 bytes)
    if data.len() < 64 {
        return;
    }

    // Create a temporary library file with the fuzzer data
    let temp_file = match create_temp_library(data) {
        Ok(file) => file,
        Err(_) => return, // If we can't create the file, just skip this input
    };

    // Get the path to the temporary file
    let path = temp_file.path().to_path_buf();

    // Create a runtime for async operations
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to build runtime");
    
    // Initialize the plugin discovery system
    let discovery = Arc::new(DefaultPluginDiscovery::default());

    // Attempt to load the plugin (this is what we're testing)
    let _result = rt.block_on(async {
        let _plugins = discovery.discover_plugins(path.parent().unwrap()).await;
        // We don't care about the result, we just care that it doesn't crash
    });
}); 