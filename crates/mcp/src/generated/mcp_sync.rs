// Re-export everything from the generated code
// This includes all the protobuf-generated types and clients/servers for the mcp_sync service

// Include the generated code from the OUT_DIR environment variable
include!(concat!(env!("OUT_DIR"), "/mcp.sync.rs")); 