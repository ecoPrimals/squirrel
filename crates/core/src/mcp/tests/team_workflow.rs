use crate::mcp::{
    TeamWorkflow, TeamMessageType, ReviewSeverity, DocType, Priority,
    ProcessErrorType, BuildStatus, BuildMetrics,
};

#[tokio::test]
async fn test_team_workflow_initialization() {
    let mut workflow = TeamWorkflow::new();
    assert!(workflow.initialize_team_ports().await.is_ok());
}

#[tokio::test]
async fn test_review_feedback() {
    let workflow = TeamWorkflow::new();
    let result = workflow.send_review_feedback(
        "src/test.rs".to_string(),
        vec![1, 2, 3],
        ReviewSeverity::Warning,
        "Missing documentation".to_string(),
        vec!["1005-rust-documentation".to_string()],
    ).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_process_error() {
    let workflow = TeamWorkflow::new();
    let result = workflow.send_process_error(
        "build".to_string(),
        ProcessErrorType::Build,
        "Build failed due to missing dependency".to_string(),
        vec!["1006-rust-performance".to_string()],
    ).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_documentation_update() {
    let workflow = TeamWorkflow::new();
    let result = workflow.send_documentation_update(
        "MCP".to_string(),
        DocType::API,
        "Updated API documentation".to_string(),
        Priority::High,
    ).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_build_status() {
    let workflow = TeamWorkflow::new();
    let metrics = BuildMetrics {
        duration: 120.5,
        test_coverage: 85.5,
        warnings_count: 3,
        errors_count: 0,
    };
    let result = workflow.send_build_status(
        "feature/MCP".to_string(),
        BuildStatus::Success,
        metrics,
        vec!["Minor performance warning".to_string()],
    ).await;
    assert!(result.is_ok());
} 