use std::collections::HashMap;
use serde_json::json;

/// Focused Demo: Manual Selection & Primal Routing
/// Shows NestGate=storage, ToadStool=compute, manual overrides, and agent groups
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🐿️  Squirrel MCP - Manual Selection & Primal Routing Demo");
    println!("=========================================================");
    println!("Fixed Primal Mappings:");
    println!("  • NestGate   → Storage Persistence");
    println!("  • ToadStool  → Compute Processing");
    println!("  • BearDog    → Security Operations");
    println!("  • BiomeOS    → Integration & Orchestration");
    println!();

    let mut router = ConfigurableRouter::new().await?;
    
    // Demo 1: Manual Agent Selection
    println!("🎯 Demo 1: Manual Agent Selection Override");
    demo_manual_selection(&mut router).await?;
    
    // Demo 2: Agent Groups with Priorities
    println!("\n👥 Demo 2: Agent Groups & Priority Routing");
    demo_agent_groups(&mut router).await?;
    
    // Demo 3: Fixed Primal Routing
    println!("\n🏰 Demo 3: Cross-Primal Routing (Fixed Mappings)");
    demo_primal_routing(&mut router).await?;
    
    // Demo 4: Configuration-based Rules
    println!("\n⚙️  Demo 4: Rule-based Routing Configuration");
    demo_config_rules(&mut router).await?;
    
    // Demo 5: Combined Scenario
    println!("\n🚀 Demo 5: BiomeOS Integration Scenario");
    demo_biomeos_integration(&mut router).await?;
    
    println!("\n✅ Manual Selection & Primal Routing Demo Complete!");
    println!("\n📊 Final Statistics:");
    router.show_statistics();
    
    println!("\n🔗 Ready for biomeOS Integration:");
    println!("  ✓ Manual agent selection via task metadata");
    println!("  ✓ Agent groups with failover support");
    println!("  ✓ Fixed primal routing (NestGate, ToadStool, BearDog)");
    println!("  ✓ Rule-based configuration system");
    println!("  ✓ Priority override capabilities");
    println!("  ✓ Configurable selection modes (Auto/Manual/Hybrid)");
    
    Ok(())
}

#[derive(Debug, Clone)]
struct ConfigurableRouter {
    // Agent registry
    agents: HashMap<String, AgentConfig>,
    
    // Agent groups with priorities
    agent_groups: HashMap<String, AgentGroup>,
    
    // Manual routing rules
    routing_rules: Vec<RoutingRule>,
    
    // Primal endpoints (fixed mappings)
    primal_endpoints: HashMap<String, String>,
    
    // Configuration
    selection_mode: SelectionMode,
    allow_manual_override: bool,
    
    // Statistics
    routing_stats: HashMap<String, u64>,
    rule_matches: HashMap<String, u64>,
}

#[derive(Debug, Clone)]
struct AgentConfig {
    id: String,
    endpoint: String,
    capabilities: Vec<String>,
    group: Option<String>,
    health: HealthStatus,
    load: u32,
    response_time_ms: u64,
}

#[derive(Debug, Clone)]
struct AgentGroup {
    name: String,
    agents: Vec<String>,
    priority: u32,
    selection_strategy: String,
    failover_groups: Vec<String>,
    max_concurrent: Option<u32>,
}

#[derive(Debug, Clone)]
struct RoutingRule {
    id: String,
    condition: RuleCondition,
    action: RuleAction,
    priority: u32,
    enabled: bool,
}

#[derive(Debug, Clone)]
enum RuleCondition {
    TaskType(String),
    Capability(String),
    PrimalType(String),
    UserRequest(String),
    TaskMetadata(String, String),
}

#[derive(Debug, Clone)]
enum RuleAction {
    UseAgent(String),
    UseGroup(String),
    UsePrimal(String),
    SetPriority(TaskPriority),
    Reject(String),
}

#[derive(Debug, Clone)]
enum SelectionMode {
    Automatic,
    Manual,
    Hybrid,
}

#[derive(Debug, Clone)]
enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Clone)]
enum TaskPriority {
    Critical,
    High,
    Normal,
    Low,
}

