pub mod client;
#[cfg(feature = "python")]
pub mod python;

pub mod mcp_task {
    tonic::include_proto!("mcp.task");
}

pub fn add_task_prefix(name: &str) -> String {
    format!("TASK:{}", name)
} 