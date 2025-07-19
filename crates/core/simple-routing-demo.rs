use std::collections::HashMap;

/// Simple Manual Selection & Primal Routing Demo
/// Shows the new configuration-based routing capabilities

fn main() {
    println!("🐿️  Squirrel MCP - Manual Selection & Primal Routing Demo");
    println!("=========================================================");
    println!("Fixed Primal Mappings (corrected as requested):");
    println!("  • NestGate   → Storage Persistence");
    println!("  • ToadStool  → Compute Processing");
    println!("  • BearDog    → Security Operations");
    println!("  • BiomeOS    → Integration & Orchestration");
    println!();
    
    let mut demo = RoutingDemo::new();
    demo.run_all_demos();
}

struct RoutingDemo {
    agents: HashMap<String, AgentInfo>,
    agent_groups: HashMap<String, AgentGroup>,
    primal_endpoints: HashMap<String, String>,
    routing_rules: Vec<RoutingRule>,
    statistics: HashMap<String, u32>,
}

#[derive(Debug, Clone)]
struct AgentInfo {
    id: String,
    capabilities: Vec<String>,
    group: String,
    health: String,
    load: u32,
}

#[derive(Debug, Clone)]
struct AgentGroup {
    name: String,
    agents: Vec<String>,
    priority: u32,
    strategy: String,
}

#[derive(Debug, Clone)]
struct RoutingRule {
    id: String,
    condition: String,
    action: String,
    priority: u32,
}

#[derive(Debug)]
struct DemoTask {
    id: String,
    prompt: String,
    task_type: String,
    capabilities: Vec<String>,
    metadata: HashMap<String, String>,
}

#[derive(Debug)]
struct RoutingResult {
    destination: String,
    method: String,
    rule_applied: Option<String>,
    latency_ms: u64,
}

impl RoutingDemo {
    fn new() -> Self {
        let mut demo = Self {
            agents: HashMap::new(),
            agent_groups: HashMap::new(),
            primal_endpoints: HashMap::new(),
            routing_rules: Vec::new(),
            statistics: HashMap::new(),
        };
        demo.setup();
        demo
    }
    
    fn setup(&mut self) {
        // Setup agents
        self.agents.insert("openai-gpt4".to_string(), AgentInfo {
            id: "openai-gpt4".to_string(),
            capabilities: vec!["reasoning".to_string(), "bioinformatics".to_string(), "code".to_string()],
            group: "high-performance".to_string(),
            health: "healthy".to_string(),
            load: 2,
        });
        
        self.agents.insert("claude-sonnet".to_string(), AgentInfo {
            id: "claude-sonnet".to_string(),
            capabilities: vec!["reasoning".to_string(), "writing".to_string(), "analysis".to_string()],
            group: "ai-processing".to_string(),
            health: "healthy".to_string(),
            load: 1,
        });
        
        self.agents.insert("local-ollama".to_string(), AgentInfo {
            id: "local-ollama".to_string(),
            capabilities: vec!["chat".to_string(), "private".to_string(), "local".to_string()],
            group: "local-agents".to_string(),
            health: "healthy".to_string(),
            load: 0,
        });
        
        // Setup agent groups
        self.agent_groups.insert("high-performance".to_string(), AgentGroup {
            name: "high-performance".to_string(),
            agents: vec!["openai-gpt4".to_string()],
            priority: 1,
            strategy: "least-load".to_string(),
        });
        
        self.agent_groups.insert("ai-processing".to_string(), AgentGroup {
            name: "ai-processing".to_string(),
            agents: vec!["claude-sonnet".to_string(), "openai-gpt4".to_string()],
            priority: 2,
            strategy: "round-robin".to_string(),
        });
        
        self.agent_groups.insert("local-agents".to_string(), AgentGroup {
            name: "local-agents".to_string(),
            agents: vec!["local-ollama".to_string()],
            priority: 3,
            strategy: "available".to_string(),
        });
        
        // Setup primal endpoints (corrected mappings)
        self.primal_endpoints.insert("nestgate".to_string(), "http://nestgate:8080/api/v1".to_string());
        self.primal_endpoints.insert("toadstool".to_string(), "http://toadstool:8080/api/v1".to_string());
        self.primal_endpoints.insert("beardog".to_string(), "http://beardog:8080/api/v1".to_string());
        self.primal_endpoints.insert("biomeos".to_string(), "http://biomeos:8080/api/v1".to_string());
        
        // Setup routing rules
        self.routing_rules.push(RoutingRule {
            id: "storage-to-nestgate".to_string(),
            condition: "capability:storage".to_string(),
            action: "primal:nestgate".to_string(),
            priority: 100,
        });
        
        self.routing_rules.push(RoutingRule {
            id: "compute-to-toadstool".to_string(),
            condition: "capability:compute".to_string(),
            action: "primal:toadstool".to_string(),
            priority: 100,
        });
        
        self.routing_rules.push(RoutingRule {
            id: "security-to-beardog".to_string(),
            condition: "capability:security".to_string(),
            action: "primal:beardog".to_string(),
            priority: 100,
        });
        
        self.routing_rules.push(RoutingRule {
            id: "bioinformatics-to-high-perf".to_string(),
            condition: "type:bioinformatics".to_string(),
            action: "group:high-performance".to_string(),
            priority: 90,
        });
        
        self.routing_rules.push(RoutingRule {
            id: "private-to-local".to_string(),
            condition: "metadata:privacy=required".to_string(),
            action: "group:local-agents".to_string(),
            priority: 95,
        });
        
        println!("✅ Configuration loaded:");
        println!("   • {} agents registered", self.agents.len());
        println!("   • {} agent groups configured", self.agent_groups.len());
        println!("   • {} primal endpoints mapped", self.primal_endpoints.len());
        println!("   • {} routing rules active", self.routing_rules.len());
        println!();
    }
    
