#![no_main]

use std::io::Write;
use std::sync::Arc;
use tempfile::NamedTempFile;

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

// Entry point for the fuzzer
#[no_mangle]
pub extern "C" fn LLVMFuzzerTestOneInput(data: &[u8]) -> i32 {
    // Only fuzz if we have enough data to be meaningful (at least 64 bytes)
    if data.len() < 64 {
        return 0;
    }

    // Create a temporary library file with the fuzzer data
    let temp_file = match create_temp_library(data) {
        Ok(file) => file,
        Err(_) => return 0, // If we can't create the file, just skip this input
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

    // Return 0 to indicate successful fuzzing (even if the plugin failed to load)
    0
}

#[cfg(test)]
mod tests {
    /// Test that our fuzzer can handle some basic inputs without crashing
    #[test]
    fn test_fuzzer_with_empty_input() {
        // Empty input
        let data = Vec::new();
        let result = super::LLVMFuzzerTestOneInput(&data);
        assert_eq!(result, 0);
        
        // Small input should be rejected early
        let data = vec![0u8; 32];
        let result = super::LLVMFuzzerTestOneInput(&data);
        assert_eq!(result, 0);
        
        // Larger input should be processed
        let data = vec![0u8; 128];
        let result = super::LLVMFuzzerTestOneInput(&data);
        assert_eq!(result, 0);
    }
} 