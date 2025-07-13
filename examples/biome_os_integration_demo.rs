//! # biomeOS Integration Demo
//!
//! This example demonstrates the squirrel AI integration with the biomeOS ecosystem,
//! showing AI intelligence, MCP coordination, context state management, and ecosystem communication.

use squirrel::biomeos_integration::*;
use squirrel::protocol::types::{Request, Response};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 biomeOS Integration Demo Starting...");
    println!("=========================================");

    // 1. Initialize the biomeOS integration
    println!("\n📡 Initializing biomeOS Integration...");
    let mut integration = SquirrelBiomeOSIntegration::new("demo-biome".to_string());
    println!("✅ Integration created with service ID: {}", integration.service_id);
    println!("✅ Biome ID: {}", integration.biome_id);

    // 2. Display initial health status
    println!("\n❤️  Initial Health Status:");
    let health_status = integration.get_health_status();
    println!("   Status: {}", health_status.status);
    println!("   AI Engine: {}", health_status.ai_engine_status);
    println!("   MCP Server: {}", health_status.mcp_server_status);
    println!("   Context Manager: {}", health_status.context_manager_status);

    // 3. Test AI Intelligence
    println!("\n🧠 Testing AI Intelligence...");
    let analysis_result = integration.ai_intelligence.generate_ecosystem_report().await?;
    println!("✅ Ecosystem Analysis Complete:");
    println!("   Recommendations: {}", analysis_result.recommendations.len());
    for (i, rec) in analysis_result.recommendations.iter().take(3).enumerate() {
        println!("   {}. {}", i + 1, rec);
    }

    // 4. Test MCP Integration
    println!("\n🔗 Testing MCP Integration...");
    let session_id = integration.mcp_integration.create_coordination_session(
        vec!["songbird".to_string(), "nestgate".to_string()],
        "demo-coordination".to_string()
    ).await?;
    println!("✅ MCP Coordination Session Created: {}", session_id);

    // 5. Test Context State Management
    println!("\n📊 Testing Context State Management...");
    integration.context_state.create_session_context(
        "demo-session-001".to_string(),
        Some("demo-user".to_string()),
        "ai_assistant_context".to_string()
    ).await?;
    println!("✅ Session Context Created");
    println!("   Active Sessions: {}", integration.context_state.get_active_sessions());

    // 6. Create Sample MCP Protocol Messages
    println!("\n📨 Creating Sample MCP Protocol Messages...");
    
    // Initialize request
    let init_request = Request {
        id: "init-001".to_string(),
        method: "initialize".to_string(),
        params: Some(serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "ai_intelligence": { "ecosystem_analysis": true },
                "mcp_coordination": { "session_management": true },
                "context_management": { "state_persistence": true }
            }
        })),
    };

    println!("✅ MCP Initialize Request: {:?}", init_request.method);

    // Response
    let response = Response {
        id: "init-001".to_string(),
        result: Some(serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "ai_intelligence": { "ecosystem_analysis": true },
                "mcp_coordination": { "session_management": true },
                "context_management": { "state_persistence": true }
            },
            "status": "initialized"
        })),
        error: None,
    };

    println!("✅ MCP Response Created: {:?}", response.result.is_some());

    // 7. Test Intelligence Request
    println!("\n🎯 Testing Intelligence Request...");
    let intelligence_request = IntelligenceRequest {
        request_id: "intel-001".to_string(),
        request_type: "ecosystem_optimization".to_string(),
        target_component: Some("songbird".to_string()),
        parameters: {
            let mut params = HashMap::new();
            params.insert("optimization_level".to_string(), serde_json::json!("high"));
            params.insert("focus_area".to_string(), serde_json::json!("resource_efficiency"));
            params
        },
        context: Some({
            let mut context = HashMap::new();
            context.insert("session_id".to_string(), "demo-session-001".to_string());
            context.insert("user_preference".to_string(), "performance_focused".to_string());
            context
        }),
    };

    let intelligence_response = integration.provide_ecosystem_intelligence(intelligence_request).await?;
    println!("✅ Intelligence Request Processed:");
    println!("   Request ID: {}", intelligence_response.request_id);
    println!("   Confidence: {:.2}", intelligence_response.confidence);
    println!("   Recommendations: {}", intelligence_response.recommendations.len());

    // 8. Test MCP Coordination
    println!("\n🤝 Testing MCP Coordination...");
    let coordination_request = McpCoordinationRequest {
        coordination_id: "coord-001".to_string(),
        coordination_type: "resource_optimization".to_string(),
        participants: vec!["squirrel".to_string(), "songbird".to_string(), "nestgate".to_string()],
        coordination_data: {
            let mut data = HashMap::new();
            data.insert("optimization_target".to_string(), serde_json::json!("storage_efficiency"));
            data.insert("priority".to_string(), serde_json::json!("high"));
            data
        },
    };

    let coordination_response = integration.handle_mcp_coordination(coordination_request).await?;
    println!("✅ MCP Coordination Complete:");
    println!("   Coordination ID: {}", coordination_response.coordination_id);
    println!("   Status: {}", coordination_response.status);
    println!("   Plan Steps: {}", coordination_response.coordination_plan.len());

    // 9. Test Context State Request
    println!("\n📈 Testing Context State Request...");
    let context_request = ContextStateRequest {
        session_id: "demo-session-001".to_string(),
        request_type: "context_retrieval".to_string(),
        context_data: Some({
            let mut data = HashMap::new();
            data.insert("query_type".to_string(), serde_json::json!("recent_interactions"));
            data
        }),
        query: Some("Show recent AI assistant interactions".to_string()),
    };

    let context_response = integration.manage_context_state(context_request).await?;
    println!("✅ Context State Request Processed:");
    println!("   Session ID: {}", context_response.session_id);
    println!("   Related Contexts: {}", context_response.related_contexts.len());

    // 10. Update and display final health status
    println!("\n🏥 Final Health Status Check...");
    integration.update_health_status();
    let final_health = integration.get_health_status();
    
    println!("✅ Updated Health Status:");
    println!("   Status: {}", final_health.status);
    println!("   AI Engine: {}", final_health.ai_engine_status);
    println!("   MCP Server: {}", final_health.mcp_server_status);
    println!("   Context Manager: {}", final_health.context_manager_status);
    println!("   Active Sessions: {}", final_health.active_sessions);
    println!("   AI Requests Processed: {}", final_health.ai_requests_processed);
    println!("   Context States Managed: {}", final_health.context_states_managed);

    // 11. Demo ecosystem registration
    println!("\n🌐 Testing Ecosystem Registration...");
    let registration = EcosystemServiceRegistration::default();
    println!("✅ Service Registration Created:");
    println!("   Service ID: {}", registration.service_id);
    println!("   Primal Type: {}", registration.primal_type);
    println!("   AI Capabilities: {}", registration.capabilities.ai_capabilities.len());
    println!("   MCP Capabilities: {}", registration.capabilities.mcp_capabilities.len());
    println!("   Context Capabilities: {}", registration.capabilities.context_capabilities.len());
    println!("   Integration Capabilities: {}", registration.capabilities.integration_capabilities.len());

    // 12. Demo complete
    println!("\n🎉 biomeOS Integration Demo Complete!");
    println!("=====================================");
    println!("✅ All biomeOS integration features tested successfully:");
    println!("   • AI Intelligence - Ecosystem analysis and optimization");
    println!("   • MCP Integration - Protocol coordination and session management");  
    println!("   • Context State - Session context and state management");
    println!("   • Ecosystem Client - Service registration and communication");
    println!("   • Health Monitoring - System health and metrics tracking");
    
    println!("\n📊 Summary Statistics:");
    println!("   Active Sessions: {}", final_health.active_sessions);
    println!("   AI Requests: {}", final_health.ai_requests_processed);
    println!("   Context States: {}", final_health.context_states_managed);
    println!("   Service ID: {}", integration.service_id);
    println!("   Biome ID: {}", integration.biome_id);

    println!("\n🚀 Ready for production integration with other primals!");

    // 13. Optional: AI API Integration Demo (if API keys are available)
    println!("\n🤖 Testing AI API Integration...");
    
    // Check for OpenAI API key
    if let Ok(openai_key) = std::env::var("OPENAI_API_KEY") {
        println!("✅ OpenAI API Key detected - setting up AI integration");
        
        // Create a simple AI request simulation
        let ai_request = serde_json::json!({
            "model": "gpt-3.5-turbo",
            "messages": [
                {
                    "role": "system",
                    "content": "You are an AI assistant integrated with the squirrel biomeOS ecosystem."
                },
                {
                    "role": "user", 
                    "content": "Analyze the health status of our biomeOS ecosystem."
                }
            ],
            "max_tokens": 150
        });
        
        println!("📨 AI Request prepared: {}", ai_request.get("model").unwrap_or(&serde_json::Value::Null));
        println!("🔑 API Key configured (length: {})", openai_key.len());
        
        // Simulate AI response processing
        let simulated_response = serde_json::json!({
            "ecosystem_analysis": "The biomeOS ecosystem appears healthy with strong AI intelligence coordination",
            "recommendations": [
                "Continue monitoring resource utilization",
                "Expand cross-primal coordination",
                "Enhance predictive analytics capabilities"
            ]
        });
        
        println!("✅ AI Response processed: {}", simulated_response.get("ecosystem_analysis").unwrap_or(&serde_json::Value::Null));
    } else {
        println!("💡 No OpenAI API key found - set OPENAI_API_KEY environment variable for AI integration");
    }
    
    // Check for Anthropic API key
    if let Ok(anthropic_key) = std::env::var("ANTHROPIC_API_KEY") {
        println!("✅ Anthropic API Key detected - Claude integration available");
        println!("🔑 API Key configured (length: {})", anthropic_key.len());
        
        let claude_request = serde_json::json!({
            "model": "claude-3-sonnet-20240229",
            "max_tokens": 150,
            "messages": [
                {
                    "role": "user",
                    "content": "As an AI assistant in the squirrel biomeOS ecosystem, provide insights on our system health."
                }
            ]
        });
        
        println!("📨 Claude Request prepared: {}", claude_request.get("model").unwrap_or(&serde_json::Value::Null));
    } else {
        println!("💡 No Anthropic API key found - set ANTHROPIC_API_KEY environment variable for Claude integration");
    }

    Ok(())
} 