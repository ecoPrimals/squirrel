use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader, Write};
use std::env;
use std::fs;

const BOOTSTRAP_SCRIPT: &str = r#"#!/usr/bin/env python3
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
            # Print response and exit
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
        tb = traceback.format_exc()
        print(f"Error: {e}\n{tb}", file=sys.stderr)
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
    println!("Starting Python process test");
    
    // Create temp directory and bootstrap script
    let temp_dir = env::temp_dir().join("python_test");
    fs::create_dir_all(&temp_dir).expect("Failed to create temp directory");
    let script_path = temp_dir.join("bootstrap.py");
    fs::write(&script_path, BOOTSTRAP_SCRIPT).expect("Failed to write bootstrap script");
    
    // Make script executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path).expect("Failed to get permissions").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).expect("Failed to set permissions");
    }
    
    println!("Created bootstrap script at: {:?}", script_path);
    
    // Spawn Python process
    let mut cmd = Command::new("python3");
    cmd.arg(&script_path)
       .stdin(Stdio::piped())
       .stdout(Stdio::piped())
       .stderr(Stdio::piped())
       .env("PYTHONUNBUFFERED", "1");
    
    println!("Starting command: {:?}", cmd);
    
    let mut child = cmd.spawn().expect("Failed to spawn process");
    let mut stdin = child.stdin.take().expect("Failed to get stdin");
    let stdout = child.stdout.take().expect("Failed to get stdout");
    let stderr = child.stderr.take().expect("Failed to get stderr");
    
    // Start thread to read stderr
    let stderr_thread = std::thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            match line {
                Ok(line) => println!("STDERR: {}", line),
                Err(e) => eprintln!("Error reading stderr: {}", e),
            }
        }
    });
    
    // Create a BufReader for stdout
    let mut stdout_reader = BufReader::new(stdout);
    
    // Build test command
    let command = format!(
        r#"{{"id": "test1", "command_type": "execute", "params": {{"code": "2 + 2"}}}}"#
    );
    
    println!("Sending command: {}", command);
    
    // Send command
    stdin.write_all(command.as_bytes()).expect("Failed to write command");
    stdin.write_all(b"\n").expect("Failed to write newline");
    stdin.flush().expect("Failed to flush stdin");
    
    // Read response
    let mut response = String::new();
    stdout_reader.read_line(&mut response).expect("Failed to read response");
    
    println!("Received response: {}", response.trim());
    
    // Send shutdown command
    let shutdown = r#"{"id": "shutdown", "command_type": "shutdown", "params": {}}"#;
    println!("Sending shutdown command: {}", shutdown);
    
    stdin.write_all(shutdown.as_bytes()).expect("Failed to write shutdown command");
    stdin.write_all(b"\n").expect("Failed to write newline");
    stdin.flush().expect("Failed to flush stdin");
    
    // Read shutdown response
    let mut shutdown_response = String::new();
    stdout_reader.read_line(&mut shutdown_response).expect("Failed to read shutdown response");
    
    println!("Received shutdown response: {}", shutdown_response.trim());
    
    // Wait for process to exit
    match child.wait() {
        Ok(status) => println!("Process exited with status: {}", status),
        Err(e) => eprintln!("Error waiting for process: {}", e),
    }
    
    // Wait for stderr thread to finish
    let _ = stderr_thread.join();
    
    println!("Test completed successfully!");
} 