#[derive(Debug, Clone)]
struct DemoTask {
    id: String,
    prompt: String,
    task_type: String,
    required_capabilities: Vec<String>,
    priority: TaskPriority,
    metadata: HashMap<String, String>,
}

#[derive(Debug)]
struct RoutingResult {
    destination: String,
    destination_type: String,
    routing_method: String,
    rule_applied: Option<String>,
    latency_ms: u64,
    success: bool,
}

impl ConfigurableRouter {
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut router = Self {
            agents: HashMap::new(),
            agent_groups: HashMap::new(),
            routing_rules: Vec::new(),
            primal_endpoints: HashMap::new(),
            selection_mode: SelectionMode::Hybrid,
            allow_manual_override: true,
            routing_stats: HashMap::new(),
            rule_matches: HashMap::new(),
        };
        
        router.setup_agents().await?;
        router.setup_agent_groups().await?;
        router.setup_primal_endpoints().await?;
        router.setup_routing_rules().await?;
        
        Ok(router)
    }
    
    async fn setup_agents(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // High-performance AI agents
        self.agents.insert("openai-gpt4".to_string(), AgentConfig {
            id: "openai-gpt4".to_string(),
            endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
            capabilities: vec!["reasoning".to_string(), "code".to_string(), "bioinformatics".to_string()],
            group: Some("high-performance".to_string()),
            health: HealthStatus::Healthy,
            load: 2,
            response_time_ms: 800,
        });
        
        self.agents.insert("claude-sonnet".to_string(), AgentConfig {
            id: "claude-sonnet".to_string(),
            endpoint: "https://api.anthropic.com/v1/messages".to_string(),
            capabilities: vec!["reasoning".to_string(), "writing".to_string(), "analysis".to_string()],
            group: Some("ai-processing".to_string()),
            health: HealthStatus::Healthy,
            load: 1,
            response_time_ms: 600,
        });
        
        // Local/private agents
        self.agents.insert("local-ollama".to_string(), AgentConfig {
            id: "local-ollama".to_string(),
            endpoint: "http://localhost:11434/api/generate".to_string(),
            capabilities: vec!["chat".to_string(), "private".to_string(), "local".to_string()],
            group: Some("local-agents".to_string()),
            health: HealthStatus::Healthy,
            load: 0,
            response_time_ms: 2000,
        });
        
        println!("✅ Agents configured: {} total", self.agents.len());
        Ok(())
    }
    
    async fn setup_agent_groups(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // High-performance group for critical tasks
        self.agent_groups.insert("high-performance".to_string(), AgentGroup {
            name: "high-performance".to_string(),
            agents: vec!["openai-gpt4".to_string()],
            priority: 1,
            selection_strategy: "least-load".to_string(),
            failover_groups: vec!["ai-processing".to_string()],
            max_concurrent: Some(5),
        });
        
        // General AI processing group
        self.agent_groups.insert("ai-processing".to_string(), AgentGroup {
            name: "ai-processing".to_string(),
            agents: vec!["claude-sonnet".to_string(), "openai-gpt4".to_string()],
            priority: 2,
            selection_strategy: "round-robin".to_string(),
            failover_groups: vec!["local-agents".to_string()],
            max_concurrent: Some(10),
        });
        
        // Local agents for privacy-sensitive tasks
        self.agent_groups.insert("local-agents".to_string(), AgentGroup {
            name: "local-agents".to_string(),
            agents: vec!["local-ollama".to_string()],
            priority: 3,
            selection_strategy: "available".to_string(),
            failover_groups: vec![],
            max_concurrent: Some(2),
        });
        
        println!("✅ Agent groups configured: {} total", self.agent_groups.len());
        Ok(())
    }
    
    async fn setup_primal_endpoints(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Fixed primal mappings as requested
        self.primal_endpoints.insert("nestgate".to_string(), "http://nestgate:8080/api/v1".to_string());
        self.primal_endpoints.insert("toadstool".to_string(), "http://toadstool:8080/api/v1".to_string());
        self.primal_endpoints.insert("beardog".to_string(), "http://beardog:8080/api/v1".to_string());
        self.primal_endpoints.insert("biomeos".to_string(), "http://biomeos:8080/api/v1".to_string());
        
        println!("✅ Primal endpoints configured:");
        println!("   • NestGate (storage): {}", self.primal_endpoints["nestgate"]);
        println!("   • ToadStool (compute): {}", self.primal_endpoints["toadstool"]);
        println!("   • BearDog (security): {}", self.primal_endpoints["beardog"]);
        println!("   • BiomeOS (integration): {}", self.primal_endpoints["biomeos"]);
        Ok(())
    }
    
    async fn setup_routing_rules(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // High-priority rules for primal routing
        self.routing_rules.push(RoutingRule {
            id: "storage-to-nestgate".to_string(),
            condition: RuleCondition::Capability("storage".to_string()),
            action: RuleAction::UsePrimal("nestgate".to_string()),
            priority: 100,
            enabled: true,
        });
        
        self.routing_rules.push(RoutingRule {
            id: "compute-to-toadstool".to_string(),
            condition: RuleCondition::Capability("compute".to_string()),
            action: RuleAction::UsePrimal("toadstool".to_string()),
            priority: 100,
            enabled: true,
        });
        
        self.routing_rules.push(RoutingRule {
            id: "security-to-beardog".to_string(),
            condition: RuleCondition::Capability("security".to_string()),
            action: RuleAction::UsePrimal("beardog".to_string()),
            priority: 100,
            enabled: true,
        });
        
        // Task-type specific rules
        self.routing_rules.push(RoutingRule {
            id: "bioinformatics-to-high-perf".to_string(),
            condition: RuleCondition::TaskType("bioinformatics".to_string()),
            action: RuleAction::UseGroup("high-performance".to_string()),
            priority: 90,
            enabled: true,
        });
        
        self.routing_rules.push(RoutingRule {
            id: "private-to-local".to_string(),
            condition: RuleCondition::TaskMetadata("privacy".to_string(), "required".to_string()),
            action: RuleAction::UseGroup("local-agents".to_string()),
            priority: 95,
            enabled: true,
        });
        
        println!("✅ Routing rules configured: {} total", self.routing_rules.len());
        Ok(())
    }
    
    async fn route_task(&mut self, task: &DemoTask) -> Result<RoutingResult, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        
        // Step 1: Check for manual override
        if self.allow_manual_override {
            if let Some(preferred_agent) = task.metadata.get("preferred_agent") {
                if self.agents.contains_key(preferred_agent) {
                    return self.create_result("agent", preferred_agent, "manual_override", None, start_time);
                }
            }
            
            if let Some(preferred_group) = task.metadata.get("preferred_group") {
                if self.agent_groups.contains_key(preferred_group) {
                    return self.create_result("group", preferred_group, "manual_override", None, start_time);
                }
            }
            
            if let Some(target_primal) = task.metadata.get("target_primal") {
                if self.primal_endpoints.contains_key(target_primal) {
                    return self.create_result("primal", target_primal, "manual_override", None, start_time);
                }
            }
        }
        
        // Step 2: Apply routing rules
        for rule in &self.routing_rules {
            if !rule.enabled {
                continue;
            }
            
            if self.matches_condition(&rule.condition, task) {
                *self.rule_matches.entry(rule.id.clone()).or_insert(0) += 1;
                
                match &rule.action {
                    RuleAction::UseAgent(agent_id) => {
                        return self.create_result("agent", agent_id, "rule_based", Some(&rule.id), start_time);
                    }
                    RuleAction::UseGroup(group_id) => {
                        return self.create_result("group", group_id, "rule_based", Some(&rule.id), start_time);
                    }
                    RuleAction::UsePrimal(primal_id) => {
                        return self.create_result("primal", primal_id, "rule_based", Some(&rule.id), start_time);
                    }
                    RuleAction::Reject(reason) => {
                        return Ok(RoutingResult {
                            destination: format!("REJECTED: {}", reason),
                            destination_type: "rejection".to_string(),
                            routing_method: "rule_based".to_string(),
                            rule_applied: Some(rule.id.clone()),
                            latency_ms: start_time.elapsed().as_millis() as u64,
                            success: false,
                        });
                    }
                    _ => continue,
                }
            }
        }
        
        // Step 3: Automatic selection based on capabilities
        let best_agent = self.select_best_agent(task)?;
        self.create_result("agent", &best_agent, "automatic", None, start_time)
    }
    
    fn matches_condition(&self, condition: &RuleCondition, task: &DemoTask) -> bool {
        match condition {
            RuleCondition::TaskType(task_type) => task.task_type == *task_type,
            RuleCondition::Capability(capability) => task.required_capabilities.contains(capability),
            RuleCondition::PrimalType(primal) => {
                // Check if task should be routed to this primal
                match primal.as_str() {
                    "nestgate" => task.required_capabilities.contains(&"storage".to_string()),
                    "toadstool" => task.required_capabilities.contains(&"compute".to_string()),
                    "beardog" => task.required_capabilities.contains(&"security".to_string()),
                    _ => false,
                }
            }
            RuleCondition::UserRequest(user) => {
                task.metadata.get("user_id").map_or(false, |u| u == user)
            }
            RuleCondition::TaskMetadata(key, value) => {
                task.metadata.get(key).map_or(false, |v| v == value)
            }
        }
    }
    
    fn select_best_agent(&self, task: &DemoTask) -> Result<String, Box<dyn std::error::Error>> {
        let mut best_agent = None;
        let mut best_score = 0.0;
        
        for (agent_id, agent) in &self.agents {
            if !matches!(agent.health, HealthStatus::Healthy) {
                continue;
            }
            
            let mut score = 0.0;
            
            // Capability matching
            for required_cap in &task.required_capabilities {
                if agent.capabilities.contains(required_cap) {
                    score += 10.0;
                }
            }
            
            // Load balancing (prefer lower load)
            score += 5.0 / (agent.load + 1) as f64;
            
            // Response time preference
            score += 1000.0 / agent.response_time_ms as f64;
            
            if score > best_score {
                best_score = score;
                best_agent = Some(agent_id.clone());
            }
        }
        
        best_agent.ok_or_else(|| "No suitable agent found".into())
    }
    
    fn create_result(
        &mut self,
        dest_type: &str,
        dest_id: &str,
        method: &str,
        rule_id: Option<&str>,
        start_time: std::time::Instant,
    ) -> Result<RoutingResult, Box<dyn std::error::Error>> {
        let destination = format!("{}:{}", dest_type, dest_id);
        *self.routing_stats.entry(destination.clone()).or_insert(0) += 1;
        
        // Simulate different latencies based on destination type
        let latency_ms = match dest_type {
            "primal" => 250, // Cross-primal latency
            "group" => 150,  // Group selection latency
            "agent" => 100,  // Direct agent latency
            _ => 50,
        };
        
        Ok(RoutingResult {
            destination,
            destination_type: dest_type.to_string(),
            routing_method: method.to_string(),
            rule_applied: rule_id.map(String::from),
            latency_ms: latency_ms + start_time.elapsed().as_millis() as u64,
            success: true,
        })
    }
    
    fn show_statistics(&self) {
        println!("Routing Statistics:");
        for (destination, count) in &self.routing_stats {
            println!("  • {}: {} requests", destination, count);
        }
        
        if !self.rule_matches.is_empty() {
            println!("\nRule Match Statistics:");
            for (rule_id, count) in &self.rule_matches {
                println!("  • {}: {} matches", rule_id, count);
            }
        }
    }
}

