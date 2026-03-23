// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

use super::*;
use crate::task::json_rpc_types::{
    AssignTaskRequest, CancelTaskRequest, CompleteTaskRequest, CreateTaskRequest, GetTaskRequest,
    JsonTask, ListTasksRequest, ReportProgressRequest, UpdateTaskRequest,
};
use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixListener;

fn test_client(socket_path: &Path) -> MCPTaskClient {
    MCPTaskClient::with_config(TaskClientConfig {
        server_address: socket_path.to_string_lossy().into_owned(),
        max_retries: 1,
        connect_timeout_ms: 5000,
        request_timeout_ms: 10000,
        initial_backoff_ms: 10,
        max_backoff_ms: 100,
    })
}

/// Single-shot JSON-RPC server: reads one request, writes one response, then closes.
fn run_mock_rpc_server(
    socket_path: &Path,
    response: serde_json::Value,
) -> tokio::task::JoinHandle<()> {
    let path = socket_path.to_path_buf();
    let _ = std::fs::remove_file(&path);
    let listener = UnixListener::bind(&path).expect("bind unix socket");
    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("accept");
        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).await.expect("read request");
        assert!(!buf.is_empty(), "expected JSON-RPC request bytes");
        let body = serde_json::to_vec(&response).expect("serialize response");
        stream.write_all(&body).await.expect("write response");
    })
}

#[test]
fn default_config_and_with_config_accessors() {
    let cfg = TaskClientConfig {
        server_address: "/tmp/unit-test.sock".to_string(),
        max_retries: 7,
        connect_timeout_ms: 100,
        request_timeout_ms: 200,
        initial_backoff_ms: 10,
        max_backoff_ms: 50,
    };
    let client = MCPTaskClient::with_config(cfg.clone());
    assert!(client.connect().is_ok());
    assert_eq!(client.server_address(), cfg.server_address);
    assert_eq!(client.max_retries(), 7);
    assert_eq!(client.connect_timeout(), 100);
    assert_eq!(client.request_timeout(), 200);
    assert_eq!(client.initial_backoff(), 10);
    assert_eq!(client.max_backoff(), 50);

    let def = MCPTaskClient::default();
    assert!(!def.server_address().is_empty());
}

#[test]
fn create_task_params_and_list_tasks_params_default() {
    let list = ListTasksParams::default();
    assert!(list.status.is_none());
    assert!(list.limit.is_none());

    let params = CreateTaskParams {
        name: "n".to_string(),
        description: "d".to_string(),
        priority: TaskPriority::High,
        input_data: Some(serde_json::json!({"k": 1})),
        metadata: Some(serde_json::json!({"m": true})),
        context_id: Some("ctx".to_string()),
        prerequisites: vec!["p1".to_string()],
    };
    let mut req = CreateTaskRequest {
        name: params.name.clone(),
        description: params.description.clone(),
        priority: params.priority as i32,
        input_data: Vec::new(),
        metadata: Vec::new(),
        prerequisite_task_ids: params.prerequisites.clone(),
        context_id: params.context_id.clone().unwrap_or_default(),
        agent_id: String::new(),
        agent_type: 0,
    };
    if let Some(data) = params.input_data.clone() {
        req.input_data = serde_json::to_vec(&data).unwrap();
    }
    if let Some(meta) = params.metadata {
        req.metadata = serde_json::to_vec(&meta).unwrap();
    }
    let v = serde_json::to_value(&req).unwrap();
    let back: CreateTaskRequest = serde_json::from_value(v).unwrap();
    assert_eq!(back.name, "n");
    assert!(!back.input_data.is_empty());
}

