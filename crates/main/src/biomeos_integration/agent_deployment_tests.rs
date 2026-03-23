// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

use super::*;
use crate::biomeos_integration::manifest::BiomeManifestParser;
use std::sync::Arc;

#[tokio::test]
async fn test_agent_deployment_creation() {
    let config = AgentDeploymentConfig::default();
    let mcp_integration = Arc::new(McpIntegration::new());
    let ai_intelligence = Arc::new(AiIntelligence::new());

    let manager = AgentDeploymentManager::new(config, mcp_integration, ai_intelligence);

    assert_eq!(manager.list_agents().await.len(), 0);
}

#[tokio::test]
async fn test_manifest_deployment() {
    let config = AgentDeploymentConfig::default();
    let mcp_integration = Arc::new(McpIntegration::new());
    let ai_intelligence = Arc::new(AiIntelligence::new());

    let manager = AgentDeploymentManager::new(config, mcp_integration, ai_intelligence);

    let manifest = BiomeManifestParser::generate_template();
    let result = manager.deploy_from_manifest(&manifest).await;

    // This would pass once the full implementation is complete
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_agent_validation() {
    let config = AgentDeploymentConfig::default();
    let mcp_integration = Arc::new(McpIntegration::new());
    let ai_intelligence = Arc::new(AiIntelligence::new());

    let manager = AgentDeploymentManager::new(config, mcp_integration, ai_intelligence);

    let manifest = BiomeManifestParser::generate_template();
    let agent_spec = &manifest.agents[0];

    let result = manager.validate_agent_spec(agent_spec).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_validate_manifest_rejects_too_many_agents() {
    let config = AgentDeploymentConfig {
        max_concurrent_agents: 1,
        ..Default::default()
    };
    let manager = AgentDeploymentManager::new(
        config,
        Arc::new(McpIntegration::new()),
        Arc::new(AiIntelligence::new()),
    );
    let mut manifest = BiomeManifestParser::generate_template();
    manifest.agents.push(manifest.agents[0].clone());
    let err = manager
        .deploy_from_manifest(&manifest)
        .await
        .expect_err("should reject");
    assert!(err.to_string().contains("maximum") || err.to_string().contains("Cannot deploy"));
}

#[tokio::test]
async fn test_stop_agent_not_found() {
    let manager = AgentDeploymentManager::new(
        AgentDeploymentConfig::default(),
        Arc::new(McpIntegration::new()),
        Arc::new(AiIntelligence::new()),
    );
    let err = manager.stop_agent("missing").await.expect_err("not found");
    assert!(err.to_string().contains("not found") || err.to_string().contains("Agent"));
}

#[tokio::test]
async fn test_get_agent_status_not_found() {
    let manager = AgentDeploymentManager::new(
        AgentDeploymentConfig::default(),
        Arc::new(McpIntegration::new()),
        Arc::new(AiIntelligence::new()),
    );
    let err = manager
        .get_agent_status("nope")
        .await
        .expect_err("not found");
    assert!(err.to_string().contains("not found") || err.to_string().contains("Agent"));
}

#[tokio::test]
async fn test_deploy_native_wasm_and_vm_cover_start_branches() {
    let mut config = AgentDeploymentConfig::default();
    config.security.allowed_execution_environments = vec![
        ExecutionEnvironment::Native,
        ExecutionEnvironment::Wasm,
        ExecutionEnvironment::Container,
        ExecutionEnvironment::VirtualMachine,
    ];
    let manager = AgentDeploymentManager::new(
        config,
        Arc::new(McpIntegration::new()),
        Arc::new(AiIntelligence::new()),
    );
    let mut m = BiomeManifestParser::generate_template();
    m.agents[0].execution_environment = ExecutionEnvironment::Native;
    let id = manager.deploy_agent(&m.agents[0]).await.expect("native");
    manager.stop_agent(&id).await.expect("stop");

    let mut a = m.agents[0].clone();
    a.name = "wasm-a".to_string();
    a.execution_environment = ExecutionEnvironment::Wasm;
    let id2 = manager.deploy_agent(&a).await.expect("wasm");
    manager.stop_agent(&id2).await.expect("stop");

    let mut b = m.agents[0].clone();
    b.name = "vm-a".to_string();
    b.execution_environment = ExecutionEnvironment::VirtualMachine;
    let id3 = manager.deploy_agent(&b).await.expect("vm");
    manager.stop_agent(&id3).await.expect("stop");
}

#[tokio::test]
async fn test_generate_agent_endpoints_monitoring_and_health_capabilities() {
    use crate::biomeos_integration::manifest::AgentManifest;
    let manager = AgentDeploymentManager::new(
        AgentDeploymentConfig::default(),
        Arc::new(McpIntegration::new()),
        Arc::new(AiIntelligence::new()),
    );
    let mut spec = BiomeManifestParser::generate_template().agents[0].clone();
    spec.name = "mon-agent".to_string();
    spec.resources.memory_limit_mb = Some(4096);
    spec.manifest = Some(AgentManifest {
        version: "1".to_string(),
        description: "d".to_string(),
        author: "a".to_string(),
        capabilities: vec!["monitoring".to_string(), "health_reporting".to_string()],
        dependencies: vec![],
        metadata: std::collections::HashMap::new(),
    });
    let id = manager.deploy_agent(&spec).await.expect("deploy");
    let agents = manager.list_agents().await;
    let deployed = agents.iter().find(|a| a.agent_id == id).expect("deployed");
    assert!(deployed.endpoints.metrics.contains("metrics"));
    assert!(deployed.endpoints.health.contains("health"));
}

#[tokio::test]
async fn test_health_check_marks_failed_when_not_running() {
    let manager = AgentDeploymentManager::new(
        AgentDeploymentConfig::default(),
        Arc::new(McpIntegration::new()),
        Arc::new(AiIntelligence::new()),
    );
    let spec = BiomeManifestParser::generate_template().agents[0].clone();
    let id = manager.deploy_agent(&spec).await.expect("deploy");
    {
        let mut map = manager.deployed_agents.write().await;
        if let Some(a) = map.get_mut(&id) {
            a.status = AgentStatus::Deploying;
        }
    }
    manager.health_check().await.expect("hc");
    let st = manager.get_agent_status(&id).await.expect("st");
    assert!(matches!(st, AgentStatus::Failed(_)));
}