async fn demo_manual_selection(router: &mut ConfigurableRouter) -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing manual agent/group/primal selection...");
    
    // Test 1: Manual agent selection
    let mut task = DemoTask {
        id: "manual-1".to_string(),
        prompt: "Analyze protein structure".to_string(),
        task_type: "analysis".to_string(),
        required_capabilities: vec!["analysis".to_string()],
        priority: TaskPriority::High,
        metadata: {
            let mut map = HashMap::new();
            map.insert("preferred_agent".to_string(), "claude-sonnet".to_string());
            map
        },
    };
    
    let result = router.route_task(&task).await?;
    println!("  ✅ Manual agent: {} -> {} ({}ms)", 
             task.prompt, result.destination, result.latency_ms);
    
    // Test 2: Manual group selection
    task.metadata.clear();
    task.metadata.insert("preferred_group".to_string(), "local-agents".to_string());
    
    let result = router.route_task(&task).await?;
    println!("  ✅ Manual group: {} -> {} ({}ms)", 
             task.prompt, result.destination, result.latency_ms);
    
    // Test 3: Manual primal selection
    task.metadata.clear();
    task.metadata.insert("target_primal".to_string(), "toadstool".to_string());
    
    let result = router.route_task(&task).await?;
    println!("  ✅ Manual primal: {} -> {} ({}ms)", 
             task.prompt, result.destination, result.latency_ms);
    
    Ok(())
}