#[test]
fn json_rpc_request_types_roundtrip() {
    let ct = CreateTaskRequest {
        name: "t".to_string(),
        description: String::new(),
        priority: 1,
        input_data: vec![1, 2],
        metadata: vec![],
        prerequisite_task_ids: vec![],
        context_id: "c".to_string(),
        agent_id: "a".to_string(),
        agent_type: 2,
    };
    let v = serde_json::to_value(&ct).unwrap();
    let ct2: CreateTaskRequest = serde_json::from_value(v).unwrap();
    assert_eq!(ct2.agent_type, 2);

    let gt = GetTaskRequest {
        task_id: "id-1".to_string(),
    };
    let gt_rt: GetTaskRequest = serde_json::from_value(serde_json::to_value(&gt).unwrap()).unwrap();
    assert_eq!(gt_rt.task_id, gt.task_id);

    let lt = ListTasksRequest {
        status: -1,
        agent_id: String::new(),
        agent_type: -1,
        context_id: String::new(),
        limit: 50,
        offset: 0,
    };
    let lt2: ListTasksRequest = serde_json::from_value(serde_json::to_value(&lt).unwrap()).unwrap();
    assert_eq!(lt2.limit, 50);

    let assign = AssignTaskRequest {
        task_id: "t".to_string(),
        agent_id: "ag".to_string(),
        agent_type: AgentType::AI as i32,
    };
    let _: AssignTaskRequest =
        serde_json::from_value(serde_json::to_value(&assign).unwrap()).unwrap();

    let rp = ReportProgressRequest {
        task_id: "t".to_string(),
        progress_percent: 50,
        progress_message: "m".to_string(),
        interim_results: vec![],
    };
    let _: ReportProgressRequest =
        serde_json::from_value(serde_json::to_value(&rp).unwrap()).unwrap();

    let comp = CompleteTaskRequest {
        task_id: "t".to_string(),
        output_data: vec![0],
        metadata: vec![],
    };
    let _: CompleteTaskRequest =
        serde_json::from_value(serde_json::to_value(&comp).unwrap()).unwrap();

    let can = CancelTaskRequest {
        task_id: "t".to_string(),
        reason: "r".to_string(),
    };
    let _: CancelTaskRequest = serde_json::from_value(serde_json::to_value(&can).unwrap()).unwrap();
}

#[test]
fn json_task_to_task_maps_fields() {
    let jt = JsonTask {
        id: "tid".to_string(),
        name: "nm".to_string(),
        description: "desc".to_string(),
        status: TaskStatus::Running as i32,
        priority: TaskPriority::Medium as i32,
        agent_type: AgentType::AI as i32,
        progress_percent: 42,
        agent_id: "agent-1".to_string(),
        context_id: "ctx-1".to_string(),
        prerequisite_task_ids: vec!["pre".to_string()],
        created_at: None,
        updated_at: None,
        completed_at: None,
        input_data: serde_json::to_vec(&serde_json::json!({"x": "1"})).unwrap(),
        output_data: vec![],
        error_message: String::new(),
        progress_message: "working".to_string(),
        metadata: serde_json::to_vec(&serde_json::json!({"k": "v"})).unwrap(),
    };
    let task = json_task_to_task(jt);
    assert_eq!(task.id.as_ref(), "tid");
    assert_eq!(task.status_code, TaskStatus::Running);
    assert_eq!(task.priority_code, TaskPriority::Medium);
    assert_eq!(task.agent_type, AgentType::AI);
    assert_eq!(task.progress, 42.0);
    assert_eq!(task.agent_id.as_deref(), Some("agent-1"));
    assert_eq!(task.context_id.as_deref(), Some("ctx-1"));
    assert!(task.input_data.is_some());
    assert!(task.metadata.is_some());
    assert_eq!(task.status_message.as_deref(), Some("working"));
}

#[test]
fn json_task_empty_optional_bytes() {
    let jt = JsonTask {
        id: "1".to_string(),
        name: "n".to_string(),
        description: String::new(),
        status: 0,
        priority: 0,
        agent_type: 0,
        progress_percent: 0,
        agent_id: String::new(),
        context_id: String::new(),
        prerequisite_task_ids: vec![],
        created_at: None,
        updated_at: None,
        completed_at: None,
        input_data: vec![],
        output_data: vec![],
        error_message: String::new(),
        progress_message: String::new(),
        metadata: vec![],
    };
    let task = json_task_to_task(jt);
    assert!(task.input_data.is_none());
    assert!(task.output_data.is_none());
    assert!(task.metadata.is_none());
    assert!(task.agent_id.is_none());
    assert!(task.status_message.is_none());
}