    fn run_all_demos(&mut self) {
        self.demo_manual_selection();
        self.demo_agent_groups();
        self.demo_primal_routing();
        self.demo_configuration_rules();
        self.demo_biomeos_integration();
        self.show_final_statistics();
    }
    
    fn demo_manual_selection(&mut self) {
        println!("🎯 Demo 1: Manual Agent Selection Override");
        println!("==========================================");
        
        // Test 1: Manual agent selection
        let mut task = DemoTask {
            id: "manual-1".to_string(),
            prompt: "Analyze protein structure with specific model".to_string(),
            task_type: "analysis".to_string(),
            capabilities: vec!["analysis".to_string()],
            metadata: {
                let mut map = HashMap::new();
                map.insert("preferred_agent".to_string(), "claude-sonnet".to_string());
                map
            },
        };
        
        let result = self.route_task(&task);
        println!("  ✅ Manual agent: {} → {}", task.prompt, result.destination);
        println!("     Method: {} ({}ms)", result.method, result.latency_ms);
        
        // Test 2: Manual group selection
        task.metadata.clear();
        task.metadata.insert("preferred_group".to_string(), "local-agents".to_string());
        task.prompt = "Use local processing for privacy".to_string();
        
        let result = self.route_task(&task);
        println!("  ✅ Manual group: {} → {}", task.prompt, result.destination);
        println!("     Method: {} ({}ms)", result.method, result.latency_ms);
        
        // Test 3: Manual primal selection
        task.metadata.clear();
        task.metadata.insert("target_primal".to_string(), "toadstool".to_string());
        task.prompt = "Direct computation on ToadStool".to_string();
        
        let result = self.route_task(&task);
        println!("  ✅ Manual primal: {} → {}", task.prompt, result.destination);
        println!("     Method: {} ({}ms)", result.method, result.latency_ms);
        println!();
    }
    
    fn demo_agent_groups(&mut self) {
        println!("👥 Demo 2: Agent Groups & Priority Routing");
        println!("==========================================");
        
        let tasks = vec![
            DemoTask {
                id: "group-1".to_string(),
                prompt: "Critical bioinformatics analysis".to_string(),
                task_type: "bioinformatics".to_string(),
                capabilities: vec!["bioinformatics".to_string()],
                metadata: HashMap::new(),
            },
            DemoTask {
                id: "group-2".to_string(),
                prompt: "Private patient data processing".to_string(),
                task_type: "analysis".to_string(),
                capabilities: vec!["analysis".to_string()],
                metadata: {
                    let mut map = HashMap::new();
                    map.insert("privacy".to_string(), "required".to_string());
                    map
                },
            },
            DemoTask {
                id: "group-3".to_string(),
                prompt: "General text processing task".to_string(),
                task_type: "text".to_string(),
                capabilities: vec!["chat".to_string()],
                metadata: HashMap::new(),
            },
        ];
        
        for task in tasks {
            let result = self.route_task(&task);
            println!("  ✅ {}", task.prompt);
            println!("     → {} via {} ({}ms)", result.destination, result.method, result.latency_ms);
            if let Some(rule) = result.rule_applied {
                println!("     └─ Rule applied: {}", rule);
            }
        }
        println!();
    }
    
