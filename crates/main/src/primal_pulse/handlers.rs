//! Handler implementations for PrimalPulse tools
//!
//! These handlers implement the actual logic for each tool, using Squirrel's
//! multi-provider AI routing to execute the analysis.

use crate::api::ai::router::AiRouter;
use crate::api::ai::types::{TextGenerationRequest, UniversalAiResponse};
use crate::error::PrimalError;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;
use std::sync::Arc;
use tracing::info;

/// Input for primal.analyze
#[derive(Debug, Deserialize)]
pub struct PrimalAnalyzeInput {
    primal_path: String,
    #[serde(default = "default_depth")]
    depth: String,
}

fn default_depth() -> String {
    "standard".to_string()
}

/// Output for primal.analyze
#[derive(Debug, Serialize)]
pub struct PrimalAnalyzeOutput {
    primal_name: String,
    grade: String,
    architecture_pattern: String,
    capabilities: Vec<String>,
    dependencies: Vec<String>,
    hardcoding_issues: usize,
    evolution_opportunities: Vec<String>,
}

/// Handle primal.analyze action
pub async fn handle_primal_analyze(
    input: Value,
    router: Arc<AiRouter>,
    constraints: Vec<String>,
) -> Result<UniversalAiResponse, PrimalError> {
    let params: PrimalAnalyzeInput = serde_json::from_value(input)
        .map_err(|e| PrimalError::ValidationError(format!("Invalid analyze input: {}", e)))?;

    info!("🔍 Analyzing primal at: {}", params.primal_path);

    // Read primal structure (simplified - just check key files exist)
    let path = Path::new(&params.primal_path);
    if !path.exists() {
        return Err(PrimalError::NotFoundError(format!(
            "Primal path not found: {}",
            params.primal_path
        )));
    }

    // Extract primal name from path
    let primal_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    // Read Cargo.toml if it exists (to get real info)
    let cargo_toml_path = path.join("Cargo.toml");
    let has_cargo_toml = cargo_toml_path.exists();

    // Build analysis prompt for AI
    let analysis_prompt = format!(
        r#"Analyze this ecoPrimals primal structure and provide insights:

Primal Name: {}
Path: {}
Has Cargo.toml: {}
Analysis Depth: {}

Please analyze and provide:
1. Architecture pattern (infant_primal, legacy, monolithic, etc.)
2. Key capabilities this primal provides
3. Likely dependencies on other primals
4. Estimate hardcoding issues (0-100 scale)
5. Evolution opportunities for improvement

Return as structured analysis focusing on ecoPrimals TRUE PRIMAL principles:
- Zero hardcoding (capability-based discovery)
- Self-knowledge only (each primal knows only itself)
- Unix sockets for IPC
- JSON-RPC 2.0 protocol

Format response as concise bullet points."#,
        primal_name, params.primal_path, has_cargo_toml, params.depth
    );

    // Use AI router with constraints (will use local AI if require_local)
    let ai_request = TextGenerationRequest {
        prompt: analysis_prompt,
        system: None,
        max_tokens: 500,
        temperature: 0.3, // Lower temperature for more factual analysis
        model: None,
        constraints: vec![],
        params: std::collections::HashMap::new(),
    };

    // For now, use generate_text directly (constraints will be added later)
    let ai_response = router
        .generate_text(ai_request, None)
        .await
        .map_err(|e| PrimalError::Internal(format!("AI analysis failed: {}", e)))?;

    // Parse AI response into structured output (simplified - real impl would parse properly)
    let analysis_text = ai_response.text.clone();

    // Create structured response (for demo, using heuristics)
    let output = PrimalAnalyzeOutput {
        primal_name: primal_name.clone(),
        grade: if has_cargo_toml { "A-" } else { "B" }.to_string(),
        architecture_pattern: if analysis_text.contains("infant") {
            "infant_primal".to_string()
        } else {
            "standard".to_string()
        },
        capabilities: extract_capabilities(&analysis_text),
        dependencies: extract_dependencies(&analysis_text),
        hardcoding_issues: estimate_hardcoding_from_text(&analysis_text),
        evolution_opportunities: extract_opportunities(&analysis_text),
    };

    let response = UniversalAiResponse {
        action: "primal.analyze".to_string(),
        output: serde_json::to_value(&output).unwrap_or_default(),
        metadata: crate::api::ai::types::ResponseMetadata {
            provider_id: ai_response.provider_id.clone(),
            provider_name: ai_response.model.clone(),
            cost_usd: ai_response.cost_usd,
            latency_ms: ai_response.latency_ms,
            timestamp: chrono::Utc::now(),
            extras: std::collections::HashMap::new(),
        },
    };

    info!(
        "✅ Analysis complete for: {} (Grade: {})",
        primal_name, output.grade
    );
    Ok(response)
}

