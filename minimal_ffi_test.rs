use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::time::Duration;

fn main() {
    // Configure logging
    env::set_var("RUST_LOG", "debug,mcp_python_adapter=trace");
    env_logger::init();
    println!("Starting minimal FFI test");

    // Create a temporary directory for the runtime
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let runtime_dir = temp_dir.path().to_path_buf();
    println!("Created temporary runtime directory: {:?}", runtime_dir);

    // Extract runtime
    mcp_python_adapter::runtime::extract_runtime(&runtime_dir)
        .expect("Failed to extract runtime");
    println!("Extracted runtime to temporary directory");

    // Check if bootstrap.py exists
    let bootstrap_path = runtime_dir.join("bootstrap.py");
    if !bootstrap_path.exists() {
        panic!("Bootstrap file does not exist at {:?}", bootstrap_path);
    }
    println!("Verified bootstrap.py exists at {:?}", bootstrap_path);

    // Create interpreter config
    let interpreter_config = mcp_python_adapter::config::InterpreterConfig {
        python_executable: Some("python3".to_string()),
        working_dir: Some(runtime_dir.to_string_lossy().to_string()),
        runtime_dir: Some(runtime_dir.to_string_lossy().to_string()),
        env_vars: Some(HashMap::from([
            ("PYTHONUNBUFFERED".to_string(), "1".to_string()),
            ("PYTHONDEBUG".to_string(), "1".to_string()),
            ("PYTHONPATH".to_string(), runtime_dir.to_string_lossy().to_string()),
        ])),
        log_level: Some("DEBUG".to_string()),
        use_sandbox: false,
        capture_output: true,
        command_timeout: Some(30),
        ..mcp_python_adapter::config::InterpreterConfig::default()
    };

    // Create full config
    let config = mcp_python_adapter::config::PythonFFIConfig {
        interpreter: interpreter_config,
        resource_limits: mcp_python_adapter::config::ResourceLimits::default(),
        security: mcp_python_adapter::config::SecurityConfig {
            use_sandbox: false,
            ..mcp_python_adapter::config::SecurityConfig::default()
        },
        auto_start: true,
    };

    // Run in a tokio runtime
    let runtime = tokio::runtime::Runtime::new().expect("Failed to create runtime");
    runtime.block_on(async {
        // Initialize FFI
        let mut ffi = mcp_python_adapter::ffi::process::ProcessFFI::new(config);
        println!("Created ProcessFFI instance");

        // Start FFI
        match ffi.start().await {
            Ok(_) => println!("FFI started successfully"),
            Err(e) => {
                println!("Failed to start FFI: {}", e);
                return;
            }
        }

        // Create simple command
        let command = mcp_python_adapter::ffi::Command {
            id: "test_execute_1".to_string(),
            command_type: mcp_python_adapter::ffi::CommandType::Execute,
            params: mcp_python_adapter::ffi::CommandParams::Execute { 
                code: "2 + 2".to_string(),
            },
        };

        // Send command
        println!("Sending execute code command: {:?}", command);
        match ffi.send_command(command).await {
            Ok(response) => {
                println!("Received response: {:?}", response);
                assert!(response.success, "Command execution failed: {:?}", response.error);
            },
            Err(e) => {
                println!("Failed to send command: {}", e);
            }
        }

        // Stop FFI
        match ffi.stop().await {
            Ok(_) => println!("FFI stopped successfully"),
            Err(e) => println!("Failed to stop FFI: {}", e),
        }
    });

    println!("Test completed");
} 