#[test]
fn update_task_request_serializes_from_task() {
    let mut input = std::collections::HashMap::new();
    input.insert("k".to_string(), "v".to_string());
    let task = Task {
        id: std::sync::Arc::from("u1"),
        name: std::sync::Arc::from("name"),
        description: "d".to_string(),
        status_code: TaskStatus::Pending,
        priority_code: TaskPriority::Low,
        agent_type: AgentType::Unspecified,
        progress: 0.0,
        agent_id: None,
        context_id: None,
        parent_id: None,
        prerequisites: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        completed_at: None,
        input_data: Some(input),
        output_data: None,
        metadata: None,
        error_message: None,
        status_message: None,
        deadline: None,
        watchable: false,
        retry_count: 0,
        max_retries: 1,
    };
    let request = UpdateTaskRequest {
        task_id: task.id.as_ref().to_string(),
        name: task.name.as_ref().to_string(),
        description: task.description.clone(),
        priority: task.priority_code as i32,
        input_data: task
            .input_data
            .as_ref()
            .map(|m| serde_json::to_vec(m).unwrap_or_default())
            .unwrap_or_default(),
        metadata: task
            .metadata
            .as_ref()
            .map(|m| serde_json::to_vec(m).unwrap_or_default())
            .unwrap_or_default(),
    };
    let _: UpdateTaskRequest =
        serde_json::from_value(serde_json::to_value(&request).unwrap()).unwrap();
}

#[test]
fn json_task_to_task_invalid_json_bytes_yield_none() {
    let jt = JsonTask {
        id: "1".to_string(),
        name: "n".to_string(),
        description: String::new(),
        status: 0,
        priority: 0,
        agent_type: 0,
        progress_percent: 0,
        agent_id: String::new(),
        context_id: String::new(),
        prerequisite_task_ids: vec![],
        created_at: None,
        updated_at: None,
        completed_at: None,
        input_data: vec![0xff, 0xfe],
        output_data: vec![0x01],
        error_message: "err".to_string(),
        progress_message: String::new(),
        metadata: vec![],
    };
    let task = json_task_to_task(jt);
    assert!(task.input_data.is_none());
    assert!(task.output_data.is_none());
    assert_eq!(task.error_message.as_deref(), Some("err"));
}

#[tokio::test]
async fn json_rpc_create_task_success_and_rpc_error() {
    let dir = std::env::temp_dir();
    let path = dir.join(format!(
        "mcp_task_client_test_{}.sock",
        uuid::Uuid::new_v4()
    ));
    let ok = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "success": true,
            "task_id": "new-task-id",
            "error_message": ""
        }
    });
    let h = run_mock_rpc_server(&path, ok);
    let client = test_client(&path);
    let id = client
        .create_task(CreateTaskParams {
            name: "a".into(),
            description: "b".into(),
            priority: TaskPriority::Low,
            input_data: None,
            metadata: None,
            context_id: None,
            prerequisites: vec![],
        })
        .await
        .expect("create_task");
    assert_eq!(id, "new-task-id");
    h.await.unwrap();

    let path2 = dir.join(format!(
        "mcp_task_client_test_{}.sock",
        uuid::Uuid::new_v4()
    ));
    let err_resp = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "error": { "message": "server boom" }
    });
    let h2 = run_mock_rpc_server(&path2, err_resp);
    let client2 = test_client(&path2);
    let e = client2
        .create_task(CreateTaskParams {
            name: "a".into(),
            description: "b".into(),
            priority: TaskPriority::Low,
            input_data: None,
            metadata: None,
            context_id: None,
            prerequisites: vec![],
        })
        .await
        .expect_err("expected rpc error");
    assert!(e.to_string().contains("server boom"));
    h2.await.unwrap();
}