/// Input for primal.audit_hardcoding
#[derive(Debug, Deserialize)]
pub struct PrimalAuditInput {
    primal_path: String,
    #[serde(default = "default_check_types")]
    check_types: Vec<String>,
}

fn default_check_types() -> Vec<String> {
    vec![
        "primal_names".to_string(),
        "ips".to_string(),
        "ports".to_string(),
        "vendors".to_string(),
    ]
}

/// Output for primal.audit_hardcoding
#[derive(Debug, Serialize)]
pub struct PrimalAuditOutput {
    total_violations: usize,
    by_type: serde_json::Map<String, serde_json::Value>,
    critical_files: Vec<String>,
    suggested_fixes: Vec<String>,
    grade: String,
    evolution_path: String,
}

/// Handle primal.audit_hardcoding action
pub async fn handle_primal_audit_hardcoding(
    input: Value,
    router: Arc<AiRouter>,
    constraints: Vec<String>,
) -> Result<UniversalAiResponse, PrimalError> {
    let params: PrimalAuditInput = serde_json::from_value(input)
        .map_err(|e| PrimalError::ValidationError(format!("Invalid audit input: {}", e)))?;

    info!(
        "🔍 Auditing hardcoding violations at: {}",
        params.primal_path
    );

    let path = Path::new(&params.primal_path);
    if !path.exists() {
        return Err(PrimalError::ValidationError(format!(
            "Primal path not found: {}",
            params.primal_path
        )));
    }

    // Build audit prompt
    let audit_prompt = format!(
        r#"Audit this ecoPrimals primal for TRUE PRIMAL hardcoding violations:

Path: {}
Check Types: {}

Look for violations of:
1. Hardcoded primal names (should use capability discovery)
2. Hardcoded IPs (should use environment/discovery)
3. Hardcoded ports (should use dynamic allocation)
4. Hardcoded vendor names (Consul, k8s, etc. - should use agnostic traits)

For each violation type, estimate:
- Total count
- Critical files
- Suggested fixes

Return structured analysis focusing on TRUE PRIMAL compliance."#,
        params.primal_path,
        params.check_types.join(", ")
    );

    let ai_request = TextGenerationRequest {
        prompt: audit_prompt,
        system: None,
        max_tokens: 600,
        temperature: 0.2, // Very factual
        model: None,
        constraints: vec![],
        params: std::collections::HashMap::new(),
    };

    // For now, use generate_text directly (constraints will be added later)
    let ai_response = router
        .generate_text(ai_request, None)
        .await
        .map_err(|e| PrimalError::OperationFailed(format!("AI audit failed: {}", e)))?;

    // Parse response (simplified)
    let audit_text = ai_response.text.clone();

    let total_violations = estimate_total_violations(&audit_text);
    let grade = calculate_grade(total_violations);

    let mut by_type = serde_json::Map::new();
    for check_type in &params.check_types {
        by_type.insert(
            check_type.clone(),
            json!(estimate_violations_by_type(&audit_text, check_type)),
        );
    }

    let output = PrimalAuditOutput {
        total_violations,
        by_type,
        critical_files: extract_critical_files(&audit_text),
        suggested_fixes: extract_suggested_fixes(&audit_text),
        grade: grade.clone(),
        evolution_path: "See CAPABILITY_INTEGRATION_TEMPLATE.md".to_string(),
    };

    let response = UniversalAiResponse {
        action: "primal.audit_hardcoding".to_string(),
        output: serde_json::to_value(&output).unwrap_or_default(),
        metadata: crate::api::ai::types::ResponseMetadata {
            provider_id: ai_response.provider_id.clone(),
            provider_name: ai_response.model.clone(),
            cost_usd: ai_response.cost_usd,
            latency_ms: ai_response.latency_ms,
            timestamp: chrono::Utc::now(),
            extras: std::collections::HashMap::new(),
        },
    };

    info!(
        "✅ Audit complete: {} violations found (Grade: {})",
        total_violations, grade
    );
    Ok(response)
}

