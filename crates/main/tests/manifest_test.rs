//! Tests for biome.yaml manifest parsing functionality

use squirrel::biomeos_integration::manifest::*;

#[tokio::test]
async fn test_manifest_generation() {
    let template = BiomeManifestParser::generate_template();

    assert_eq!(template.metadata.name, "example-biome");
    assert!(!template.agents.is_empty());
    assert_eq!(template.agents[0].name, "data-analyst");
    assert_eq!(template.agents[0].ai_provider, "openai");
    assert_eq!(template.agents[0].model, "gpt-4");
}

#[tokio::test]
async fn test_manifest_yaml_serialization() {
    let template = BiomeManifestParser::generate_template();

    // Test YAML serialization
    let yaml_content = serde_yaml::to_string(&template).unwrap();
    assert!(!yaml_content.is_empty());
    assert!(yaml_content.contains("name: example-biome"));
    assert!(yaml_content.contains("data-analyst"));
}

#[tokio::test]
async fn test_manifest_parsing() {
    let template = BiomeManifestParser::generate_template();
    let yaml_content = serde_yaml::to_string(&template).unwrap();

    let parser = BiomeManifestParser::new();
    let parsed = parser.parse_content(&yaml_content).await.unwrap();

    assert_eq!(parsed.metadata.name, "example-biome");
    assert_eq!(parsed.agents.len(), template.agents.len());
    assert_eq!(parsed.agents[0].name, "data-analyst");
}

#[tokio::test]
async fn test_manifest_validation() {
    let parser = BiomeManifestParser::new();

    // Test invalid manifest with empty name
    let mut template = BiomeManifestParser::generate_template();
    template.metadata.name = "".to_string();

    let yaml_content = serde_yaml::to_string(&template).unwrap();
    let result = parser.parse_content(&yaml_content).await;

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("name cannot be empty"));
}

#[tokio::test]
async fn test_agent_validation() {
    let parser = BiomeManifestParser::new();

    // Test invalid agent with empty capabilities
    let mut template = BiomeManifestParser::generate_template();
    template.agents[0].capabilities = vec![];

    let yaml_content = serde_yaml::to_string(&template).unwrap();
    let result = parser.parse_content(&yaml_content).await;

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("must have at least one capability"));
}

#[tokio::test]
async fn test_resource_validation() {
    let parser = BiomeManifestParser::new();

    // Test invalid resource limits
    let mut template = BiomeManifestParser::generate_template();
    template.resources.limits.cpu_cores = 0.0;

    let yaml_content = serde_yaml::to_string(&template).unwrap();
    let result = parser.parse_content(&yaml_content).await;

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("CPU cores must be greater than 0"));
}

#[tokio::test]
async fn test_execution_environment_equality() {
    let env1 = ExecutionEnvironment::Wasm;
    let env2 = ExecutionEnvironment::Wasm;
    let env3 = ExecutionEnvironment::Native;

    assert_eq!(env1, env2);
    assert_ne!(env1, env3);
}

#[tokio::test]
async fn test_agent_resource_limits() {
    let limits = AgentResourceLimits::default();

    assert_eq!(limits.memory_mb, 256);
    assert_eq!(limits.cpu_percent, 10.0);
    assert_eq!(limits.timeout_seconds, 300);
    assert_eq!(limits.max_concurrent_requests, 10);
    assert_eq!(limits.storage_mb, 100);
}

#[tokio::test]
async fn test_custom_manifest_creation() {
    let mut manifest = BiomeManifestParser::generate_template();

    // Customize the manifest
    manifest.metadata.name = "custom-biome".to_string();
    manifest.agents[0].ai_provider = "anthropic".to_string();
    manifest.agents[0].model = "claude-3-sonnet".to_string();

    let yaml_content = serde_yaml::to_string(&manifest).unwrap();

    let parser = BiomeManifestParser::new();
    let parsed = parser.parse_content(&yaml_content).await.unwrap();

    assert_eq!(parsed.metadata.name, "custom-biome");
    assert_eq!(parsed.agents[0].ai_provider, "anthropic");
    assert_eq!(parsed.agents[0].model, "claude-3-sonnet");
}

#[tokio::test]
async fn test_storage_volume_configuration() {
    let volume = StorageVolume {
        name: "test-volume".to_string(),
        size: "100GB".to_string(),
        tier: "hot".to_string(),
        provisioner: "nestgate".to_string(),
        mount_path: "/data".to_string(),
        access_mode: "ReadWriteOnce".to_string(),
    };

    assert_eq!(volume.name, "test-volume");
    assert_eq!(volume.size, "100GB");
    assert_eq!(volume.tier, "hot");
    assert_eq!(volume.provisioner, "nestgate");
}

#[tokio::test]
async fn test_multiple_agents_manifest() {
    let mut manifest = BiomeManifestParser::generate_template();

    // Add a second agent
    let mut second_agent = manifest.agents[0].clone();
    second_agent.name = "ml-trainer".to_string();
    second_agent.ai_provider = "local".to_string();
    second_agent.model = "llama-3.1-8b".to_string();
    second_agent.execution_environment = ExecutionEnvironment::Container;

    manifest.agents.push(second_agent);

    let yaml_content = serde_yaml::to_string(&manifest).unwrap();

    let parser = BiomeManifestParser::new();
    let parsed = parser.parse_content(&yaml_content).await.unwrap();

    assert_eq!(parsed.agents.len(), 2);
    assert_eq!(parsed.agents[0].name, "data-analyst");
    assert_eq!(parsed.agents[1].name, "ml-trainer");
    assert_eq!(
        parsed.agents[1].execution_environment,
        ExecutionEnvironment::Container
    );
}
