use std::env;
use std::fs;
use std::path::Path;
use std::process::Stdio;
use std::io::{Write, BufRead, BufReader};
use std::time::Duration;

const BOOTSTRAP_CONTENT: &str = r#"#!/usr/bin/env python3
import json
import sys
import traceback

print("Bootstrap starting", file=sys.stderr)

# Process commands from stdin
for line in sys.stdin:
    line = line.strip()
    if not line:
        continue
    
    try:
        # Parse command
        command = json.loads(line)
        command_id = command.get("id", "unknown")
        command_type = command.get("command_type", "")
        params = command.get("params", {})
        
        # Execute code
        if command_type == "execute":
            code = params.get("code", "")
            try:
                result = eval(code)
                response = {
                    "id": command_id,
                    "success": True,
                    "result": result,
                    "error": None
                }
            except Exception as e:
                response = {
                    "id": command_id,
                    "success": False,
                    "result": None,
                    "error": str(e)
                }
        elif command_type == "shutdown":
            response = {
                "id": command_id,
                "success": True,
                "result": None,
                "error": None
            }
            # Exit after shutdown
            print(json.dumps(response), flush=True)
            break
        else:
            response = {
                "id": command_id,
                "success": False,
                "result": None,
                "error": f"Unsupported command type: {command_type}"
            }
        
        # Send response
        print(json.dumps(response), flush=True)
    
    except Exception as e:
        # Create error response for parsing errors
        response = {
            "id": "error",
            "success": False,
            "result": None,
            "error": f"Failed to process command: {str(e)}"
        }
        print(json.dumps(response), flush=True)

print("Bootstrap exiting", file=sys.stderr)
"#;

fn main() {
    // Setup temp directory for bootstrap script
    let temp_dir = std::env::temp_dir().join("simple_test");
    if !temp_dir.exists() {
        fs::create_dir_all(&temp_dir).expect("Failed to create temp directory");
    }
    
    let bootstrap_path = temp_dir.join("bootstrap.py");
    
    // Write bootstrap script
    fs::write(&bootstrap_path, BOOTSTRAP_CONTENT).expect("Failed to write bootstrap script");
    
    // Make it executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&bootstrap_path).expect("Failed to get file metadata").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&bootstrap_path, perms).expect("Failed to set permissions");
    }
    
    println!("Created bootstrap script at {:?}", bootstrap_path);
    
    // Start Python process
    let mut cmd = std::process::Command::new("python3");
    cmd.arg(&bootstrap_path)
       .stdin(Stdio::piped())
       .stdout(Stdio::piped())
       .stderr(Stdio::piped())
       .env("PYTHONUNBUFFERED", "1");
    
    println!("Starting Python process: {:?}", cmd);
    
    let mut child = cmd.spawn().expect("Failed to start Python process");
    let stdin = child.stdin.take().expect("Failed to get stdin handle");
    let stdout = child.stdout.take().expect("Failed to get stdout handle");
    let stderr = child.stderr.take().expect("Failed to get stderr handle");
    
    // Start thread to read stderr
    let stderr_thread = std::thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            match line {
                Ok(line) => println!("Python stderr: {}", line),
                Err(e) => eprintln!("Error reading stderr: {}", e),
            }
        }
    });
    
    // Create test command to execute 2+2
    let command = serde_json::json!({
        "id": "test_execute_1",
        "command_type": "execute",
        "params": {
            "code": "2 + 2"
        }
    });
    
    // Send command
    let command_str = serde_json::to_string(&command).expect("Failed to serialize command");
    println!("Sending command: {}", command_str);
    
    let mut stdin_writer = std::io::BufWriter::new(stdin);
    stdin_writer.write_all(command_str.as_bytes()).expect("Failed to write command");
    stdin_writer.write_all(b"\n").expect("Failed to write newline");
    stdin_writer.flush().expect("Failed to flush stdin");
    
    // Read response
    let mut stdout_reader = BufReader::new(stdout);
    let mut response_line = String::new();
    stdout_reader.read_line(&mut response_line).expect("Failed to read response");
    
    println!("Received response: {}", response_line.trim());
    
    // Parse response
    let response: serde_json::Value = serde_json::from_str(&response_line).expect("Failed to parse response");
    
    // Verify result is 4
    let success = response["success"].as_bool().expect("Missing success field");
    assert!(success, "Command failed: {:?}", response["error"]);
    
    let result = response["result"].as_i64().expect("Missing or invalid result");
    assert_eq!(result, 4, "Expected 4, got {}", result);
    
    // Send shutdown command
    let shutdown_command = serde_json::json!({
        "id": "shutdown",
        "command_type": "shutdown",
        "params": {}
    });
    
    let shutdown_str = serde_json::to_string(&shutdown_command).expect("Failed to serialize shutdown command");
    println!("Sending shutdown command: {}", shutdown_str);
    
    stdin_writer.write_all(shutdown_str.as_bytes()).expect("Failed to write shutdown command");
    stdin_writer.write_all(b"\n").expect("Failed to write newline");
    stdin_writer.flush().expect("Failed to flush stdin");
    
    // Read shutdown response
    let mut shutdown_response = String::new();
    stdout_reader.read_line(&mut shutdown_response).expect("Failed to read shutdown response");
    
    println!("Received shutdown response: {}", shutdown_response.trim());
    
    // Wait for process to exit
    match child.wait() {
        Ok(status) => println!("Process exited with status: {}", status),
        Err(e) => eprintln!("Failed to wait for process: {}", e),
    }
    
    // Wait for stderr thread to finish
    stderr_thread.join().expect("Failed to join stderr thread");
    
    println!("Test completed successfully!");
} 