/// Input for rootpulse.semantic_commit
#[derive(Debug, Deserialize)]
pub struct RootPulseCommitInput {
    diff: String,
    #[serde(default)]
    context: String,
}

/// Output for rootpulse.semantic_commit
#[derive(Debug, Serialize)]
pub struct RootPulseCommitOutput {
    commit_message: String,
    semantic_tags: Vec<String>,
    attribution_weight: f64,
    related_primals: Vec<String>,
    estimated_impact: String,
}

/// Handle rootpulse.semantic_commit action
pub async fn handle_rootpulse_semantic_commit(
    input: Value,
    router: Arc<AiRouter>,
    constraints: Vec<String>,
) -> Result<UniversalAiResponse, PrimalError> {
    let params: RootPulseCommitInput = serde_json::from_value(input)
        .map_err(|e| PrimalError::ValidationError(format!("Invalid commit input: {}", e)))?;

    info!("📝 Generating semantic commit message...");

    // Build commit generation prompt
    let commit_prompt = format!(
        r#"Generate a semantic commit message for this code change in an ecoPrimals primal:

Diff:
```
{}
```

Context: {}

Generate a commit message following Conventional Commits format:
- Type (feat, fix, refactor, docs, test, etc.)
- Scope (component/module)
- Description
- Body explaining what and why
- Footer with semantic impact

Also identify:
- Semantic tags (e.g., capability_evolution, zero_hardcoding, etc.)
- Attribution weight (0.0-1.0, how significant is this change)
- Related primals that might be affected
- Estimated impact (low/medium/high)

Focus on TRUE PRIMAL principles and architectural significance."#,
        params.diff, params.context
    );

    let ai_request = TextGenerationRequest {
        prompt: commit_prompt,
        system: None,
        max_tokens: 400,
        temperature: 0.5, // Balanced creativity/factuality
        model: None,
        constraints: vec![],
        params: std::collections::HashMap::new(),
    };

    // For now, use generate_text directly (constraints will be added later)
    let ai_response = router
        .generate_text(ai_request, None)
        .await
        .map_err(|e| PrimalError::OperationFailed(format!("AI commit generation failed: {}", e)))?;

    // Parse AI response
    let commit_text = ai_response.text.clone();

    let output = RootPulseCommitOutput {
        commit_message: extract_commit_message(&commit_text),
        semantic_tags: extract_semantic_tags(&commit_text),
        attribution_weight: estimate_attribution_weight(&commit_text),
        related_primals: extract_related_primals(&commit_text),
        estimated_impact: estimate_impact(&commit_text),
    };

    let response = UniversalAiResponse {
        action: "rootpulse.semantic_commit".to_string(),
        output: serde_json::to_value(&output).unwrap_or_default(),
        metadata: crate::api::ai::types::ResponseMetadata {
            provider_id: ai_response.provider_id.clone(),
            provider_name: ai_response.model.clone(),
            cost_usd: ai_response.cost_usd,
            latency_ms: ai_response.latency_ms,
            timestamp: chrono::Utc::now(),
            extras: std::collections::HashMap::new(),
        },
    };

    info!(
        "✅ Semantic commit generated (Impact: {})",
        output.estimated_impact
    );
    Ok(response)
}