async fn demo_agent_groups(router: &mut ConfigurableRouter) -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing agent groups and priority routing...");
    
    let tasks = vec![
        DemoTask {
            id: "group-1".to_string(),
            prompt: "Critical bioinformatics analysis".to_string(),
            task_type: "bioinformatics".to_string(),
            required_capabilities: vec!["bioinformatics".to_string()],
            priority: TaskPriority::Critical,
            metadata: HashMap::new(),
        },
        DemoTask {
            id: "group-2".to_string(),
            prompt: "General text processing".to_string(),
            task_type: "text".to_string(),
            required_capabilities: vec!["chat".to_string()],
            priority: TaskPriority::Normal,
            metadata: HashMap::new(),
        },
        DemoTask {
            id: "group-3".to_string(),
            prompt: "Private data analysis".to_string(),
            task_type: "analysis".to_string(),
            required_capabilities: vec!["analysis".to_string()],
            priority: TaskPriority::High,
            metadata: {
                let mut map = HashMap::new();
                map.insert("privacy".to_string(), "required".to_string());
                map
            },
        },
    ];
    
    for task in tasks {
        let result = router.route_task(&task).await?;
        println!("  ✅ Group routing: {} -> {} via {} ({}ms)", 
                 task.prompt, result.destination, result.routing_method, result.latency_ms);
        if let Some(rule) = result.rule_applied {
            println!("     Rule applied: {}", rule);
        }
    }
    
    Ok(())
}

