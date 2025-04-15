fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Uncomment the following to regenerate protobuf code
    println!("cargo:rerun-if-changed=../../proto/mcp_sync.proto");
    println!("cargo:rerun-if-changed=../../proto/mcp_task.proto");
    
    // Generate gRPC code from proto files
    tonic_build::configure()
        .build_server(true)  // Enable server code generation
        .build_client(true)  // Enable client code generation
        // Remove the serde attributes since they cause issues with Timestamp
        // .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile_protos(
            &[
                "../../proto/mcp_sync.proto",
                "../../proto/mcp_task.proto",
            ], 
            &["../../proto"]
        )?;
    
    println!("cargo:rerun-if-changed=build.rs");
    
    Ok(())
} 