// Helper functions for parsing AI responses (simplified implementations)

fn extract_capabilities(text: &str) -> Vec<String> {
    // Simple keyword extraction
    let mut caps = Vec::new();
    if text.contains("ai") || text.contains("AI") {
        caps.push("ai_routing".to_string());
    }
    if text.contains("tool") {
        caps.push("tool_orchestration".to_string());
    }
    if text.contains("mcp") || text.contains("MCP") {
        caps.push("mcp".to_string());
    }
    caps
}

fn extract_dependencies(text: &str) -> Vec<String> {
    let mut deps = Vec::new();
    for primal in &["songbird", "beardog", "nestgate", "toadstool"] {
        if text.to_lowercase().contains(primal) {
            deps.push(primal.to_string());
        }
    }
    deps
}

fn estimate_hardcoding_from_text(text: &str) -> usize {
    if text.contains("zero") || text.contains("no hardcoding") {
        0
    } else if text.contains("minimal") {
        5
    } else if text.contains("some") {
        15
    } else {
        10
    }
}

fn extract_opportunities(_text: &str) -> Vec<String> {
    vec![
        "Consider integrating RootPulse for version tracking".to_string(),
        "Neural API coordination could optimize performance".to_string(),
    ]
}

fn estimate_total_violations(text: &str) -> usize {
    // Count numeric mentions
    text.split_whitespace()
        .filter_map(|w| w.parse::<usize>().ok())
        .sum::<usize>()
        .min(100) // Cap at 100
}

fn calculate_grade(violations: usize) -> String {
    match violations {
        0 => "A+".to_string(),
        1..=5 => "A".to_string(),
        6..=15 => "B+".to_string(),
        16..=30 => "B".to_string(),
        _ => "C".to_string(),
    }
}

fn estimate_violations_by_type(text: &str, check_type: &str) -> usize {
    if text.to_lowercase().contains(check_type) {
        5 // Simple heuristic
    } else {
        0
    }
}

fn extract_critical_files(_text: &str) -> Vec<String> {
    vec!["See AI analysis above".to_string()]
}

fn extract_suggested_fixes(_text: &str) -> Vec<String> {
    vec![
        "Replace hardcoded names with capability discovery".to_string(),
        "Use environment variables for configuration".to_string(),
        "Implement UniversalAdapterV2 for discovery".to_string(),
    ]
}

fn extract_commit_message(text: &str) -> String {
    // Extract first few lines as commit message
    text.lines()
        .take(5)
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

fn extract_semantic_tags(text: &str) -> Vec<String> {
    let mut tags = Vec::new();
    if text.contains("capability") {
        tags.push("capability_evolution".to_string());
    }
    if text.contains("hardcoding") || text.contains("zero") {
        tags.push("zero_hardcoding".to_string());
    }
    if text.contains("infrastructure") {
        tags.push("infrastructure".to_string());
    }
    tags
}

fn estimate_attribution_weight(text: &str) -> f64 {
    if text.contains("major") || text.contains("significant") {
        0.85
    } else if text.contains("minor") {
        0.3
    } else {
        0.5
    }
}

fn extract_related_primals(text: &str) -> Vec<String> {
    let mut primals = Vec::new();
    for primal in &["songbird", "beardog", "nestgate", "squirrel", "toadstool"] {
        if text.to_lowercase().contains(primal) {
            primals.push(primal.to_string());
        }
    }
    primals
}

fn estimate_impact(text: &str) -> String {
    if text.contains("high") || text.contains("major") {
        "high".to_string()
    } else if text.contains("low") || text.contains("minor") {
        "low".to_string()
    } else {
        "medium".to_string()
    }
}
