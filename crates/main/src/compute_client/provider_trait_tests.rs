// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use super::*;

struct MockComputeProvider {
    name: String,
}

impl ComputeProvider for MockComputeProvider {
    fn provider_name(&self) -> &str {
        &self.name
    }

    async fn get_capabilities(&self) -> ComputeResult<Vec<ComputeCapabilityType>> {
        Ok(vec![])
    }

    async fn execute_workload(&self, _spec: WorkloadExecutionSpec) -> ComputeResult<Uuid> {
        Ok(Uuid::new_v4())
    }

    async fn get_workload_status(&self, id: Uuid) -> ComputeResult<WorkloadExecutionResult> {
        Ok(WorkloadExecutionResult {
            id,
            status: WorkloadStatus::Running,
            exit_code: None,
            logs: None,
            metadata: HashMap::new(),
        })
    }

    async fn cancel_workload(&self, _id: Uuid) -> ComputeResult<()> {
        Ok(())
    }

    async fn list_workloads(&self) -> ComputeResult<Vec<WorkloadExecutionResult>> {
        Ok(vec![])
    }

    async fn health_check(&self) -> bool {
        true
    }
}

fn sample_workload_spec() -> WorkloadExecutionSpec {
    WorkloadExecutionSpec {
        id: Uuid::new_v4(),
        name: "test-workload".to_string(),
        image: "test-image".to_string(),
        command: vec!["echo".to_string(), "hello".to_string()],
        environment: HashMap::new(),
        resources: ResourceRequirements {
            cpu_cores: 1,
            memory_gb: 1,
            gpu_units: None,
            storage_gb: 10,
            max_execution_time: std::time::Duration::from_secs(60),
            network_bandwidth_mbps: None,
        },
        labels: HashMap::new(),
    }
}

#[tokio::test]
async fn test_compute_provider_trait() {
    let provider = MockComputeProvider {
        name: "test".to_string(),
    };

    assert_eq!(provider.provider_name(), "test");
    assert!(provider.health_check().await);

    let capabilities = provider.get_capabilities().await.expect("should succeed");
    assert_eq!(capabilities.len(), 0);
}

#[tokio::test]
async fn test_execute_workload() {
    let provider = MockComputeProvider {
        name: "test".to_string(),
    };

    let spec = sample_workload_spec();
    let workload_id = provider
        .execute_workload(spec)
        .await
        .expect("should succeed");
    let status = provider
        .get_workload_status(workload_id)
        .await
        .expect("should succeed");
    assert_eq!(status.status, WorkloadStatus::Running);
}

#[test]
fn auto_detect_unknown_provider_type_from_env_errors() {
    temp_env::with_var("COMPUTE_PROVIDER_TYPE", Some("quantum-hypervisor"), || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("rt");
        let result = rt.block_on(auto_detect_compute_provider());
        assert!(result.is_err(), "expected err");
        let Err(e) = result else {
            unreachable!("expected err");
        };
        match e {
            ComputeProviderError::NotAvailable(msg) => {
                assert!(
                    msg.contains("COMPUTE_ENDPOINT"),
                    "Error should guide to setting COMPUTE_ENDPOINT: {msg}"
                );
            }
            ref other => unreachable!("unexpected {other:?}"),
        }
    });
}

#[test]
fn auto_detect_local_provider_succeeds() {
    temp_env::with_var("COMPUTE_PROVIDER_TYPE", Some("local"), || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("rt");
        let provider = rt
            .block_on(auto_detect_compute_provider())
            .expect("local provider should succeed");
        assert_eq!(provider.provider_name(), "local");
    });
}

