// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

#![forbid(unsafe_code)]
#![allow(
    clippy::unused_self,
    clippy::unused_async,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::used_underscore_binding,
    reason = "MCP demo binary; illustrative code with relaxed lints"
)]
//! Squirrel MCP demo binary — configuration-based routing and primal coordination showcase.

use serde_json::json;
use std::collections::HashMap;
use tokio::time::{Duration, sleep};

/// Enhanced Squirrel MCP Demo with Manual Selection and Primal Routing
/// This demonstrates the new configuration-based routing capabilities
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🐿️  Squirrel MCP Demo - Enhanced AI Routing with Manual Selection & Primal Routing");
    println!("==============================================================================");

    // Initialize enhanced demo router
    let mut router = EnhancedDemoRouter::new();

    // Setup agent groups and manual routing rules
    router.setup_agent_groups().await?;
    router.setup_manual_routing_rules().await?;

    // Register AI agents with enhanced capabilities
    router.register_ai_agents().await?;

    // Setup capability endpoints (discovery pattern - resolved at runtime in production)
    router.setup_primal_endpoints().await?;

    // Demo 1: Manual Agent Selection
    println!("\n🎯 Demo 1: Manual Agent Selection & Override");
    demo_manual_selection(&mut router).await?;

    // Demo 2: Agent Groups & Priorities
    println!("\n👥 Demo 2: Agent Groups & Priority Routing");
    demo_agent_groups(&mut router).await?;

    // Demo 3: Capability-based routing
    println!("\n🏰 Demo 3: Capability-Based Routing");
    demo_primal_routing(&mut router).await?;

    // Demo 4: Configuration-based Rules
    println!("\n⚙️  Demo 4: Configuration-based Routing Rules");
    demo_config_rules(&mut router).await?;

    // Demo 5: Automatic vs Manual vs Hybrid modes
    println!("\n🔄 Demo 5: Selection Mode Comparison");
    demo_selection_modes(&mut router).await?;

    println!("\n✅ Enhanced Demo completed successfully!");
    println!("🔗 biomeOS integration features:");
    println!("   • Manual agent selection via metadata");
    println!("   • Agent groups with failover");
    println!("   • Capability-based routing (storage, compute, security)");
    println!("   • Rule-based routing configuration");
    println!("   • Priority overrides and task rejection");

    Ok(())
}

struct EnhancedDemoRouter {
    agents: HashMap<String, AgentConfig>,
    agent_groups: HashMap<String, AgentGroup>,
    manual_rules: Vec<ManualRoutingRule>,
    primal_endpoints: HashMap<String, String>,
    request_count: HashMap<String, u64>,
    selection_mode: SelectionMode,
}

#[derive(Debug, Clone)]
struct AgentConfig {
    _endpoint: String,
    capabilities: Vec<String>,
    health_status: bool,
    _response_time: Duration,
    _group: Option<String>,
}

#[derive(Debug, Clone)]
struct AgentGroup {
    _name: String,
    agents: Vec<String>,
    _priority: u32,
    _selection_strategy: String,
    _failover_groups: Vec<String>,
}

#[derive(Debug, Clone)]
struct ManualRoutingRule {
    rule_id: String,
    condition: String,
    action: String,
    _priority: u32,
    enabled: bool,
}

#[derive(Debug, Clone)]
enum SelectionMode {
    Automatic,
    _Manual,
    Hybrid,
}

impl EnhancedDemoRouter {
    fn new() -> Self {
        Self {
            agents: HashMap::new(),
            agent_groups: HashMap::new(),
            manual_rules: Vec::new(),
            primal_endpoints: HashMap::new(),
            request_count: HashMap::new(),
            selection_mode: SelectionMode::Hybrid,
        }
    }