#[tokio::test]
async fn json_rpc_get_task_missing_task_and_list_success_false() {
    let dir = std::env::temp_dir();
    let path = dir.join(format!(
        "mcp_task_client_test_{}.sock",
        uuid::Uuid::new_v4()
    ));
    let resp = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "success": true,
            "task": null,
            "error_message": ""
        }
    });
    let h = run_mock_rpc_server(&path, resp);
    let client = test_client(&path);
    let err = client.get_task("x").await.expect_err("no task in body");
    assert!(err.to_string().contains("No task"));
    h.await.unwrap();

    let path2 = dir.join(format!(
        "mcp_task_client_test_{}.sock",
        uuid::Uuid::new_v4()
    ));
    let resp2 = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "success": false,
            "tasks": [],
            "total_count": 0,
            "error_message": "list failed"
        }
    });
    let h2 = run_mock_rpc_server(&path2, resp2);
    let client2 = test_client(&path2);
    let e = client2
        .list_tasks(ListTasksParams::default())
        .await
        .expect_err("list failed");
    assert!(e.to_string().contains("list failed"));
    h2.await.unwrap();
}

#[tokio::test]
async fn json_rpc_assign_report_complete_cancel_branches() {
    let dir = std::env::temp_dir();
    for (path, method_result, call) in [
        (
            dir.join(format!("as_{}.sock", uuid::Uuid::new_v4())),
            serde_json::json!({
                "success": false,
                "error_message": "assign bad"
            }),
            "assign",
        ),
        (
            dir.join(format!("rp_{}.sock", uuid::Uuid::new_v4())),
            serde_json::json!({
                "success": false,
                "error_message": "progress bad"
            }),
            "progress",
        ),
        (
            dir.join(format!("cp_{}.sock", uuid::Uuid::new_v4())),
            serde_json::json!({
                "success": false,
                "error_message": "complete bad"
            }),
            "complete",
        ),
        (
            dir.join(format!("cn_{}.sock", uuid::Uuid::new_v4())),
            serde_json::json!({
                "success": false,
                "error_message": "cancel bad"
            }),
            "cancel",
        ),
    ] {
        let resp = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": method_result
        });
        let _ = std::fs::remove_file(&path);
        let h = run_mock_rpc_server(&path, resp);
        let client = test_client(&path);
        let r = match call {
            "assign" => client.assign_task("t1", "a1", AgentType::AI).await,
            "progress" => client.report_progress("t1", 50, Some("hi")).await,
            "complete" => client.complete_task("t1", None, None).await,
            "cancel" => client.cancel_task("t1", "why").await,
            _ => unreachable!(),
        };
        assert!(r.is_err());
        h.await.unwrap();
    }
}

#[tokio::test]
async fn json_rpc_update_task_failure_response() {
    let dir = std::env::temp_dir();
    let path = dir.join(format!("upd_{}.sock", uuid::Uuid::new_v4()));
    let resp = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "success": false,
            "error_message": "cannot update"
        }
    });
    let h = run_mock_rpc_server(&path, resp);
    let client = test_client(&path);
    let task = Task {
        id: std::sync::Arc::from("tid"),
        name: std::sync::Arc::from("n"),
        description: "d".into(),
        status_code: TaskStatus::Pending,
        priority_code: TaskPriority::Low,
        agent_type: AgentType::Unspecified,
        progress: 0.0,
        agent_id: None,
        context_id: None,
        parent_id: None,
        prerequisites: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        completed_at: None,
        input_data: None,
        output_data: None,
        metadata: None,
        error_message: None,
        status_message: None,
        deadline: None,
        watchable: false,
        retry_count: 0,
        max_retries: 1,
    };
    let err = client
        .update_task(&task)
        .await
        .expect_err("update should fail");
    assert!(err.to_string().contains("cannot update"));
    h.await.unwrap();
}

#[tokio::test]
async fn list_tasks_request_builds_filters_and_limits() {
    let dir = std::env::temp_dir();
    let path = dir.join(format!("lt_{}.sock", uuid::Uuid::new_v4()));
    let resp = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "success": true,
            "tasks": [],
            "total_count": 0,
            "error_message": ""
        }
    });
    let h = run_mock_rpc_server(&path, resp);
    let client = test_client(&path);
    client
        .list_tasks(ListTasksParams {
            status: Some(TaskStatus::Running),
            agent_id: Some("ag".into()),
            agent_type: Some(AgentType::Human),
            context_id: Some("ctx".into()),
            limit: Some(2000),
            offset: Some(5),
        })
        .await
        .expect("list");
    h.await.unwrap();
}
