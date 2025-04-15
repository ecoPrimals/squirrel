#![cfg(test)]

use std::{
    io::{BufRead, BufReader, Write},
    process::{Command as StdCommand, Stdio},
    thread,
    time::{Duration, Instant},
    path::{Path, PathBuf},
    env,
};
use tempfile::{Builder, NamedTempFile};
use tracing::{info, warn};
use anyhow;

// Import our Python bindings crate
use mcp_pyo3_bindings::set_python_paths;

// Include the generated library info from build.rs
include!(concat!(env!("OUT_DIR"), "/python_lib_info.rs"));

// Test that we can launch a subprocess without Python initialization
#[test]
fn test_python_subprocess() -> anyhow::Result<()> {
    // Basic logging setup for tests
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_test_writer()
        .try_init();

    info!("Starting test_python_subprocess...");
    let temp_dir = Builder::new().prefix("mcp_python_test_").tempdir()
        .map_err(|e| anyhow::anyhow!("Failed to create temp dir: {}", e))?;
    let temp_dir_path = temp_dir.path();
    info!("Test Temp Dir: {:?}", temp_dir_path);

    // Set up Python paths using our shared function
    set_python_paths();
    
    // Get Python info from build time for logging
    let python_exec = get_python_executable();
    let python_version = get_python_version();
    let sys_prefix = get_python_sys_prefix();
    
    info!("Using Python executable: {}", python_exec);
    info!("Python version: {}", python_version);
    info!("Python sys.prefix: {}", sys_prefix);
    
    // Create a temporary Python script
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
import sys
import os

print("Python version: {{}}".format(sys.version))
print("Executable: {{}}".format(sys.executable))
print("Prefix: {{}}".format(sys.prefix))
print("PATH: {{}}".format(os.environ.get('PATH', '')))
print("PYTHONPATH: {{}}".format(os.environ.get('PYTHONPATH', '')))
print("PYTHONHOME: {{}}".format(os.environ.get('PYTHONHOME', '')))
print("LD_LIBRARY_PATH: {{}}".format(os.environ.get('LD_LIBRARY_PATH', '')))
"#
    )
    .unwrap();
    
    let script_path = temp_file.path().to_str().unwrap();
    info!("Created temporary script at: {}", script_path);
    
    // Prepare command environment - use the same environment as our process
    // since set_python_paths has already configured it
    let mut cmd = StdCommand::new(python_exec);
    cmd.current_dir(temp_dir_path);
    cmd.arg(script_path);
    
    // Execute the command
    info!("Executing Python subprocess...");
    let output = cmd.output().expect("Failed to execute Python subprocess");
    
    // Check the results
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    info!("Python subprocess stdout: {}", stdout);
    
    if !stderr.is_empty() {
        info!("Python subprocess stderr: {}", stderr);
    }
    
    assert!(output.status.success(), "Python subprocess failed");
    assert!(stdout.contains("Python version:"), "Missing Python version in output");
    
    info!("Python subprocess test completed successfully");
    Ok(())
} 