    async fn setup_agent_groups(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // AI Processing Group
        self.agent_groups.insert(
            "ai-processing".to_string(),
            AgentGroup {
                _name: "ai-processing".to_string(),
                agents: vec!["openai-gpt4".to_string(), "claude-sonnet".to_string()],
                _priority: 1,
                _selection_strategy: "capability-based".to_string(),
                _failover_groups: vec!["local-agents".to_string()],
            },
        );

        // Local Agents Group
        self.agent_groups.insert(
            "local-agents".to_string(),
            AgentGroup {
                _name: "local-agents".to_string(),
                agents: vec!["local-ollama".to_string()],
                _priority: 2,
                _selection_strategy: "round-robin".to_string(),
                _failover_groups: vec![],
            },
        );

        // High-Performance Group
        self.agent_groups.insert(
            "high-performance".to_string(),
            AgentGroup {
                _name: "high-performance".to_string(),
                agents: vec!["openai-gpt4".to_string()],
                _priority: 0,
                _selection_strategy: "least-connections".to_string(),
                _failover_groups: vec!["ai-processing".to_string()],
            },
        );

        println!("✅ Agent groups configured: ai-processing, local-agents, high-performance");
        Ok(())
    }

    async fn setup_manual_routing_rules(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Bioinformatics tasks -> High-performance group
        self.manual_rules.push(ManualRoutingRule {
            rule_id: "bio-to-high-perf".to_string(),
            condition: "task_type=bioinformatics".to_string(),
            action: "use_group=high-performance".to_string(),
            _priority: 100,
            enabled: true,
        });

        // Security tasks -> security capability (discovered at runtime)
        self.manual_rules.push(ManualRoutingRule {
            rule_id: "security-to-capability".to_string(),
            condition: "capability=security".to_string(),
            action: "use_capability=security".to_string(),
            _priority: 90,
            enabled: true,
        });

        // Storage tasks -> storage capability (discovered at runtime)
        self.manual_rules.push(ManualRoutingRule {
            rule_id: "storage-to-capability".to_string(),
            condition: "capability=storage".to_string(),
            action: "use_capability=storage".to_string(),
            _priority: 90,
            enabled: true,
        });

        // Compute tasks -> compute capability (discovered at runtime)
        self.manual_rules.push(ManualRoutingRule {
            rule_id: "compute-to-capability".to_string(),
            condition: "capability=compute".to_string(),
            action: "use_capability=compute".to_string(),
            _priority: 90,
            enabled: true,
        });

        println!(
            "✅ Manual routing rules configured: {} rules",
            self.manual_rules.len()
        );
        Ok(())
    }

    async fn register_ai_agents(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // OpenAI GPT-4 - High capability
        self.agents.insert(
            "openai-gpt4".to_string(),
            AgentConfig {
                _endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
                capabilities: vec![
                    "reasoning".to_string(),
                    "code".to_string(),
                    "analysis".to_string(),
                    "bioinformatics".to_string(),
                ],
                health_status: true,
                _response_time: Duration::from_millis(800),
                _group: Some("ai-processing".to_string()),
            },
        );

        // Claude Sonnet - Balanced capability
        self.agents.insert(
            "claude-sonnet".to_string(),
            AgentConfig {
                _endpoint: "https://api.anthropic.com/v1/messages".to_string(),
                capabilities: vec![
                    "reasoning".to_string(),
                    "writing".to_string(),
                    "analysis".to_string(),
                ],
                health_status: true,
                _response_time: Duration::from_millis(600),
                _group: Some("ai-processing".to_string()),
            },
        );

        // Local Ollama - Privacy-focused
        self.agents.insert(
            "local-ollama".to_string(),
            AgentConfig {
                _endpoint: "http://localhost:11434/api/generate".to_string(),
                capabilities: vec![
                    "chat".to_string(),
                    "local".to_string(),
                    "private".to_string(),
                ],
                health_status: true,
                _response_time: Duration::from_millis(2000),
                _group: Some("local-agents".to_string()),
            },
        );

        println!("✅ AI agents registered: openai-gpt4, claude-sonnet, local-ollama");
        Ok(())
    }