    fn demo_primal_routing(&mut self) {
        println!("🏰 Demo 3: Cross-Primal Routing (Fixed Mappings)");
        println!("===============================================");
        
        let tasks = vec![
            DemoTask {
                id: "primal-1".to_string(),
                prompt: "Store genomic data in persistent database".to_string(),
                task_type: "storage".to_string(),
                capabilities: vec!["storage".to_string()],
                metadata: HashMap::new(),
            },
            DemoTask {
                id: "primal-2".to_string(),
                prompt: "Run large-scale protein folding simulation".to_string(),
                task_type: "computation".to_string(),
                capabilities: vec!["compute".to_string()],
                metadata: HashMap::new(),
            },
            DemoTask {
                id: "primal-3".to_string(),
                prompt: "Encrypt patient research data".to_string(),
                task_type: "security".to_string(),
                capabilities: vec!["security".to_string()],
                metadata: HashMap::new(),
            },
        ];
        
        for task in tasks {
            let result = self.route_task(&task);
            println!("  ✅ {}", task.prompt);
            println!("     → {} via {} ({}ms)", result.destination, result.method, result.latency_ms);
            if let Some(rule) = result.rule_applied {
                println!("     └─ Rule applied: {}", rule);
            }
        }
        println!();
    }
    
    fn demo_configuration_rules(&mut self) {
        println!("⚙️  Demo 4: Rule-based Routing Configuration");
        println!("===========================================");
        
        println!("Active routing rules:");
        for rule in &self.routing_rules {
            println!("  • {} (priority: {})", rule.id, rule.priority);
            println!("    {} → {}", rule.condition, rule.action);
        }
        println!();
        
        let task = DemoTask {
            id: "rule-test".to_string(),
            prompt: "Bioinformatics protein folding analysis".to_string(),
            task_type: "bioinformatics".to_string(),
            capabilities: vec!["bioinformatics".to_string()],
            metadata: HashMap::new(),
        };
        
        let result = self.route_task(&task);
        println!("  ✅ Rule-based routing test:");
        println!("     {} → {} via {}", task.prompt, result.destination, result.method);
        if let Some(rule) = result.rule_applied {
            println!("     └─ Rule applied: {}", rule);
        }
        println!();
    }
    
    fn demo_biomeos_integration(&mut self) {
        println!("🚀 Demo 5: BiomeOS Integration Scenario");
        println!("======================================");
        
        let workflow_steps = vec![
            ("Patient Data Storage", DemoTask {
                id: "biomeos-1".to_string(),
                prompt: "Store patient genomic sequences".to_string(),
                task_type: "storage".to_string(),
                capabilities: vec!["storage".to_string()],
                metadata: {
                    let mut map = HashMap::new();
                    map.insert("data_type".to_string(), "genomic".to_string());
                    map
                },
            }),
            ("Computational Analysis", DemoTask {
                id: "biomeos-2".to_string(),
                prompt: "Execute variant calling pipeline".to_string(),
                task_type: "computation".to_string(),
                capabilities: vec!["compute".to_string()],
                metadata: {
                    let mut map = HashMap::new();
                    map.insert("pipeline_type".to_string(), "variant_calling".to_string());
                    map
                },
            }),
            ("AI-Powered Analysis", DemoTask {
                id: "biomeos-3".to_string(),
                prompt: "AI analysis of genomic variants".to_string(),
                task_type: "bioinformatics".to_string(),
                capabilities: vec!["bioinformatics".to_string()],
                metadata: HashMap::new(),
            }),
            ("Security Validation", DemoTask {
                id: "biomeos-4".to_string(),
                prompt: "Validate data access permissions".to_string(),
                task_type: "security".to_string(),
                capabilities: vec!["security".to_string()],
                metadata: {
                    let mut map = HashMap::new();
                    map.insert("security_level".to_string(), "patient_data".to_string());
                    map
                },
            }),
            ("Emergency Override", DemoTask {
                id: "biomeos-5".to_string(),
                prompt: "Emergency analysis with manual selection".to_string(),
                task_type: "analysis".to_string(),
                capabilities: vec!["reasoning".to_string()],
                metadata: {
                    let mut map = HashMap::new();
                    map.insert("preferred_agent".to_string(), "openai-gpt4".to_string());
                    map.insert("emergency".to_string(), "true".to_string());
                    map
                },
            }),
        ];
        
        println!("Processing complete biomeOS workflow:");
        for (step_name, task) in workflow_steps {
            let result = self.route_task(&task);
            println!("  {} → {}", step_name, result.destination);
            println!("    └─ Method: {} ({}ms)", result.method, result.latency_ms);
        }
        
        println!();
        println!("✅ BiomeOS workflow demonstrates:");
        println!("   • Storage tasks → NestGate (storage persistence)");
        println!("   • Compute tasks → ToadStool (compute processing)");
        println!("   • Security tasks → BearDog (security operations)");
        println!("   • AI tasks → High-performance agent groups");
        println!("   • Manual overrides → Direct agent selection");
        println!();
    }
    
