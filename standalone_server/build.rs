fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=../proto/mcp_task.proto");
    tonic_build::compile_protos("../proto/mcp_task.proto")?;
    Ok(())
} 