    async fn setup_primal_endpoints(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // DEMO: Placeholder discovery pattern. In production, endpoints are discovered
        // at runtime via capability-based discovery (Songbird/socket-registry).
        // These capability keys demonstrate the PATTERN - actual URLs come from discovery.
        self.primal_endpoints
            .insert("storage".to_string(), "discovered://storage".to_string()); // Storage capability - resolved via discovery
        self.primal_endpoints
            .insert("compute".to_string(), "discovered://compute".to_string()); // Compute capability
        self.primal_endpoints
            .insert("security".to_string(), "discovered://security".to_string()); // Security capability
        self.primal_endpoints.insert(
            "service-mesh".to_string(),
            "discovered://service-mesh".to_string(),
        ); // Service mesh capability

        println!("✅ Primal endpoints configured (capability-based discovery pattern):");
        println!("   • storage: discovered://storage");
        println!("   • compute: discovered://compute");
        println!("   • security: discovered://security");
        println!("   • service-mesh: discovered://service-mesh");
        println!("   (In production, these are resolved at runtime via capability discovery)");
        Ok(())
    }

    async fn route_task(
        &mut self,
        task: &DemoTask,
    ) -> Result<TaskResponse, Box<dyn std::error::Error>> {
        // Enhanced routing with manual selection support
        let selected_destination = self.select_destination(task).await?;

        // Simulate routing
        let response = self.simulate_routing(&selected_destination, task).await?;

        // Update metrics
        *self
            .request_count
            .entry(selected_destination.clone())
            .or_insert(0) += 1;

        Ok(TaskResponse {
            destination: selected_destination,
            _response: response,
            _execution_time: Duration::from_millis(rand::random::<u64>() % 500 + 100),
            routing_method: self.get_routing_method(task),
        })
    }

    async fn select_destination(
        &self,
        task: &DemoTask,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // 1. Check for manual override in task metadata
        if let Some(preferred_agent) = task.metadata.get("preferred_agent")
            && self.agents.contains_key(preferred_agent)
        {
            return Ok(format!("agent:{preferred_agent}"));
        }

        if let Some(preferred_group) = task.metadata.get("preferred_group")
            && self.agent_groups.contains_key(preferred_group)
        {
            let group = self
                .agent_groups
                .get(preferred_group)
                .unwrap_or_else(|| unreachable!("contains_key checked above"));
            if !group.agents.is_empty() {
                return Ok(format!("group:{preferred_group}"));
            }
        }

        // Capability-based routing: target_primal or target_capability
        if let Some(target) = task
            .metadata
            .get("target_primal")
            .or_else(|| task.metadata.get("target_capability"))
            && self.primal_endpoints.contains_key(target)
        {
            return Ok(format!("primal:{target}"));
        }

        // 2. Check manual routing rules
        for rule in &self.manual_rules {
            if !rule.enabled {
                continue;
            }

            if self.matches_rule_condition(&rule.condition, task) {
                return Ok(self.apply_rule_action(&rule.action));
            }
        }

        // 3. Automatic selection based on capabilities
        let mut best_agent = None;
        let mut best_score = 0.0;

        for (agent_name, config) in &self.agents {
            if !config.health_status {
                continue;
            }

            let mut score = 0.0;

            // Capability matching
            for required_cap in &task.required_capabilities {
                if config.capabilities.contains(required_cap) {
                    score += 1.0;
                }
            }

            // Load balancing
            let usage = self.request_count.get(agent_name).unwrap_or(&0);
            score += 1.0 / (usage + 1) as f64;

            if score > best_score {
                best_score = score;
                best_agent = Some(agent_name.clone());
            }
        }

        best_agent
            .map(|agent| format!("agent:{agent}"))
            .ok_or_else(|| "No suitable destination found".into())
    }

    fn matches_rule_condition(&self, condition: &str, task: &DemoTask) -> bool {
        if condition.starts_with("task_type=") {
            let task_type = condition
                .strip_prefix("task_type=")
                .unwrap_or_else(|| unreachable!("starts_with checked above"));
            return task
                .metadata
                .get("task_type")
                .is_some_and(|t| t == task_type);
        }

        if condition.starts_with("capability=") {
            let capability = condition
                .strip_prefix("capability=")
                .unwrap_or_else(|| unreachable!("starts_with checked above"));
            return task.required_capabilities.contains(&capability.to_string());
        }

        false
    }