#[tokio::test]
async fn local_provider_rejects_workload_execution() {
    let provider = LocalProcessProvider::new();
    let spec = WorkloadExecutionSpec {
        id: Uuid::new_v4(),
        name: "test-local".to_string(),
        image: "none".to_string(),
        command: vec!["echo".to_string()],
        environment: HashMap::new(),
        resources: ResourceRequirements {
            cpu_cores: 1,
            memory_gb: 1,
            gpu_units: None,
            storage_gb: 1,
            max_execution_time: std::time::Duration::from_secs(5),
            network_bandwidth_mbps: None,
        },
        labels: HashMap::new(),
    };
    let result = provider.execute_workload(spec).await;
    assert!(
        result.is_err(),
        "local provider should reject workloads (development fallback only)"
    );
}

#[tokio::test]
async fn mock_metadata_includes_provider_key() {
    let provider = MockComputeProvider {
        name: "meta-test".to_string(),
    };
    let m = provider.metadata();
    assert_eq!(m.get("provider").map(String::as_str), Some("meta-test"));
}

#[tokio::test]
async fn test_get_available_resources_default_impl() {
    let provider = MockComputeProvider {
        name: "res-test".to_string(),
    };
    let r = provider
        .get_available_resources()
        .await
        .expect("should succeed");
    assert_eq!(r.cpu_cores, u32::MAX);
}

#[test]
fn auto_detect_with_compute_endpoint_creates_remote() {
    temp_env::with_vars(
        [
            ("COMPUTE_PROVIDER_TYPE", None::<&str>),
            ("COMPUTE_ENDPOINT", Some("unix:///tmp/compute.sock")),
        ],
        || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("rt");
            let provider = rt
                .block_on(auto_detect_compute_provider())
                .expect("remote provider creation should succeed");
            assert_eq!(provider.provider_name(), "remote");
        },
    );
}

#[test]
fn auto_detect_explicit_remote_type_needs_endpoint() {
    temp_env::with_vars(
        [
            ("COMPUTE_PROVIDER_TYPE", Some("remote")),
            ("COMPUTE_ENDPOINT", None::<&str>),
            ("COMPUTE_SERVICE_ENDPOINT", None::<&str>),
            ("TOADSTOOL_ENDPOINT", None::<&str>),
        ],
        || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("rt");
            let result = rt.block_on(auto_detect_compute_provider());
            assert!(result.is_err(), "remote type without endpoint should error");
        },
    );
}

#[tokio::test]
async fn remote_provider_metadata_includes_endpoint() {
    let provider = RemoteComputeProvider::new("unix:///tmp/test.sock".to_string());
    let m = provider.metadata();
    assert_eq!(m.get("provider").map(String::as_str), Some("remote"));
    assert_eq!(
        m.get("endpoint").map(String::as_str),
        Some("unix:///tmp/test.sock")
    );
}

#[tokio::test]
async fn remote_provider_health_check_fails_on_unreachable() {
    let provider = RemoteComputeProvider::new("unix:///nonexistent/socket.sock".to_string());
    assert!(!provider.health_check().await);
}

#[test]
fn auto_detect_no_env_falls_back_to_local() {
    temp_env::with_vars(
        [
            ("COMPUTE_PROVIDER_TYPE", None::<&str>),
            ("COMPUTE_ENDPOINT", None::<&str>),
            ("COMPUTE_SERVICE_ENDPOINT", None::<&str>),
            ("TOADSTOOL_ENDPOINT", None::<&str>),
        ],
        || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("rt");
            let provider = rt
                .block_on(auto_detect_compute_provider())
                .expect("should fall back to local");
            assert_eq!(provider.provider_name(), "local");
        },
    );
}

#[test]
fn workload_status_from_wire_round_trips() {
    assert_eq!(
        workload_status_from_wire("Running"),
        WorkloadStatus::Running
    );
    assert_eq!(
        workload_status_from_wire("Completed"),
        WorkloadStatus::Completed
    );
    assert_eq!(workload_status_from_wire("Failed"), WorkloadStatus::Failed);
    assert_eq!(
        workload_status_from_wire("Cancelled"),
        WorkloadStatus::Cancelled
    );
    assert_eq!(
        workload_status_from_wire("Unknown"),
        WorkloadStatus::Pending
    );
}