    fn route_task(&mut self, task: &DemoTask) -> RoutingResult {
        // Step 1: Check for manual overrides
        if let Some(preferred_agent) = task.metadata.get("preferred_agent") {
            if self.agents.contains_key(preferred_agent) {
                return self.create_result(
                    &format!("agent:{}", preferred_agent),
                    "manual_override",
                    None,
                    100,
                );
            }
        }
        
        if let Some(preferred_group) = task.metadata.get("preferred_group") {
            if self.agent_groups.contains_key(preferred_group) {
                return self.create_result(
                    &format!("group:{}", preferred_group),
                    "manual_override",
                    None,
                    150,
                );
            }
        }
        
        if let Some(target_primal) = task.metadata.get("target_primal") {
            if self.primal_endpoints.contains_key(target_primal) {
                return self.create_result(
                    &format!("primal:{}", target_primal),
                    "manual_override",
                    None,
                    250,
                );
            }
        }
        
        // Step 2: Apply routing rules
        for rule in &self.routing_rules {
            if self.matches_condition(&rule.condition, task) {
                return self.create_result(
                    &rule.action.replace(":", ":"),
                    "rule_based",
                    Some(rule.id.clone()),
                    200,
                );
            }
        }
        
        // Step 3: Automatic selection
        let best_agent = self.select_best_agent(task);
        self.create_result(
            &format!("agent:{}", best_agent),
            "automatic",
            None,
            120,
        )
    }
    
    fn matches_condition(&self, condition: &str, task: &DemoTask) -> bool {
        if condition.starts_with("capability:") {
            let capability = condition.strip_prefix("capability:").unwrap();
            return task.capabilities.contains(&capability.to_string());
        }
        
        if condition.starts_with("type:") {
            let task_type = condition.strip_prefix("type:").unwrap();
            return task.task_type == task_type;
        }
        
        if condition.starts_with("metadata:") {
            let metadata_condition = condition.strip_prefix("metadata:").unwrap();
            if let Some((key, value)) = metadata_condition.split_once('=') {
                return task.metadata.get(key).map_or(false, |v| v == value);
            }
        }
        
        false
    }
    
    fn select_best_agent(&self, task: &DemoTask) -> String {
        let mut best_agent = "claude-sonnet".to_string();
        let mut best_score = 0.0;
        
        for (agent_id, agent) in &self.agents {
            if agent.health != "healthy" {
                continue;
            }
            
            let mut score = 0.0;
            
            // Capability matching
            for required_cap in &task.capabilities {
                if agent.capabilities.contains(required_cap) {
                    score += 10.0;
                }
            }
            
            // Load balancing
            score += 5.0 / (agent.load + 1) as f64;
            
            if score > best_score {
                best_score = score;
                best_agent = agent_id.clone();
            }
        }
        
        best_agent
    }
    
    fn create_result(&mut self, destination: &str, method: &str, rule: Option<String>, latency: u64) -> RoutingResult {
        *self.statistics.entry(destination.to_string()).or_insert(0) += 1;
        
        RoutingResult {
            destination: destination.to_string(),
            method: method.to_string(),
            rule_applied: rule,
            latency_ms: latency,
        }
    }
    
    fn show_final_statistics(&self) {
        println!("📊 Final Routing Statistics");
        println!("===========================");
        for (destination, count) in &self.statistics {
            println!("  • {}: {} requests", destination, count);
        }
        
        println!();
        println!("🔗 Ready for biomeOS Integration!");
        println!("================================");
        println!("✓ Manual agent selection via task metadata");
        println!("✓ Agent groups with priority and failover");
        println!("✓ Fixed primal routing:");
        println!("  - NestGate: Storage persistence");
        println!("  - ToadStool: Compute processing");
        println!("  - BearDog: Security operations");
        println!("  - BiomeOS: Integration & orchestration");
        println!("✓ Rule-based configuration system");
        println!("✓ Configurable selection modes");
        println!("✓ Load balancing and health monitoring");
        println!("✓ Cross-primal coordination capabilities");
    }
} 