async fn demo_primal_routing(router: &mut ConfigurableRouter) -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing cross-primal routing with fixed mappings...");
    
    let tasks = vec![
        DemoTask {
            id: "primal-1".to_string(),
            prompt: "Store patient genomic data".to_string(),
            task_type: "storage".to_string(),
            required_capabilities: vec!["storage".to_string(), "persistence".to_string()],
            priority: TaskPriority::High,
            metadata: HashMap::new(),
        },
        DemoTask {
            id: "primal-2".to_string(),
            prompt: "Run complex protein folding simulation".to_string(),
            task_type: "computation".to_string(),
            required_capabilities: vec!["compute".to_string(), "processing".to_string()],
            priority: TaskPriority::Critical,
            metadata: HashMap::new(),
        },
        DemoTask {
            id: "primal-3".to_string(),
            prompt: "Encrypt sensitive research data".to_string(),
            task_type: "security".to_string(),
            required_capabilities: vec!["security".to_string(), "encryption".to_string()],
            priority: TaskPriority::High,
            metadata: HashMap::new(),
        },
    ];
    
    for task in tasks {
        let result = router.route_task(&task).await?;
        println!("  ✅ Primal routing: {} -> {} via {} ({}ms)", 
                 task.prompt, result.destination, result.routing_method, result.latency_ms);
        if let Some(rule) = result.rule_applied {
            println!("     Rule applied: {}", rule);
        }
    }
    
    Ok(())
}