    fn apply_rule_action(&self, action: &str) -> String {
        if action.starts_with("use_group=") {
            let group_name = action
                .strip_prefix("use_group=")
                .unwrap_or_else(|| unreachable!("starts_with checked above"));
            return format!("group:{group_name}");
        }

        // use_primal= or use_capability= - both resolve to capability key for discovery
        if action.starts_with("use_primal=") || action.starts_with("use_capability=") {
            let prefix = if action.starts_with("use_primal=") {
                "use_primal="
            } else {
                "use_capability="
            };
            let capability_key = action
                .strip_prefix(prefix)
                .unwrap_or_else(|| unreachable!("starts_with checked above"));
            return format!("primal:{capability_key}");
        }

        if action.starts_with("use_agent=") {
            let agent_name = action
                .strip_prefix("use_agent=")
                .unwrap_or_else(|| unreachable!("starts_with checked above"));
            return format!("agent:{agent_name}");
        }

        "fallback".to_string()
    }

    async fn simulate_routing(
        &self,
        destination: &str,
        task: &DemoTask,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        // Simulate different routing latencies
        let latency = if destination.starts_with("primal:") {
            Duration::from_millis(300) // Cross-primal latency
        } else if destination.starts_with("group:") {
            Duration::from_millis(150) // Group selection latency
        } else {
            Duration::from_millis(100) // Direct agent latency
        };

        sleep(latency).await;

        Ok(json!({
            "destination": destination,
            "task_id": task.id,
            "result": format!("Processed '{}' via {}", task.prompt, destination),
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "routing_info": {
                "method": self.get_routing_method(task),
                "latency_ms": latency.as_millis(),
                "capabilities_matched": self.count_capability_matches(task),
                "load_factor": self.calculate_load_factor(destination)
            }
        }))
    }

    fn get_routing_method(&self, task: &DemoTask) -> String {
        if task.metadata.contains_key("preferred_agent")
            || task.metadata.contains_key("preferred_group")
            || task.metadata.contains_key("target_primal")
            || task.metadata.contains_key("target_capability")
        {
            return "manual_override".to_string();
        }

        for rule in &self.manual_rules {
            if rule.enabled && self.matches_rule_condition(&rule.condition, task) {
                return format!("rule:{}", rule.rule_id);
            }
        }

        "automatic".to_string()
    }

    const fn count_capability_matches(&self, task: &DemoTask) -> u32 {
        // Simple capability matching simulation
        task.required_capabilities.len() as u32
    }

    fn calculate_load_factor(&self, destination: &str) -> f64 {
        // Simple load factor calculation
        let usage = if destination.starts_with("agent:") {
            let agent_name = destination
                .strip_prefix("agent:")
                .unwrap_or_else(|| unreachable!("starts_with checked above"));
            self.request_count.get(agent_name).unwrap_or(&0)
        } else {
            &0
        };

        *usage as f64 / 10.0
    }
}

