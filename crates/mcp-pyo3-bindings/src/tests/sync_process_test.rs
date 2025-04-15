use std::env;
use std::path::Path;
use std::process::Command;

use crate::{setup_python_environment, get_python_executable_path};

#[test]
fn test_sync_python_process() {
    // Setup Python environment
    setup_python_environment();
    
    // Get Python executable path from our environment
    let python_exe = get_python_executable_path();
    assert!(Path::new(&python_exe).exists(), "Python executable not found at: {}", python_exe);
    
    // Simple test script to verify Python is working
    let script = r#"
import sys
print("Python version: {}".format(sys.version))
print("SUCCESS")
"#;

    // Create a temporary file for the script
    let temp_dir = env::temp_dir();
    let script_path = temp_dir.join("test_script.py");
    std::fs::write(&script_path, script).expect("Failed to write test script");
    
    // Run Python with our script
    let output = Command::new(&python_exe)
        .arg(&script_path)
        .output()
        .expect("Failed to execute python process");
    
    // Clean up
    std::fs::remove_file(script_path).ok();
    
    // Check results
    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("Python stdout: {}", stdout);
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        println!("Python stderr: {}", stderr);
    }
    
    assert!(output.status.success(), "Python process failed");
    assert!(stdout.contains("SUCCESS"), "Expected SUCCESS in output");
} 