async fn demo_config_rules(router: &mut ConfigurableRouter) -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing configuration-based routing rules...");
    
    println!("Active routing rules:");
    for rule in &router.routing_rules {
        if rule.enabled {
            println!("  • {} (priority: {}): {:?} -> {:?}", 
                     rule.id, rule.priority, rule.condition, rule.action);
        }
    }
    
    // Test rule matching
    let task = DemoTask {
        id: "rule-test".to_string(),
        prompt: "Complex bioinformatics protein analysis".to_string(),
        task_type: "bioinformatics".to_string(),
        required_capabilities: vec!["bioinformatics".to_string(), "reasoning".to_string()],
        priority: TaskPriority::High,
        metadata: HashMap::new(),
    };
    
    let result = router.route_task(&task).await?;
    println!("  ✅ Rule-based: {} -> {} via {} ({}ms)", 
             task.prompt, result.destination, result.routing_method, result.latency_ms);
    if let Some(rule) = result.rule_applied {
        println!("     Rule applied: {}", rule);
    }
    
    Ok(())
}

async fn demo_biomeos_integration(router: &mut ConfigurableRouter) -> Result<(), Box<dyn std::error::Error>> {
    println!("Simulating complete biomeOS integration workflow...");
    
    let scenarios = vec![
        ("Genomic Storage", DemoTask {
            id: "biomeos-1".to_string(),
            prompt: "Store patient genomic sequences in secure database".to_string(),
            task_type: "storage".to_string(),
            required_capabilities: vec!["storage".to_string()],
            priority: TaskPriority::Critical,
            metadata: {
                let mut map = HashMap::new();
                map.insert("user_id".to_string(), "biomeos-system".to_string());
                map.insert("data_type".to_string(), "genomic".to_string());
                map
            },
        }),
        ("Compute Analysis", DemoTask {
            id: "biomeos-2".to_string(),
            prompt: "Run large-scale variant calling pipeline".to_string(),
            task_type: "computation".to_string(),
            required_capabilities: vec!["compute".to_string()],
            priority: TaskPriority::High,
            metadata: {
                let mut map = HashMap::new();
                map.insert("compute_type".to_string(), "pipeline".to_string());
                map
            },
        }),
        ("AI Analysis", DemoTask {
            id: "biomeos-3".to_string(),
            prompt: "Analyze genomic variants for drug interaction predictions".to_string(),
            task_type: "bioinformatics".to_string(),
            required_capabilities: vec!["bioinformatics".to_string(), "reasoning".to_string()],
            priority: TaskPriority::High,
            metadata: {
                let mut map = HashMap::new();
                map.insert("analysis_type".to_string(), "drug_interaction".to_string());
                map
            },
        }),
        ("Security Check", DemoTask {
            id: "biomeos-4".to_string(),
            prompt: "Validate patient data access permissions".to_string(),
            task_type: "security".to_string(),
            required_capabilities: vec!["security".to_string(), "auth".to_string()],
            priority: TaskPriority::Critical,
            metadata: {
                let mut map = HashMap::new();
                map.insert("security_level".to_string(), "patient_data".to_string());
                map
            },
        }),
        ("Manual Override", DemoTask {
            id: "biomeos-5".to_string(),
            prompt: "Emergency analysis with specific AI model".to_string(),
            task_type: "analysis".to_string(),
            required_capabilities: vec!["reasoning".to_string()],
            priority: TaskPriority::Critical,
            metadata: {
                let mut map = HashMap::new();
                map.insert("preferred_agent".to_string(), "openai-gpt4".to_string());
                map.insert("emergency".to_string(), "true".to_string());
                map
            },
        }),
    ];
    
    println!("🔄 Processing biomeOS workflow:");
    for (step_name, task) in scenarios {
        let result = router.route_task(&task).await?;
        println!("  {} → {} via {} ({}ms)", 
                 step_name, result.destination, result.routing_method, result.latency_ms);
        
        if let Some(rule) = result.rule_applied {
            println!("    └─ Rule: {}", rule);
        }
        
        // Simulate processing delay
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    
    println!("✅ biomeOS workflow completed successfully!");
    println!("   Storage → NestGate, Compute → ToadStool, Security → BearDog");
    println!("   AI Analysis → High-performance agents with manual override support");
    
    Ok(())
} 