#[derive(Debug, Clone)]
struct DemoTask {
    id: String,
    prompt: String,
    required_capabilities: Vec<String>,
    _priority: TaskPriority,
    metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
enum TaskPriority {
    High,
    Normal,
    _Low,
}

#[derive(Debug)]
struct TaskResponse {
    destination: String,
    _response: serde_json::Value,
    _execution_time: Duration,
    routing_method: String,
}

async fn demo_manual_selection(
    router: &mut EnhancedDemoRouter,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing manual agent selection and overrides...");

    let mut task = DemoTask {
        id: "task-manual-1".to_string(),
        prompt: "Analyze protein structure".to_string(),
        required_capabilities: vec!["analysis".to_string()],
        _priority: TaskPriority::High,
        metadata: HashMap::new(),
    };

    // Test 1: Manual agent selection
    task.metadata
        .insert("preferred_agent".to_string(), "openai-gpt4".to_string());
    let response = router.route_task(&task).await?;
    println!(
        "✅ Manual agent selection: {} -> {}",
        task.prompt, response.destination
    );

    // Test 2: Manual group selection
    task.metadata.clear();
    task.metadata
        .insert("preferred_group".to_string(), "local-agents".to_string());
    let response = router.route_task(&task).await?;
    println!(
        "✅ Manual group selection: {} -> {}",
        task.prompt, response.destination
    );

    // Test 3: Manual capability selection (discovered at runtime)
    task.metadata.clear();
    task.metadata
        .insert("target_capability".to_string(), "compute".to_string());
    let response = router.route_task(&task).await?;
    println!(
        "✅ Manual capability selection: {} -> {}",
        task.prompt, response.destination
    );

    Ok(())
}

async fn demo_agent_groups(
    router: &mut EnhancedDemoRouter,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing agent groups and priority routing...");

    let tasks = vec![
        DemoTask {
            id: "task-group-1".to_string(),
            prompt: "High-priority bioinformatics analysis".to_string(),
            required_capabilities: vec!["bioinformatics".to_string()],
            _priority: TaskPriority::High,
            metadata: {
                let mut map = HashMap::new();
                map.insert("task_type".to_string(), "bioinformatics".to_string());
                map
            },
        },
        DemoTask {
            id: "task-group-2".to_string(),
            prompt: "General text processing".to_string(),
            required_capabilities: vec!["chat".to_string()],
            _priority: TaskPriority::Normal,
            metadata: HashMap::new(),
        },
    ];

    for task in tasks {
        let response = router.route_task(&task).await?;
        println!(
            "✅ Group routing: {} -> {} (method: {})",
            task.prompt, response.destination, response.routing_method
        );
    }

    Ok(())
}

async fn demo_primal_routing(
    router: &mut EnhancedDemoRouter,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing capability-based routing (discovery pattern)...");

    let tasks = vec![
        DemoTask {
            id: "task-primal-1".to_string(),
            prompt: "Store patient data securely".to_string(),
            required_capabilities: vec!["storage".to_string()],
            _priority: TaskPriority::High,
            metadata: HashMap::new(),
        },
        DemoTask {
            id: "task-primal-2".to_string(),
            prompt: "Run complex bioinformatics calculation".to_string(),
            required_capabilities: vec!["compute".to_string()],
            _priority: TaskPriority::High,
            metadata: HashMap::new(),
        },
        DemoTask {
            id: "task-primal-3".to_string(),
            prompt: "Encrypt sensitive research data".to_string(),
            required_capabilities: vec!["security".to_string()],
            _priority: TaskPriority::High,
            metadata: HashMap::new(),
        },
    ];

    for task in tasks {
        let response = router.route_task(&task).await?;
        println!(
            "✅ Primal routing: {} -> {} (method: {})",
            task.prompt, response.destination, response.routing_method
        );
    }

    Ok(())
}

async fn demo_config_rules(
    router: &mut EnhancedDemoRouter,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing configuration-based routing rules...");

    // Display configured rules
    println!("Active routing rules:");
    for rule in &router.manual_rules {
        println!(
            "  • {} (priority: {}): {} -> {}",
            rule.rule_id, rule._priority, rule.condition, rule.action
        );
    }

    // Test rule matching
    let task = DemoTask {
        id: "task-rule-1".to_string(),
        prompt: "Bioinformatics protein folding analysis".to_string(),
        required_capabilities: vec!["bioinformatics".to_string()],
        _priority: TaskPriority::High,
        metadata: {
            let mut map = HashMap::new();
            map.insert("task_type".to_string(), "bioinformatics".to_string());
            map
        },
    };

    let response = router.route_task(&task).await?;
    println!(
        "✅ Rule-based routing: {} -> {} (method: {})",
        task.prompt, response.destination, response.routing_method
    );

    Ok(())
}

async fn demo_selection_modes(
    router: &mut EnhancedDemoRouter,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing different selection modes...");

    let task = DemoTask {
        id: "task-mode-1".to_string(),
        prompt: "General analysis task".to_string(),
        required_capabilities: vec!["analysis".to_string()],
        _priority: TaskPriority::Normal,
        metadata: HashMap::new(),
    };

    // Test automatic mode
    router.selection_mode = SelectionMode::Automatic;
    let response = router.route_task(&task).await?;
    println!(
        "✅ Automatic mode: {} -> {}",
        task.prompt, response.destination
    );

    // Test hybrid mode (default)
    router.selection_mode = SelectionMode::Hybrid;
    let response = router.route_task(&task).await?;
    println!(
        "✅ Hybrid mode: {} -> {}",
        task.prompt, response.destination
    );

    // Show final statistics
    println!("\n📊 Final routing statistics:");
    for (destination, count) in &router.request_count {
        println!("  • {destination}: {count} requests");
    }

    Ok(())
}
